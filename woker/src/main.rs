use clap::Parser;
use log::{LevelFilter, error, info};
use simple_logger::SimpleLogger;
use woker::{
    client,
    command::{Cli, handle_command},
};

type Error = Box<dyn std::error::Error + Send + Sync>;
type Result<T> = std::result::Result<T, Error>;

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
