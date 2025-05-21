use rayon::prelude::*;
use ssh2::Session;
use std::{env, path::Path};
use tokio::net::TcpStream as AsyncTcpStream;

#[derive(Debug, Clone)]
pub struct HostConfig {
    pub ip: String,
    pub port: u16,
    pub username: String,
    pub privkey_path: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug)]
pub struct HostCheckResult {
    pub ip: String,
    pub username: String,
    pub ssh_accessible: bool,
    pub auth_method: Option<AuthMethod>,
    pub has_root_access: bool,
    pub has_passwordless_sudo: bool,
    pub can_sudo_with_password: bool,
}

#[derive(Debug, PartialEq)]
pub enum AuthMethod {
    PublicKey,
    Password,
}

impl HostCheckResult {
    pub fn new(ip: String, username: String) -> Self {
        HostCheckResult {
            ip,
            username,
            ssh_accessible: false,
            auth_method: None,
            has_root_access: false,
            has_passwordless_sudo: false,
            can_sudo_with_password: false,
        }
    }
}

async fn check_single_host_async(host: &HostConfig) -> HostCheckResult {
    let mut result = HostCheckResult::new(host.ip.clone(), host.username.clone());

    // Rest of the function remains the same...
    let conn = match AsyncTcpStream::connect((host.ip.as_str(), host.port)).await {
        Ok(stream) => stream.into_std().unwrap(),
        Err(_) => return result,
    };

    let mut sess = match Session::new() {
        Ok(s) => s,
        Err(_) => return result,
    };
    sess.set_tcp_stream(conn);
    if sess.handshake().is_err() {
        return result;
    }

    let mut authenticated = false;

    if let Some(privkey_path) = &host.privkey_path {
        authenticated = sess
            .userauth_pubkey_file(&host.username, None, Path::new(privkey_path), None)
            .is_ok()
            && sess.authenticated();

        if authenticated {
            result.auth_method = Some(AuthMethod::PublicKey);
        }
    }

    if !authenticated {
        if let Some(password) = &host.password {
            authenticated =
                sess.userauth_password(&host.username, password).is_ok() && sess.authenticated();

            if authenticated {
                result.auth_method = Some(AuthMethod::Password);
            }
        }
    }

    if !authenticated {
        return result;
    }

    result.ssh_accessible = true;

    if host.username == "root" {
        result.has_root_access = true;
        result.has_passwordless_sudo = true;
        return result;
    }

    let (passwordless_sudo, password_sudo) = check_sudo_privileges(&sess, host.password.as_ref());
    result.has_passwordless_sudo = passwordless_sudo;
    result.can_sudo_with_password = password_sudo;

    result
}

fn check_sudo_privileges(sess: &Session, password: Option<&String>) -> (bool, bool) {
    // First check passwordless sudo
    let passwordless = check_passwordless_sudo(sess);

    // Then check sudo with password if provided
    let with_password = if let Some(password) = password {
        check_sudo_with_password(sess, password)
    } else {
        false
    };

    (passwordless, with_password)
}

fn check_passwordless_sudo(sess: &Session) -> bool {
    let mut channel = match sess.channel_session() {
        Ok(ch) => ch,
        Err(_) => return false,
    };

    let command = r#"
    if sudo -n true 2>/dev/null; then
        exit 0
    fi
    if sudo -l 2>/dev/null | grep -q '(ALL) NOPASSWD: ALL'; then
        exit 0
    fi
    exit 1
    "#;

    if channel.exec(command).is_err() {
        return false;
    }

    channel.wait_eof().ok();
    channel.close().ok();
    channel.wait_close().ok();

    channel.exit_status().unwrap_or(1) == 0
}

fn check_sudo_with_password(sess: &Session, password: &String) -> bool {
    let mut channel = match sess.channel_session() {
        Ok(ch) => ch,
        Err(_) => return false,
    };

    // Use a more secure way to pass password (though still not perfect)
    let command = format!("echo '{password}' | sudo -S --prompt=\"\" true 2>/dev/null");

    if channel.exec(&command).is_err() {
        return false;
    }

    channel.wait_eof().ok();
    channel.close().ok();
    channel.wait_close().ok();

    channel.exit_status().unwrap_or(1) == 0
}

pub fn bulk_check_hosts(hosts: Vec<HostConfig>) -> Vec<HostCheckResult> {
    let rt = tokio::runtime::Runtime::new().unwrap();

    hosts
        .par_iter()
        .map(|host| rt.block_on(check_single_host_async(host)))
        .collect()
}

#[test]
fn test_bulk_check() {
    let home = env::var("HOME").unwrap();
    let hosts = vec![
        HostConfig {
            ip: "ubuntu".to_string(),
            port: 22,
            username: "root".to_string(),
            privkey_path: Some(format!("{home}/.ssh/id_rsa")),
            password: None,
        },
        HostConfig {
            ip: "ubuntu".to_string(),
            port: 22,
            username: "mahongqin".to_string(),
            privkey_path: Some(format!("{home}/.ssh/id_rsa")),
            password: Some("ma@4056".to_string()),
        },
        HostConfig {
            ip: "localhost".to_string(),
            port: 22,
            username: "alice".to_string(),
            privkey_path: None,
            password: Some("123456".to_string()),
        },
    ];

    let results = bulk_check_hosts(hosts);
    for result in results {
        println!("{result:#?}");
        println!(
            "{} (user: {}) - SSH: {} ({:?}), Root: {}, Passwordless Sudo: {}, Sudo with Password: {}",
            result.ip,
            result.username, // Added username to output
            if result.ssh_accessible { "✅" } else { "❌" },
            result.auth_method,
            if result.has_root_access { "✅" } else { "❌" },
            if result.has_passwordless_sudo {
                "✅"
            } else {
                "❌"
            },
            if result.can_sudo_with_password {
                "✅"
            } else {
                "❌"
            },
        );
    }
}
