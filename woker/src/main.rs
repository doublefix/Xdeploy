use std::{
    collections::HashMap,
    io::{Read, Write},
    net::TcpStream,
    sync::Arc,
};
use tokio::{
    sync::{Mutex, mpsc},
    task::JoinHandle,
};
use tokio_stream::StreamExt;
use tonic::Request;

pub mod ssh {
    tonic::include_proto!("ssh");
}

use ssh::{
    SshError, SshInit, SshInput, SshOutput, SshStream, ssh_service_client::SshServiceClient,
};

struct SshSession {
    tx: mpsc::Sender<SshInput>,
    #[allow(dead_code)]
    task: JoinHandle<()>,
}

#[derive(Clone)]
struct AppState {
    ssh_sessions: Arc<Mutex<HashMap<String, SshSession>>>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let grpc_addr = "http://localhost:50052";
    let mut client = SshServiceClient::connect(grpc_addr).await?;

    let (stream_tx, stream_rx) = mpsc::channel::<SshStream>(100);
    let state = AppState {
        ssh_sessions: Arc::new(Mutex::new(HashMap::new())),
    };

    let mut response_stream = client
        .start_stream(Request::new(tokio_stream::wrappers::ReceiverStream::new(
            stream_rx,
        )))
        .await?
        .into_inner();

    let state_clone = state.clone();
    let stream_tx_clone = stream_tx.clone();
    tokio::spawn(async move {
        while let Some(message) = response_stream.next().await {
            match message {
                Ok(stream) => {
                    if let Err(e) =
                        handle_server_message(stream, state_clone.clone(), stream_tx_clone.clone())
                            .await
                    {
                        eprintln!("Error handling server message: {e}");
                    }
                }
                Err(e) => eprintln!("gRPC stream error: {e}"),
            }
        }
    });

    println!("Client started. Press Ctrl+C to exit.");
    tokio::signal::ctrl_c().await?;
    println!("Exiting...");
    Ok(())
}

async fn handle_server_message(
    stream: SshStream,
    state: AppState,
    stream_tx: mpsc::Sender<SshStream>,
) -> Result<(), Box<dyn std::error::Error>> {
    match stream.payload {
        Some(ssh::ssh_stream::Payload::Init(init)) => {
            start_ssh_session(init, stream.session_id, state, stream_tx).await?;
        }
        Some(ssh::ssh_stream::Payload::Input(input)) => {
            send_ssh_input(stream.session_id, input, state).await?;
        }
        _ => {}
    }
    Ok(())
}

async fn start_ssh_session(
    init: SshInit,
    session_id: String,
    state: AppState,
    stream_tx: mpsc::Sender<SshStream>,
) -> Result<(), Box<dyn std::error::Error>> {
    let tcp = TcpStream::connect(format!("{}:{}", init.host, init.port))?;
    let mut sess = ssh2::Session::new()?;
    sess.set_tcp_stream(tcp);
    sess.handshake()?;
    sess.userauth_password(&init.username, &init.password)?;
    let mut channel = sess.channel_session()?;
    channel.request_pty(
        "xterm",
        None,
        Some((init.cols as u32, init.rows as u32, 0, 0)),
    )?;
    channel.shell()?;

    let (tx, mut rx) = mpsc::channel::<SshInput>(100);

    let mut read_channel = channel.clone();
    let session_id_clone = session_id.clone();
    let stream_tx_clone = stream_tx.clone();

    let output_task = tokio::spawn(async move {
        let mut buf = [0u8; 1024];
        loop {
            match read_channel.read(&mut buf) {
                Ok(n) if n > 0 => {
                    let data = String::from_utf8_lossy(&buf[..n]).to_string();
                    let _ = stream_tx_clone
                        .send(SshStream {
                            session_id: session_id_clone.clone(),
                            payload: Some(ssh::ssh_stream::Payload::Output(SshOutput { data })),
                        })
                        .await;
                }
                Ok(_) => break, // EOF
                Err(e) => {
                    let _ = stream_tx_clone
                        .send(SshStream {
                            session_id: session_id_clone.clone(),
                            payload: Some(ssh::ssh_stream::Payload::Error(SshError {
                                message: e.to_string(),
                            })),
                        })
                        .await;
                    break;
                }
            }
        }
    });

    let mut write_channel = channel;
    tokio::spawn(async move {
        while let Some(input) = rx.recv().await {
            if let Err(e) = write_channel.write_all(input.data.as_bytes()) {
                eprintln!("Failed to write to SSH channel: {e}");
                break;
            }
        }
    });

    state.ssh_sessions.lock().await.insert(
        session_id,
        SshSession {
            tx,
            task: output_task,
        },
    );
    Ok(())
}

async fn send_ssh_input(
    session_id: String,
    input: SshInput,
    state: AppState,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(session) = state.ssh_sessions.lock().await.get(&session_id) {
        session.tx.send(input).await?;
    }
    Ok(())
}
