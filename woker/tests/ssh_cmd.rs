use std::error::Error;
use woker::ssh_cmd::{
    AuthMethod, SshConfig, build_std_linux_tar_zxvf_commands, run_commands_on_multiple_hosts,
};

#[tokio::test]
async fn test_main() -> Result<(), Box<dyn Error>> {
    let home = std::env::var("HOME")?;
    let images = &["ee65adc925d6d5acd33beeba4747f90fda68bec1dbea6a1dea16691fe9fdfeb8"];
    let commands = build_std_linux_tar_zxvf_commands(images);

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

    let results = run_commands_on_multiple_hosts(configs, commands).await;

    for (host, result) in results {
        match result {
            Ok(outputs) => {
                println!("✅ [{host}] File operations completed:");
                for output in outputs {
                    println!("{output}");
                }
            }
            Err(e) => eprintln!("❌ [{host}] Error: {e}"),
        }
    }

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

    let results = run_commands_on_multiple_hosts(configs, commands).await;

    for (host, result) in results {
        match result {
            Ok(outputs) => {
                println!("✅ [{host}] File operations completed:");
                for output in outputs {
                    println!("{output}");
                }
            }
            Err(e) => eprintln!("❌ [{host}] Error: {e}"),
        }
    }

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
    let results = run_commands_on_multiple_hosts(configs, commands).await;

    for (host, result) in results {
        match result {
            Ok(outputs) => {
                println!("✅ [{host}] Success:");
                for (i, output) in outputs.iter().enumerate() {
                    println!("Command {} output:\n{}", i + 1, output);
                }
            }
            Err(e) => eprintln!("❌ [{host}] Error: {e}"),
        }
    }

    Ok(())
}
