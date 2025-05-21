use clap::{Parser, Subcommand};
use log::{LevelFilter, error, info, warn};
use simple_logger::SimpleLogger;
use woker::client;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Result<T> = std::result::Result<T, Error>;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Init {
        #[arg(long)]
        master: Vec<String>,
        #[arg(long, action = clap::ArgAction::Append)]
        node: Vec<String>,
    },
}

async fn handle_command(command: Commands) -> Result<()> {
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

async fn run_default_service() -> Result<()> {
    info!("Starting agent client service...");
    let endpoint = "http://localhost:50051";
    let agent_id = "rust-agent-001";

    info!("Connecting to default endpoint: {endpoint}");
    let client = client::AgentClient::new(endpoint, agent_id);

    match client.run().await {
        Ok(_) => info!("Server closed stream"),
        Err(e) => error!("Error running client: {e}"),
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init()
        .unwrap();

    let cli = Cli::parse();

    if let Some(command) = cli.command {
        handle_command(command).await?;
    } else {
        run_default_service().await?;
    }

    Ok(())
}

// websocat ws://localhost:8080/api/v1/tunnel/rust-agent-001
// {"payload":{"input":"pwd"},"metadata":{"source":"test"}}

// cargo run -- init --master 192.168.64.2,192.168.64.22,192.168.64.20 --node 192.168.64.21,192.168.64.19
