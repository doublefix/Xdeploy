use ssh2::Session;
use std::env;
use std::error::Error;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::Path;

// 认证方式枚举
#[derive(Debug)]
pub enum AuthMethod {
    Key {
        pubkey_path: String,
        privkey_path: String,
        passphrase: Option<String>,
    },
    // 可扩展：Password(String)
}

// SSH 配置
#[derive(Debug)]
pub struct SshConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub auth: AuthMethod,
}

// SSH 客户端结构体
pub struct SshClient {
    session: Session,
}

impl SshClient {
    /// 连接到远程主机
    pub fn connect(config: &SshConfig) -> Result<Self, Box<dyn Error>> {
        let tcp = TcpStream::connect(format!("{}:{}", config.host, config.port))?;
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

        Ok(SshClient { session })
    }

    /// 执行单条命令并返回输出
    pub fn exec_command(&self, command: &str) -> Result<String, Box<dyn Error>> {
        let mut channel = self.session.channel_session()?;
        channel.exec(command)?;

        let mut output = String::new();
        channel.read_to_string(&mut output)?;
        channel.wait_close()?;

        Ok(output)
    }

    /// 执行多行脚本（如 Shell 脚本）并返回输出
    pub fn exec_script(&self, script: &str) -> Result<String, Box<dyn Error>> {
        let mut channel = self.session.channel_session()?;
        channel.shell()?;

        // 发送脚本内容
        channel.write_all(script.as_bytes())?;
        channel.write_all(b"exit\n")?; // 确保脚本执行后退出

        // 读取输出
        let mut output = String::new();
        channel.read_to_string(&mut output)?;
        channel.wait_close()?;

        Ok(output)
    }
}

#[test]
fn test_run_cmd_on_server() -> Result<(), Box<dyn Error>> {
    let home = env::var("HOME")?;
    let config = SshConfig {
        host: "ubuntu".to_string(),
        port: 22,
        username: "root".to_string(),
        auth: AuthMethod::Key {
            pubkey_path: format!("{home}/.ssh/id_rsa.pub"),
            privkey_path: format!("{home}/.ssh/id_rsa"),
            passphrase: None,
        },
    };

    // 连接到远程主机
    let client = SshClient::connect(&config)?;

    // 示例 1：执行单条命令
    let image_id = "ee65adc925d6d5acd33beeba4747f90fda68bec1dbea6a1dea16691fe9fdfeb8";
    let package = "chess-kubernetes-v1.31.0.tar.gz";
    let source_path = format!("/tmp/.chess/{image_id}/{package}");
    let target_path = format!("/tmp/.chess/{image_id}/test");
    let cmd = format!("tar -zxvf {source_path} -C {target_path}");
    println!("{cmd}");
    let output = client.exec_command(&cmd)?;
    println!("Command output:\n{output}");
    //

    // // 示例 2：执行多行脚本
    // let script = r#"
    //     echo "Start at $(date)"
    //     df -h
    //     echo "Done!"
    // "#;
    // let output = client.exec_script(script)?;
    // println!("Script output:\n{output}");

    Ok(())
}
