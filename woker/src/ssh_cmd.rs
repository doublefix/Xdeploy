use log::info;
use ssh2::Session;
use std::collections::HashMap;
use std::fmt;
use std::io::Write;
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
                let mut stdout = String::new();
                channel.read_to_string(&mut stdout)?;

                let mut stderr = String::new();
                channel.stderr().read_to_string(&mut stderr)?;
                channel.wait_close()?;

                let mut output = stdout;
                if !stderr.is_empty() {
                    if !output.is_empty() {
                        output.push('\n');
                    }
                    output.push_str(&stderr);
                }

                outputs.push(output);
            }

            Ok(outputs)
        })
        .await?
    }

    pub async fn exec_commands_stream(
        &self,
        commands: Vec<String>,
        verbose: bool,
    ) -> Result<Vec<(String, i32)>, Box<dyn Error + Send + Sync>> {
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

            let mut results = Vec::new();
            for command in commands {
                let mut channel = session.channel_session()?;
                channel.exec(&command)?;

                let mut output = String::new();
                let mut buf = [0u8; 1024];

                // 读取 stdout
                loop {
                    let n = channel.read(&mut buf)?;
                    if n == 0 {
                        break;
                    }
                    let chunk = String::from_utf8_lossy(&buf[..n]);

                    if verbose {
                        print!("{chunk}");
                        std::io::stdout().flush()?;
                    }
                    output.push_str(&chunk);
                }

                // 读取 stderr
                loop {
                    let n = channel.stderr().read(&mut buf)?;
                    if n == 0 {
                        break;
                    }
                    let chunk = String::from_utf8_lossy(&buf[..n]);

                    if verbose {
                        eprint!("{chunk}");
                        std::io::stderr().flush()?;
                    }
                    output.push_str(&chunk);
                }

                channel.wait_close()?;
                let exit_status = channel.exit_status()?;

                results.push((output, exit_status));
            }

            Ok(results)
        })
        .await?
    }
}

pub async fn run_commands_on_multiple_hosts(
    configs: Vec<SshConfig>,
    commands: Vec<String>,
    verbose: bool,
) -> bool {
    use futures::stream::StreamExt;

    let mut all_success = true;
    let mut tasks = futures::stream::iter(configs.into_iter().map(|config| {
        let host = config.host.clone();
        let cmds = commands.clone();

        tokio::spawn(async move {
            info!("🚀 Starting commands on host: {host}");
            let mut host_success = true;

            let client = SshClient::new(config);
            for (i, cmd) in cmds.into_iter().enumerate() {
                info!("🔧 [{}] Executing command {}: {}", host, i + 1, cmd);

                match client.exec_commands_stream(vec![cmd], verbose).await {
                    Ok(results) => {
                        for (_, status) in results {
                            if status != 0 {
                                host_success = false;
                                let status_msg = format!("❌ Failed (status: {status})");
                                info!("[{host}] Command output: {status_msg}");
                            } else {
                                info!("[{host}] Command output: ✅ Success");
                            }
                        }
                    }
                    Err(e) => {
                        host_success = false;
                        info!("[{host}] ❗ Error executing command: {e}");
                    }
                }
            }

            info!("🏁 Finished commands on host: {host}");
            host_success
        })
    }))
    .buffer_unordered(10);

    while let Some(task_result) = tasks.next().await {
        match task_result {
            Ok(host_success) => {
                if !host_success {
                    all_success = false;
                }
            }
            Err(e) => {
                all_success = false;
                info!("⚠️ Task failed: {e}");
            }
        }
    }

    all_success
}

pub fn build_std_linux_tarzxvf_filetoroot_commands(image_ids: &[String]) -> Vec<String> {
    image_ids
        .iter()
        .flat_map(|image_id| {
            let package = "*.gz";
            let source_path = format!("/tmp/.chess/{image_id}/{package}");
            let target_path = "/".to_string();

            vec![format!(
                r#"if ls {} 1>/dev/null 2>&1; then tar -zxvf {} -C {}; fi"#,
                source_path, source_path, target_path
            )]
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
