use std::env;

use clap::{Parser, Subcommand};
use log::{info, warn};

use crate::ssh_connect::{HostConfig, bulk_check_hosts};

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
    Init {
        #[arg(long)]
        master: Vec<String>,
        #[arg(long, action = clap::ArgAction::Append)]
        node: Vec<String>,
    },
}

pub async fn handle_command(command: Commands) -> Result<()> {
    match command {
        Commands::Init { master, node } => {
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

            // 构建 hosts 配置进行 bulk_check
            let home = env::var("HOME").unwrap();
            let hosts: Vec<HostConfig> = all_addresses
                .into_iter()
                .map(|addr| HostConfig {
                    ip: addr.to_string(),
                    port: 22,
                    username: "root".to_string(),
                    privkey_path: Some(format!("{home}/.ssh/id_rsa")),
                    password: None,
                })
                .collect();

            info!("{hosts:?}");
            // 执行批量检查
            let results = bulk_check_hosts(hosts);

            // 检查每个结果
            for result in results {
                if !result.ssh_accessible || !result.has_root_access {
                    warn!(
                        "Host check failed for {}: SSH accessible: {}, Root access: {}",
                        result.ip, result.ssh_accessible, result.has_root_access
                    );
                    return Ok(());
                }
            }

            // 处理每个节点
            for (i, node_addr) in nodes.iter().enumerate() {
                info!("Configuring node {}: {}", i + 1, node_addr);
                // 实际的节点配置逻辑
            }

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
