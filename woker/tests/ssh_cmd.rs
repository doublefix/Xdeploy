use std::error::Error;
use woker::ssh_cmd::{
    AuthMethod, SshClient, SshConfig, build_std_linux_tarzxvf_filetoroot_commands,
    run_commands_on_multiple_hosts,
};

#[tokio::test]
async fn test_main() -> Result<(), Box<dyn Error>> {
    let home = std::env::var("HOME")?;
    let images = &["ee65adc925d6d5acd33beeba4747f90fda68bec1dbea6a1dea16691fe9fdfeb8".to_string()];
    let commands = build_std_linux_tarzxvf_filetoroot_commands(images);

    let hosts = vec!["47.76.42.207"];
    let configs: Vec<SshConfig> = hosts
        .into_iter()
        .map(|host| SshConfig {
            host: host.to_string(),
            port: 22,
            username: "root".to_string(),
            auth: AuthMethod::Key {
                pubkey_path: format!("{home}/.ssh/id_rsa.pub"),
                privkey_path: format!("{home}/.ssh/id_rsa"),
                passphrase: None,
            },
        })
        .collect();

    run_commands_on_multiple_hosts(configs, commands, true).await;

    Ok(())
}

#[tokio::test]
async fn test_file_operations() -> Result<(), Box<dyn Error>> {
    let home = std::env::var("HOME")?;

    // Define file operations to execute
    let image_id = "ee65adc925d6d5acd33beeba4747f90fda68bec1dbea6a1dea16691fe9fdfeb8";
    let commands = vec![
        format!("mkdir -p /tmp/.chess/{image_id}/test1"),
        format!("mkdir -p /tmp/.chess/{image_id}/test2"),
        format!(
            "tar -zxvf /tmp/.chess/{image_id}/chess-package1.tar.gz -C /tmp/.chess/{image_id}/test1"
        ),
        format!(
            "tar -zxvf /tmp/.chess/{image_id}/chess-package2.tar.gz -C /tmp/.chess/{image_id}/test2"
        ),
        format!("ls -l /tmp/.chess/{image_id}/test1"),
        format!("ls -l /tmp/.chess/{image_id}/test2"),
    ];

    let hosts = vec!["ubuntu"];
    let configs: Vec<SshConfig> = hosts
        .into_iter()
        .map(|host| SshConfig {
            host: host.to_string(),
            port: 22,
            username: "root".to_string(),
            auth: AuthMethod::Key {
                pubkey_path: format!("{home}/.ssh/id_rsa.pub"),
                privkey_path: format!("{home}/.ssh/id_rsa"),
                passphrase: None,
            },
        })
        .collect();

    run_commands_on_multiple_hosts(configs, commands, true).await;

    Ok(())
}

#[tokio::test]
async fn test_multi_command_execution() -> Result<(), Box<dyn Error>> {
    let home = std::env::var("HOME")?;

    // Define multiple commands to execute
    let commands = vec![
        "echo 'Starting operations...'".to_string(),
        "df -h".to_string(),
        "uname -a".to_string(),
    ];

    // Define multiple hosts
    let hosts = vec!["ubuntu"];
    let configs: Vec<SshConfig> = hosts
        .into_iter()
        .map(|host| SshConfig {
            host: host.to_string(),
            port: 22,
            username: "root".to_string(),
            auth: AuthMethod::Key {
                pubkey_path: format!("{home}/.ssh/id_rsa.pub"),
                privkey_path: format!("{home}/.ssh/id_rsa"),
                passphrase: None,
            },
        })
        .collect();

    // Execute multiple commands on multiple hosts
    run_commands_on_multiple_hosts(configs, commands, true).await;

    Ok(())
}

#[tokio::test]
async fn test_get_join_info() -> Result<(), Box<dyn Error>> {
    let home = std::env::var("HOME")?;

    let config = SshConfig {
        host: "47.76.42.207".to_string(),
        port: 22,
        username: "root".to_string(),
        auth: AuthMethod::Key {
            pubkey_path: format!("{home}/.ssh/id_rsa.pub"),
            privkey_path: format!("{home}/.ssh/id_rsa"),
            passphrase: None,
        },
    };
    let ssh_client = SshClient::new(config);
    match ssh_client.get_kube_join_info().await {
        Ok(info) => {
            println!("Kubernetes join information:");
            println!("API Server: {:?}", info.kube_api_server);
            println!("Token: {:?}", info.kube_join_token);
            println!("CA Cert Hash: {:?}", info.kube_ca_cert_hash);
        }
        Err(e) => eprintln!("Failed to get join info: {e}"),
    }

    Ok(())
}
