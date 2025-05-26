use woker::cluster_config::{Cluster, Metadata, Servers, Spec};

#[tokio::test]
async fn test_cluster_operations() -> Result<(), Box<dyn std::error::Error>> {
    // Example usage

    // Create and save a new cluster
    let cluster = Cluster {
        api_version: "v1".to_string(),
        kind: "Cluster".to_string(),
        metadata: Metadata {
            name: "default".to_string(),
        },
        spec: Spec {
            servers: vec![Servers {
                roles: vec!["master".to_string()],
                ips: vec!["192.168.1.1".to_string()],
            }],
            images: vec!["ubuntu:latest".to_string()],
        },
    };

    cluster.save_to_file().await?;
    println!("Cluster saved successfully");

    // Load the cluster
    let loaded_cluster = Cluster::load_from_file("default").await?;
    println!("Loaded cluster: {loaded_cluster:?}");

    // Add a new host
    Cluster::add_host(
        "default",
        Servers {
            roles: vec!["worker".to_string()],
            ips: vec!["192.168.1.2".to_string()],
        },
    )
    .await?;

    println!("Host added successfully");

    // Remove a host
    Cluster::remove_host("default", "192.168.1.1").await?;

    println!("Host removed successfully");

    Ok(())
}
