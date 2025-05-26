use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs::{self, File};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Cluster {
    pub api_version: String,
    pub kind: String,
    pub metadata: Metadata,
    pub spec: Spec,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Metadata {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Spec {
    pub servers: Vec<Servers>,
    pub images: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Servers {
    pub roles: Vec<String>,
    pub ips: Vec<String>,
}

impl Cluster {
    /// Get the file path for this cluster's YAML file
    fn get_file_path(&self) -> PathBuf {
        let home_dir = dirs::home_dir().expect("Could not find home directory");
        let mut path = home_dir.join(".chess").join(&self.metadata.name);
        path.push("cluster.yaml");
        path
    }

    /// Asynchronously save the cluster to a YAML file
    pub async fn save_to_file(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = self.get_file_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }
        let yaml = serde_yaml::to_string(self)?;

        let mut file = File::create(&path).await?;
        file.write_all(yaml.as_bytes()).await?;

        Ok(())
    }

    /// Asynchronously load the cluster from a YAML file
    pub async fn load_from_file(name: &str) -> Result<Cluster, Box<dyn std::error::Error>> {
        let home_dir = dirs::home_dir().expect("Could not find home directory");
        let path = home_dir.join(".chess").join(name).join("cluster.yaml");

        let mut file = File::open(&path).await?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).await?;

        let cluster: Cluster = serde_yaml::from_str(&contents)?;
        Ok(cluster)
    }

    /// Add a new host to the cluster and save it
    pub async fn add_host(name: &str, host: Servers) -> Result<(), Box<dyn std::error::Error>> {
        let mut cluster = Cluster::load_from_file(name).await?;
        cluster.spec.servers.push(host);
        cluster.save_to_file().await?;
        Ok(())
    }

    /// Remove a host from the cluster by IP and save it
    pub async fn remove_host(name: &str, ip: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut cluster = Cluster::load_from_file(name).await?;
        cluster
            .spec
            .servers
            .retain(|host| !host.ips.contains(&ip.to_string()));
        cluster.save_to_file().await?;
        Ok(())
    }
}

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
