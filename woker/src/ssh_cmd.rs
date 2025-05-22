use futures::future::join_all;
use ssh2::Session;
use std::time::Duration;
use std::{error::Error, io::Read, net::TcpStream, path::Path, sync::Arc};
use tokio::task;

#[derive(Debug, Clone)]
pub enum AuthMethod {
    Key {
        pubkey_path: String,
        privkey_path: String,
        passphrase: Option<String>,
    },
}

#[derive(Debug, Clone)]
pub struct SshConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub auth: AuthMethod,
}

#[derive(Clone)]
pub struct SshClient {
    config: Arc<SshConfig>,
}

impl SshClient {
    pub fn new(config: SshConfig) -> Self {
        SshClient {
            config: Arc::new(config),
        }
    }

    /// Execute a single command
    pub async fn exec_command(
        &self,
        command: String,
    ) -> Result<String, Box<dyn Error + Send + Sync>> {
        let config = self.config.clone();
        task::spawn_blocking(move || {
            let tcp = TcpStream::connect(format!("{}:{}", config.host, config.port))?;
            tcp.set_read_timeout(Some(Duration::from_secs(30)))?;
            tcp.set_write_timeout(Some(Duration::from_secs(30)))?;

            let mut session = Session::new()?;
            session.set_tcp_stream(tcp);
            session.handshake()?;

            match &config.auth {
                AuthMethod::Key {
                    pubkey_path,
                    privkey_path,
                    passphrase,
                } => {
                    session.userauth_pubkey_file(
                        &config.username,
                        Some(Path::new(pubkey_path)),
                        Path::new(privkey_path),
                        passphrase.as_deref(),
                    )?;
                }
            }

            if !session.authenticated() {
                return Err("SSH authentication failed".into());
            }

            let mut channel = session.channel_session()?;
            channel.exec(&command)?;

            let mut output = String::new();
            channel.read_to_string(&mut output)?;
            channel.wait_close()?;

            Ok(output)
        })
        .await?
    }

    /// Execute multiple commands in sequence
    pub async fn exec_commands(
        &self,
        commands: Vec<String>,
    ) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
        let config = self.config.clone();
        task::spawn_blocking(move || {
            let tcp = TcpStream::connect(format!("{}:{}", config.host, config.port))?;
            tcp.set_read_timeout(Some(Duration::from_secs(30)))?;
            tcp.set_write_timeout(Some(Duration::from_secs(30)))?;

            let mut session = Session::new()?;
            session.set_tcp_stream(tcp);
            session.handshake()?;

            match &config.auth {
                AuthMethod::Key {
                    pubkey_path,
                    privkey_path,
                    passphrase,
                } => {
                    session.userauth_pubkey_file(
                        &config.username,
                        Some(Path::new(pubkey_path)),
                        Path::new(privkey_path),
                        passphrase.as_deref(),
                    )?;
                }
            }

            if !session.authenticated() {
                return Err("SSH authentication failed".into());
            }

            let mut outputs = Vec::new();
            for command in commands {
                let mut channel = session.channel_session()?;
                channel.exec(&command)?;

                let mut output = String::new();
                channel.read_to_string(&mut output)?;
                channel.wait_close()?;
                outputs.push(output);
            }

            Ok(outputs)
        })
        .await?
    }
}

pub async fn run_commands_on_multiple_hosts(
    configs: Vec<SshConfig>,
    commands: Vec<String>,
) -> Vec<(String, Result<Vec<String>, Box<dyn Error + Send + Sync>>)> {
    let mut tasks = Vec::new();

    for config in configs {
        let host = config.host.clone();
        let client = SshClient::new(config);
        let cmds = commands.clone();

        let task = tokio::spawn(async move {
            let result = client.exec_commands(cmds).await;
            (host, result)
        });

        tasks.push(task);
    }

    join_all(tasks)
        .await
        .into_iter()
        .map(|res| match res {
            Ok(pair) => pair,
            Err(e) => (
                "unknown".to_string(),
                Err(format!("Join error: {e}").into()),
            ),
        })
        .collect()
}

pub fn build_std_linux_tar_zxvf_commands(image_ids: &[String]) -> Vec<String> {
    image_ids
        .iter()
        .flat_map(|image_id| {
            let package = "chess*.gz";
            let source_path = format!("/tmp/.chess/{image_id}/{package}");
            let target_path = "/".to_string();

            vec![
                // format!("mkdir -p /tmp/.chess/{}", image_id),
                format!("tar -zxvf {} -C {}", source_path, target_path),
            ]
        })
        .collect()
}
