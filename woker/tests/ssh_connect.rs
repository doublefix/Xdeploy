use rayon::prelude::*;
use ssh2::Session;
use std::{env, path::Path};
use tokio::net::TcpStream as AsyncTcpStream;

#[derive(Debug, Clone)]
pub struct HostConfig {
    pub ip: String,
    pub port: u16,
    pub username: String,
    pub privkey_path: String,
    pub password: Option<String>,
}

#[derive(Debug)]
pub struct HostCheckResult {
    pub ip: String,
    pub ssh_accessible: bool,
    pub has_root_access: bool,
    pub has_passwordless_sudo: bool,
    pub can_sudo_with_password: bool,
}

impl HostCheckResult {
    pub fn new(ip: String) -> Self {
        HostCheckResult {
            ip,
            ssh_accessible: false,
            has_root_access: false,
            has_passwordless_sudo: false,
            can_sudo_with_password: false,
        }
    }
}

async fn check_single_host_async(host: &HostConfig) -> HostCheckResult {
    let mut result = HostCheckResult::new(host.ip.clone());

    // Check SSH connection
    let conn = match AsyncTcpStream::connect((host.ip.as_str(), host.port)).await {
        Ok(stream) => stream.into_std().unwrap(),
        Err(_) => return result,
    };

    // SSH session setup
    let mut sess = match Session::new() {
        Ok(s) => s,
        Err(_) => return result,
    };
    sess.set_tcp_stream(conn);
    if sess.handshake().is_err() {
        return result;
    }

    // Try public key authentication first
    let auth_ok = sess
        .userauth_pubkey_file(&host.username, None, Path::new(&host.privkey_path), None)
        .is_ok()
        && sess.authenticated();

    // If public key auth failed and password is provided, try password authentication
    if !auth_ok {
        if let Some(password) = &host.password {
            if sess.userauth_password(&host.username, password).is_ok() && sess.authenticated() {
                result.ssh_accessible = true;
            } else {
                return result;
            }
        } else {
            return result;
        }
    } else {
        result.ssh_accessible = true;
    }

    // Root user has full access
    if host.username == "root" {
        result.has_root_access = true;
        result.has_passwordless_sudo = true;
        return result;
    }

    // Check sudo privileges
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

    let command = format!("echo '{password}' | sudo -S true 2>/dev/null");

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
            privkey_path: format!("{home}/.ssh/id_rsa"),
            password: None,
        },
        HostConfig {
            ip: "ubuntu".to_string(),
            port: 22,
            username: "mahongqin".to_string(),
            privkey_path: format!("{home}/.ssh/id_rsa"),
            password: None,
        },
        HostConfig {
            ip: "ubuntu".to_string(),
            port: 22,
            username: "mahongqin".to_string(),
            privkey_path: format!("{home}/.ssh/id_rsa"),
            password: Some("ma@4056".to_string()),
        },
        HostConfig {
            ip: "ubuntu".to_string(),
            port: 22,
            username: "regular_user".to_string(),
            privkey_path: format!("{home}/.ssh/id_rsa"),
            password: None,
        },
    ];

    let results = bulk_check_hosts(hosts);
    for result in results {
        println!("{result:#?}");
        println!(
            "{} - SSH: {}, Root: {}, Passwordless Sudo: {}, Sudo with Password: {}",
            result.ip,
            if result.ssh_accessible { "✅" } else { "❌" },
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
