use woker::client;
type Error = Box<dyn std::error::Error + Send + Sync>;
type Result<T> = std::result::Result<T, Error>;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Starting agent client...");
    let endpoint = "http://localhost:50051";
    let agent_id = "rust-agent-001";

    let client = client::AgentClient::new(endpoint, agent_id);
    client.run().await?;

    println!("Server closed stream");
    Ok(())
}

// websocat ws://localhost:8080/api/v1/tunnel/rust-agent-001
// {"payload":{"input":"pwd"},"metadata":{"source":"test"}}
