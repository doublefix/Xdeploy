use clap::{Parser, Subcommand};
use log::{info, warn};

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
