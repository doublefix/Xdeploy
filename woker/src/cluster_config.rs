use dirs::home_dir;
use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::{
    fs::{self, File},
    io::Write,
};
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
            tokio::fs::create_dir_all(parent).await?;
        }
        let yaml = serde_yaml::to_string(self)?;

        let mut file = tokio::fs::File::create(&path).await?;
        file.write_all(yaml.as_bytes()).await?;

        Ok(())
    }

    /// Asynchronously load the cluster from a YAML file
    pub async fn load_from_file(name: &str) -> Result<Cluster, Box<dyn std::error::Error>> {
        let home_dir = dirs::home_dir().expect("Could not find home directory");
        let path = home_dir.join(".chess").join(name).join("cluster.yaml");

        let mut file = tokio::fs::File::open(&path).await?;
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

const DEFAULT_CLUSTER: &str = "default";
const CONFIG_DIR: &str = ".chess";
const ACTIVE_FILE: &str = ".active";

pub fn get_active_cluster_config() -> std::io::Result<String> {
    let home_dir = home_dir().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Could not find home directory",
        )
    })?;

    let config_dir = home_dir.join(".chess");
    let active_file_path = config_dir.join(".active");

    debug!("Config directory: {config_dir:?}");

    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)?;
    }

    if !active_file_path.exists() {
        debug!("No active cluster file found, creating default: {active_file_path:?}");
        let mut file = File::create(&active_file_path)?;
        file.write_all(DEFAULT_CLUSTER.as_bytes())?;
        return Ok(DEFAULT_CLUSTER.to_string());
    }

    let content = fs::read_to_string(&active_file_path)?.trim().to_string();

    if content.is_empty() {
        let mut file = File::create(&active_file_path)?;
        file.write_all(DEFAULT_CLUSTER.as_bytes())?;
        return Ok(DEFAULT_CLUSTER.to_string());
    }

    debug!("Active cluster file found: {active_file_path:?}, content: {content}");
    Ok(content)
}

pub fn list_cluster_names() -> std::io::Result<Vec<String>> {
    let config_dir = get_config_dir()?;

    let entries = fs::read_dir(config_dir)?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            if entry.file_type().ok()?.is_dir() {
                Some(entry.file_name().to_string_lossy().into_owned())
            } else {
                None
            }
        })
        .collect();

    Ok(entries)
}

pub fn get_active_cluster() -> std::io::Result<String> {
    let active_file = get_config_dir()?.join(ACTIVE_FILE);

    if !active_file.exists() {
        fs::write(&active_file, DEFAULT_CLUSTER)?;
        return Ok(DEFAULT_CLUSTER.to_string());
    }

    let content = fs::read_to_string(&active_file)?;
    let content = content.trim();

    if content.is_empty() {
        fs::write(&active_file, DEFAULT_CLUSTER)?;
        Ok(DEFAULT_CLUSTER.to_string())
    } else {
        Ok(content.to_string())
    }
}

fn get_config_dir() -> std::io::Result<PathBuf> {
    let home_dir = dirs::home_dir().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Could not find home directory",
        )
    })?;

    let config_dir = home_dir.join(CONFIG_DIR);
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)?;
    }

    Ok(config_dir)
}

pub fn switch_cluster(target_cluster: &str) -> std::io::Result<()> {
    let valid_clusters = list_cluster_names()?;
    if !valid_clusters.contains(&target_cluster.to_string()) {
        info!("Cluster {target_cluster} does not exist.");
    }

    let active_file = get_config_dir()?.join(ACTIVE_FILE);
    fs::write(active_file, target_cluster)?;

    Ok(())
}
