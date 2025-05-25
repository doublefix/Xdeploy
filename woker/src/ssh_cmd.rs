use futures::future::join_all;
use log::info;
use ssh2::Session;
use std::collections::HashMap;
use std::fmt;
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
    verbose: bool,
) -> Vec<(String, Result<Vec<String>, Box<dyn Error + Send + Sync>>)> {
    let mut tasks = Vec::new();

    for config in configs {
        let host = config.host.clone();
        let client = SshClient::new(config);
        let cmds = commands.clone();

        let task = tokio::spawn(async move {
            let result = client.exec_commands(cmds).await;

            // 根据 verbose 决定是否打印
            if verbose {
                match &result {
                    Ok(outputs) => {
                        info!("[{host}] Command outputs:");
                        for (i, output) in outputs.iter().enumerate() {
                            println!("  Command {}: {}", i + 1, output);
                        }
                    }
                    Err(e) => info!("[{host}] Error: {e}"),
                }
            }

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

pub fn build_std_linux_tarzxvf_filetoroot_commands(image_ids: &[String]) -> Vec<String> {
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

pub fn build_std_linux_init_node_commands(
    env: &HashMap<&str, &str>,
    image_ids: &[String],
) -> Vec<String> {
    let env_vars: Vec<String> = env.iter().map(|(k, v)| format!("{k}={v}")).collect();
    let env_part = env_vars.join(" ");

    image_ids
        .iter()
        .map(|image_id| format!("{env_part} bash /tmp/.chess/{image_id}/run.sh"))
        .collect()
}

#[derive(Debug, Clone, Default)]
pub struct KubeJoinInfo {
    pub kube_api_server: Option<String>,
    pub kube_join_token: Option<String>,
    pub kube_ca_cert_hash: Option<String>,
}

impl fmt::Display for KubeJoinInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "KubeJoinInfo {{\n  kube_api_server: {:?},\n  kube_join_token: {:?},\n  kube_ca_cert_hash: {:?}\n}}",
            self.kube_api_server, self.kube_join_token, self.kube_ca_cert_hash
        )
    }
}

impl SshClient {
    pub async fn get_kube_join_info(&self) -> Result<KubeJoinInfo, Box<dyn Error + Send + Sync>> {
        let output = self
            .exec_command("kubeadm token create --print-join-command".to_string())
            .await?;

        Self::parse_kubeadm_output(&output)
    }

    fn parse_kubeadm_output(output: &str) -> Result<KubeJoinInfo, Box<dyn Error + Send + Sync>> {
        let mut info = KubeJoinInfo::default();
        let parts: Vec<&str> = output.split_whitespace().collect();

        if parts.len() >= 6 {
            if let Some(api_server) = parts.get(2) {
                info.kube_api_server = Some(api_server.to_string());
            }
            if let Some(token) = parts.get(4) {
                info.kube_join_token = Some(token.to_string());
            }
            if let Some(hash_part) = parts.last() {
                info.kube_ca_cert_hash = Some(hash_part.to_string());
            }
        }

        Ok(info)
    }
}
