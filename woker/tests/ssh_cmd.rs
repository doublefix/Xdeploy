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

    /// 使用 spawn_blocking 包装阻塞 SSH 连接与命令执行
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
}

pub async fn run_command_on_multiple_hosts(
    configs: Vec<SshConfig>,
    command: String,
) -> Vec<(String, Result<String, Box<dyn Error + Send + Sync>>)> {
    let mut tasks = Vec::new();

    for config in configs {
        let host = config.host.clone();
        let client = SshClient::new(config);
        let cmd = command.clone();

        let task = tokio::spawn(async move {
            let result = client.exec_command(cmd).await;
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

#[tokio::test]
async fn test_main() -> Result<(), Box<dyn Error>> {
    let home = std::env::var("HOME")?;
    let image_id = "ee65adc925d6d5acd33beeba4747f90fda68bec1dbea6a1dea16691fe9fdfeb8";
    let package = "chess-kubernetes-v1.31.0.tar.gz";
    let source_path = format!("/tmp/.chess/{image_id}/{package}");
    let target_path = format!("/tmp/.chess/{image_id}/test");
    let command = format!("tar -zxvf {source_path} -C {target_path}");

    let hosts = vec!["ubuntu"];
    let configs: Vec<SshConfig> = hosts
        .into_iter()
        .map(|host| SshConfig {
            host: host.to_string(),
            port: 22,
            username: "root".to_string(),
            auth: AuthMethod::Key {
                pubkey_path: format!("{home}/.ssh/id_rsa.pub"),
                privkey_path: format!("{home}/.ssh/id_rsa"),
                passphrase: None,
            },
        })
        .collect();

    let results = run_command_on_multiple_hosts(configs, command).await;

    for (host, result) in results {
        match result {
            Ok(output) => println!("✅ [{host}] Output:\n{output}"),
            Err(e) => eprintln!("❌ [{host}] Error: {e}"),
        }
    }

    Ok(())
}
