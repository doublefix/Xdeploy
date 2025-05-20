use rayon::prelude::*;
use ssh2::Session;
use std::{env, fmt::format, path::Path};
use tokio::net::TcpStream as AsyncTcpStream;

#[derive(Debug)]
pub struct HostConfig {
    ip: String,
    port: u16,
    username: String,
    privkey_path: String,
}

async fn check_single_host_async(host: &HostConfig) -> (String, bool) {
    let conn = AsyncTcpStream::connect((host.ip.as_str(), host.port)).await;
    let tcp = match conn {
        Ok(stream) => stream.into_std().unwrap(),
        Err(_) => return (host.ip.clone(), false),
    };

    let mut sess = match Session::new() {
        Ok(s) => s,
        Err(_) => return (host.ip.clone(), false),
    };
    sess.set_tcp_stream(tcp);
    if sess.handshake().is_err() {
        return (host.ip.clone(), false);
    }

    let auth_ok = sess
        .userauth_pubkey_file(&host.username, None, Path::new(&host.privkey_path), None)
        .is_ok()
        && sess.authenticated();

    (host.ip.clone(), auth_ok)
}

pub fn bulk_check_hosts(hosts: Vec<HostConfig>) -> Vec<(String, bool)> {
    let rt = tokio::runtime::Runtime::new().unwrap();

    // 并行处理所有主机
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
            ip: "ubuntu2".to_string(),
            port: 22,
            username: "root".to_string(),
            privkey_path: format!("{home}/.ssh/id_rsa"),
        },
        HostConfig {
            ip: "ubuntu".to_string(),
            port: 22,
            username: "root".to_string(),
            privkey_path: format!("{home}/.ssh/id_rsa"),
        },
        // 添加更多主机...
    ];

    let results = bulk_check_hosts(hosts);
    for (ip, status) in results {
        println!("{}: {}", ip, if status { "✅" } else { "❌" });
    }
}
