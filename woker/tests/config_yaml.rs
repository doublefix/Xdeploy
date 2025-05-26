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
    pub hosts: Vec<Host>,
    pub image: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Host {
    pub roles: Vec<String>,
    pub ips: Vec<String>,
}

impl Cluster {
    /// Get the file path for this cluster's YAML file
    fn get_file_path(&self) -> PathBuf {
        let home_dir = dirs::home_dir().expect("Could not find home directory");
        let mut path = home_dir.join(".test").join(&self.metadata.name);
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
        let path = home_dir.join(".test").join(name).join("cluster.yaml");

        let mut file = File::open(&path).await?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).await?;

        let cluster: Cluster = serde_yaml::from_str(&contents)?;
        Ok(cluster)
    }

    /// Asynchronously update the cluster YAML file
    pub async fn update_file<F>(name: &str, update_fn: F) -> Result<(), Box<dyn std::error::Error>>
    where
        F: FnOnce(&mut Cluster),
    {
        let mut cluster = Cluster::load_from_file(name).await?;
        update_fn(&mut cluster);
        cluster.save_to_file().await?;
        Ok(())
    }
}

#[tokio::test]
async fn test_get_() -> Result<(), Box<dyn std::error::Error>> {
    // Example usage

    // Create and save a new cluster
    let cluster = Cluster {
        api_version: "v1".to_string(),
        kind: "Cluster".to_string(),
        metadata: Metadata {
            name: "my-cluster".to_string(),
        },
        spec: Spec {
            hosts: vec![Host {
                roles: vec!["master".to_string()],
                ips: vec!["192.168.1.1".to_string()],
            }],
            image: vec!["ubuntu:latest".to_string()],
        },
    };

    cluster.save_to_file().await?;
    println!("Cluster saved successfully");

    // Load the cluster
    let loaded_cluster = Cluster::load_from_file("my-cluster").await?;
    println!("Loaded cluster: {loaded_cluster:?}");

    // Update the cluster
    Cluster::update_file("my-cluster", |cluster| {
        cluster.spec.hosts.push(Host {
            roles: vec!["worker".to_string()],
            ips: vec!["192.168.1.2".to_string()],
        });
    })
    .await?;

    println!("Cluster updated successfully");

    Ok(())
}
