use rayon::prelude::*;
use ssh2::Session;
use std::{path::Path, time::Duration};
use tokio::{net::TcpStream as AsyncTcpStream, runtime::Runtime, time::timeout};

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
    let connect_timeout = Duration::from_secs(3); // 3秒连接超时

    // 1. 带超时的TCP连接
    let conn = match timeout(
        connect_timeout,
        AsyncTcpStream::connect((host.ip.as_str(), host.port)),
    )
    .await
    {
        Ok(Ok(stream)) => stream.into_std().unwrap(),
        Ok(Err(_)) | Err(_) => return result, // 连接失败或超时
    };

    // 2. SSH会话设置
    let mut sess = match Session::new() {
        Ok(s) => s,
        Err(_) => return result,
    };

    sess.set_tcp_stream(conn);

    // 3. 带超时的握手
    let handshake_timeout = Duration::from_secs(3);
    if timeout(handshake_timeout, async { sess.handshake() })
        .await
        .is_err()
    {
        return result;
    }

    // 剩余认证检查逻辑保持不变...
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
    hosts
        .par_iter()
        .map(|host| {
            let rt = Runtime::new().unwrap();
            rt.block_on(async {
                let timeout_duration = Duration::from_secs(5); // 总超时5秒
                timeout(timeout_duration, check_single_host_async(host))
                    .await
                    .unwrap_or_else(|_| {
                        HostCheckResult::new(host.ip.clone(), host.username.clone())
                    })
            })
        })
        .collect()
}
