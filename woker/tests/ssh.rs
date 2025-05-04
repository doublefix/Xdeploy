use ssh2::Session;
use std::env;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::Path;

enum AuthMethod {
    Password(String),
    Key {
        pubkey: String,
        privkey: String,
        passphrase: Option<String>, // 可选支持 key 密码
    },
}

struct SshConfig {
    host: String,
    port: u16,
    username: String,
    auth: AuthMethod,
}

fn connect_ssh(config: &SshConfig) -> Result<Session, Box<dyn std::error::Error>> {
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

    Ok(sess)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let home = env::var("HOME")?;
    let config = SshConfig {
        host: "ubuntu".to_string(),
        port: 22,
        username: "root".to_string(),
        auth: AuthMethod::Key {
            pubkey: format!("{home}/.ssh/id_rsa.pub"),
            privkey: format!("{home}/.ssh/id_rsa"),
            passphrase: None,
        },
        // 或者使用密码认证:
        // auth: AuthMethod::Password("your-password".to_string()),
    };

    let sess = connect_ssh(&config)?;
    let mut channel = sess.channel_session()?;
    channel.request_pty("xterm", None, Some((80, 24, 0, 0)))?;
    channel.shell()?;

    println!("SSH session established. Type 'exit' to quit.");

    let mut input = String::new();
    let mut buf = [0; 1024];

    loop {
        match channel.read(&mut buf) {
            Ok(n) if n > 0 => {
                print!("{}", String::from_utf8_lossy(&buf[..n]));
            }
            _ => {}
        }

        std::io::stdin().read_line(&mut input)?;
        if input.trim() == "exit" {
            break;
        }

        channel.write_all(input.as_bytes())?;
        input.clear();
    }

    channel.close()?;
    println!("Session closed");
    Ok(())
}
