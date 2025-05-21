use std::env;

use woker::ssh_connect::{HostConfig, bulk_check_hosts};

#[tokio::test]
async fn test_bulk_check() {
    let home = env::var("HOME").unwrap();
    let hosts = vec![
        HostConfig {
            ip: "47.76.42.207".to_string(),
            port: 22,
            username: "root".to_string(),
            privkey_path: Some(format!("{home}/.ssh/id_rsa")),
            password: None,
        },
        // HostConfig {
        //     ip: "192.168.8.8".to_string(),
        //     port: 22,
        //     username: "mahongqin".to_string(),
        //     privkey_path: Some(format!("{home}/.ssh/id_rsa")),
        //     password: Some("ma@4056".to_string()),
        // },
        // HostConfig {
        //     ip: "localhost".to_string(),
        //     port: 22,
        //     username: "alice".to_string(),
        //     privkey_path: None,
        //     password: Some("123456".to_string()),
        // },
    ];

    let results = bulk_check_hosts(hosts).await;
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
