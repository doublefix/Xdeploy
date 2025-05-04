use ssh2::Session;
use std::collections::HashMap;
use std::io::Write;
use std::net::TcpStream;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::mpsc;

mod ssh {
    tonic::include_proto!("ssh");
}

use ssh::ssh_service_client::SshServiceClient;
use ssh::{SshClose, SshData, SshStreamMessage, ssh_stream_message::Payload};

enum AuthMethod {
    Password(String),
    Key {
        pubkey: String,
        privkey: String,
        passphrase: Option<String>,
    },
}

struct SshConfig {
    host: String,
    port: u16,
    username: String,
    auth: AuthMethod,
}

struct SshSession {
    // session: Session,
    channel: ssh2::Channel,
}

type SharedSessions = Arc<Mutex<HashMap<String, SshSession>>>;

fn connect_ssh(config: &SshConfig) -> Result<SshSession, Box<dyn std::error::Error + Send + Sync>> {
    let tcp = TcpStream::connect(format!("{}:{}", config.host, config.port))?;
    let mut sess = Session::new()?;
    sess.set_tcp_stream(tcp);
    sess.handshake()?;

    match &config.auth {
        AuthMethod::Password(pw) => {
            println!("[auth] Using password authentication...");
            sess.userauth_password(&config.username, pw)?;
        }
        AuthMethod::Key {
            pubkey,
            privkey,
            passphrase,
        } => {
            println!("[auth] Using key authentication...");
            sess.userauth_pubkey_file(
                &config.username,
                Some(Path::new(pubkey)),
                Path::new(privkey),
                passphrase.as_deref(),
            )?;
        }
    }

    if !sess.authenticated() {
        return Err("Authentication failed".into());
    }

    let mut channel = sess.channel_session()?;
    channel.request_pty("xterm", None, Some((80, 24, 0, 0)))?;
    channel.shell()?;

    Ok(SshSession {
        // session: sess,
        channel,
    })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = SshServiceClient::connect("http://127.0.0.1:50051").await?;
    let (tx, rx) = mpsc::channel(32);

    // Shared state for managing SSH sessions
    let ssh_sessions: SharedSessions = Arc::new(Mutex::new(HashMap::new()));

    // Spawn a task to handle incoming messages from Manager
    let mut stream = client
        .start_stream(tokio_stream::wrappers::ReceiverStream::new(rx))
        .await?
        .into_inner();
    let ssh_sessions_clone = ssh_sessions.clone();
    tokio::spawn(async move {
        while let Some(msg) = stream.message().await.unwrap_or(None) {
            match msg.payload {
                Some(Payload::Init(init)) => {
                    println!("Received SshInit: {init:?}");
                    let ssh_sessions = ssh_sessions_clone.clone();
                    tokio::spawn(async move {
                        if let Err(e) = handle_ssh_init(init, ssh_sessions).await {
                            eprintln!("Error handling SshInit: {e:?}");
                        }
                    });
                }
                Some(Payload::Data(data)) => {
                    println!("Received SshData: {data:?}");
                    let ssh_sessions = ssh_sessions_clone.clone();
                    tokio::spawn(async move {
                        if let Err(e) = handle_ssh_data(data, ssh_sessions).await {
                            eprintln!("Error handling SshData: {e:?}");
                        }
                    });
                }
                Some(Payload::Close(close)) => {
                    println!("Session closed: {close:?}");
                    let ssh_sessions = ssh_sessions_clone.clone();
                    tokio::spawn(async move {
                        if let Err(e) = handle_ssh_close(close, ssh_sessions).await {
                            eprintln!("Error handling SshClose: {e:?}");
                        }
                    });
                }
                _ => {}
            }
        }
    });

    // Send AgentHello to register with Manager
    tx.send(SshStreamMessage {
        payload: Some(Payload::Hello(ssh::AgentHello {
            agent_id: "agent-123".to_string(),
            hostname: "localhost".to_string(),
        })),
    })
    .await?;

    println!("Agent connected to Manager");
    Ok(())
}

async fn handle_ssh_init(
    init: ssh::SshInit,
    ssh_sessions: SharedSessions,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let auth = match init.auth {
        Some(ssh::ssh_init::Auth::Password(password)) => AuthMethod::Password(password.password),
        Some(ssh::ssh_init::Auth::Key(key)) => AuthMethod::Key {
            pubkey: key.pubkey_path,
            privkey: key.privkey_path,
            passphrase: Some(key.passphrase),
        },
        None => return Err("No authentication method provided".into()),
    };

    let config = SshConfig {
        host: init.target_host,
        port: init.target_port as u16,
        username: init.username,
        auth,
    };

    match connect_ssh(&config) {
        Ok(ssh_session) => {
            let session_id = init.session_id.clone();
            ssh_sessions
                .lock()
                .await
                .insert(session_id.clone(), ssh_session);
            println!("SSH session established for session_id: {session_id}");
        }
        Err(e) => {
            eprintln!("Failed to establish SSH session: {e:?}");
        }
    }

    Ok(())
}

async fn handle_ssh_data(
    data: SshData,
    ssh_sessions: SharedSessions,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut sessions = ssh_sessions.lock().await;
    if let Some(ssh_session) = sessions.get_mut(&data.session_id) {
        ssh_session.channel.write_all(&data.data)?;
    } else {
        eprintln!("No SSH session found for session_id: {}", data.session_id);
    }
    Ok(())
}

async fn handle_ssh_close(
    close: SshClose,
    ssh_sessions: SharedSessions,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut sessions = ssh_sessions.lock().await;
    if let Some(mut ssh_session) = sessions.remove(&close.session_id) {
        ssh_session.channel.close()?;
        println!("SSH session closed for session_id: {}", close.session_id);
    } else {
        eprintln!("No SSH session found for session_id: {}", close.session_id);
    }
    Ok(())
}
