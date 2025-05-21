use clap::{Parser, Subcommand};
use log::{LevelFilter, error, info};
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
    // Initialize logger
    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init()
        .unwrap();

    let cli = Cli::parse();

    // Process commands if provided
    if let Some(command) = cli.command {
        match command {
            Commands::Init { master, node } => {
                info!("Initializing cluster with master: {master:?}");
                info!("Nodes: {node:?}");

                // Here you would do something with these parameters
                // This is your command line logic, separate from the HTTP client

                // Just a placeholder for actual initialization logic
                for (i, node_addr) in node.iter().enumerate() {
                    info!("Configuring node {}: {}", i + 1, node_addr);
                    // Add your node configuration logic here
                }

                info!("Initialization completed successfully");
            }
        }
    } else {
        // No command provided, run the default service
        run_default_service().await?;
    }

    Ok(())
}

// websocat ws://localhost:8080/api/v1/tunnel/rust-agent-001
// {"payload":{"input":"pwd"},"metadata":{"source":"test"}}

// cargo run -- init --master 192.168.64.2,192.168.64.22,192.168.64.20 --node 192.168.64.21,192.168.64.19
