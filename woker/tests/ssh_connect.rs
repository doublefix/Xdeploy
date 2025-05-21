use rayon::prelude::*;
use ssh2::Session;
use std::{env, path::Path};
use tokio::net::TcpStream as AsyncTcpStream;

#[derive(Debug)]
pub struct HostConfig {
    ip: String,
    port: u16,
    username: String,
    privkey_path: String,
}

#[derive(Debug)]
pub struct HostCheckResult {
    pub ip: String,
    pub ssh_accessible: bool,
    pub has_root_access: bool,
    pub has_sudo_privileges: bool,
}

impl HostCheckResult {
    pub fn new(ip: String) -> Self {
        HostCheckResult {
            ip,
            ssh_accessible: false,
            has_root_access: false,
            has_sudo_privileges: false,
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

    // Authentication check
    let auth_ok = sess
        .userauth_pubkey_file(&host.username, None, Path::new(&host.privkey_path), None)
        .is_ok()
        && sess.authenticated();

    if !auth_ok {
        return result;
    }

    // If we get here, SSH is accessible
    result.ssh_accessible = true;

    // Root user has full access
    if host.username == "root" {
        result.has_root_access = true;
        result.has_sudo_privileges = true;
        return result;
    }

    // For non-root users, check sudo privileges
    result.has_sudo_privileges = check_sudo_privileges(&sess);
    result
}

fn check_sudo_privileges(sess: &Session) -> bool {
    let mut channel = match sess.channel_session() {
        Ok(ch) => ch,
        Err(_) => return false,
    };

    let command = "sudo -n true 2>/dev/null || groups | grep -q sudo || sudo -l >/dev/null 2>&1";

    if channel.exec(command).is_err() {
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
        },
        HostConfig {
            ip: "ubuntu".to_string(),
            port: 22,
            username: "mahongqin".to_string(),
            privkey_path: format!("{home}/.ssh/id_rsa"),
        },
    ];

    let results = bulk_check_hosts(hosts);
    for result in results {
        println!("{result:#?}");
        println!(
            "{} - SSH: {}, Root: {}, Sudo: {}",
            result.ip,
            if result.ssh_accessible { "✅" } else { "❌" },
            if result.has_root_access { "✅" } else { "❌" },
            if result.has_sudo_privileges {
                "✅"
            } else {
                "❌"
            },
        );
    }
}
