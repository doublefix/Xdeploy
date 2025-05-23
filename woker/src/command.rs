use std::{collections::HashMap, env, path::Path};

use clap::{Parser, Subcommand};
use log::{info, warn};

use crate::{
    load_image::load_image,
    sftp::{AuthMethod, SshConfig, concurrent_upload_folders},
    ssh_cmd::{
        self, build_std_linux_init_node_commands, build_std_linux_tarzxvf_filetoroot_commands,
        run_commands_on_multiple_hosts,
    },
    ssh_connect::{HostConfig, bulk_check_hosts},
};

type Error = Box<dyn std::error::Error + Send + Sync>;
type Result<T> = std::result::Result<T, Error>;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    Run {
        #[arg(required = true)]
        images: Vec<String>,
        #[arg(long)]
        master: Vec<String>,
        #[arg(long, action = clap::ArgAction::Append)]
        node: Vec<String>,
    },
}

pub async fn handle_command(command: Commands) -> Result<()> {
    match command {
        Commands::Run {
            images,
            master,
            node,
        } => {
            let masters: Vec<String> = master
                .iter()
                .flat_map(|s| s.split(','))
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();

            let nodes: Vec<String> = node
                .iter()
                .flat_map(|s| s.split(','))
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();

            info!("All master addresses: {masters:?}");
            info!("All node addresses: {nodes:?}");

            // 合并所有地址并检查重复
            let all_addresses: Vec<_> = masters.iter().chain(nodes.iter()).collect();
            if let Some(dup) = find_first_duplicate(&all_addresses) {
                warn!("Duplicate address found: {dup}");
                return Ok(());
            }

            // 执行批量检查可达性
            let home = env::var("HOME").unwrap();
            let hosts: Vec<HostConfig> = all_addresses
                .clone()
                .into_iter()
                .map(|addr| HostConfig {
                    ip: addr.to_string(),
                    port: 22,
                    username: "root".to_string(),
                    privkey_path: Some(format!("{home}/.ssh/id_rsa")),
                    password: None,
                })
                .collect();

            info!("HOSTS: {hosts:?}");
            let results = bulk_check_hosts(hosts).await;
            for result in results {
                if !result.ssh_accessible || !result.has_root_access {
                    warn!(
                        "Host check failed for {}: SSH accessible: {}, Root access: {}",
                        result.ip, result.ssh_accessible, result.has_root_access
                    );
                    return Ok(());
                }
            }

            // 加载镜像
            let images_sha256 = load_image(images, None).await?;
            info!("{images_sha256:?}");
            // 传输文件
            let sftp_configs: Vec<SshConfig> = all_addresses
                .clone()
                .into_iter()
                .map(|addr| SshConfig {
                    host: addr.to_string(),
                    port: 22,
                    username: "root".to_string(),
                    auth: AuthMethod::Key {
                        pubkey: format!("{home}/.ssh/id_rsa.pub"),
                        privkey: format!("{home}/.ssh/id_rsa"),
                        passphrase: None,
                    },
                })
                .collect();
            let local_base = Path::new("/var/tmp/chess");
            let remote_base = Path::new("/tmp/.chess");
            let _ = concurrent_upload_folders(
                sftp_configs,
                images_sha256.clone(),
                local_base,
                remote_base,
            )
            .await;

            // 所有节点
            let commands = build_std_linux_tarzxvf_filetoroot_commands(&images_sha256);
            let run_cmd_configs: Vec<ssh_cmd::SshConfig> = all_addresses
                .into_iter()
                .map(|host| ssh_cmd::SshConfig {
                    host: host.to_string(),
                    port: 22,
                    username: "root".to_string(),
                    auth: ssh_cmd::AuthMethod::Key {
                        pubkey_path: format!("{home}/.ssh/id_rsa.pub"),
                        privkey_path: format!("{home}/.ssh/id_rsa"),
                        passphrase: None,
                    },
                })
                .collect();
            let _ = run_commands_on_multiple_hosts(run_cmd_configs, commands, false).await;

            // 主节点
            for (i, master_addr) in masters.iter().enumerate() {
                info!("Configuring master {}: {}", i + 1, master_addr);
            }
            let run_master_cmd_configs: Vec<ssh_cmd::SshConfig> = masters
                .clone()
                .into_iter()
                .map(|host| ssh_cmd::SshConfig {
                    host: host.to_string(),
                    port: 22,
                    username: "root".to_string(),
                    auth: ssh_cmd::AuthMethod::Key {
                        pubkey_path: format!("{home}/.ssh/id_rsa.pub"),
                        privkey_path: format!("{home}/.ssh/id_rsa"),
                        passphrase: None,
                    },
                })
                .collect();
            let mut mater_env_vars = HashMap::new();
            mater_env_vars.insert("NODE_ROLE", "master");
            let commands = build_std_linux_init_node_commands(&mater_env_vars);
            let _ = run_commands_on_multiple_hosts(run_master_cmd_configs, commands, true).await;

            // 工作节点
            for (i, node_addr) in nodes.iter().enumerate() {
                info!("Configuring node {}: {}", i + 1, node_addr);
            }
            let mut node_env_vars = HashMap::new();
            node_env_vars.insert("NODE_ROLE", "node");
            let run_node_cmd_configs: Vec<ssh_cmd::SshConfig> = nodes
                .clone()
                .into_iter()
                .map(|host| ssh_cmd::SshConfig {
                    host: host.to_string(),
                    port: 22,
                    username: "root".to_string(),
                    auth: ssh_cmd::AuthMethod::Key {
                        pubkey_path: format!("{home}/.ssh/id_rsa.pub"),
                        privkey_path: format!("{home}/.ssh/id_rsa"),
                        passphrase: None,
                    },
                })
                .collect();
            let commands = build_std_linux_init_node_commands(&node_env_vars);
            let _ = run_commands_on_multiple_hosts(run_node_cmd_configs, commands, true).await;

            info!("Initialization completed successfully");
            Ok(())
        }
    }
}

// 找到第一个重复元素
fn find_first_duplicate<'a, T: Eq + std::hash::Hash>(items: &'a [&'a T]) -> Option<&'a T> {
    let mut seen = std::collections::HashSet::new();
    items.iter().find(|&&item| !seen.insert(item)).copied()
}
