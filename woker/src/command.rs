use clap::{Parser, Subcommand};
use log::{info, warn};

use crate::{
    cluster_images::{load_image_to_server, tarzxf_remote_server_package},
    cluster_node::{init_master_node, init_root_node, init_woker_node},
    ssh_connect::{HostConfig, bulk_check_hosts},
};

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
    Run {
        #[arg(required = true)]
        images: Vec<String>,
        #[arg(long)]
        master: Vec<String>,
        #[arg(long, action = clap::ArgAction::Append)]
        node: Vec<String>,
    },
}

pub async fn handle_command(command: Commands) -> Result<()> {
    match command {
        Commands::Run {
            images,
            master,
            node,
        } => {
            if (!master.is_empty() && !node.is_empty()) || !master.is_empty() {
                init_cluster(images.clone(), master, node).await?;
            }
            info!("Initializing common images {images:?}");

            // Load common images

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

async fn init_cluster(images: Vec<String>, master: Vec<String>, node: Vec<String>) -> Result<()> {
    info!("Initializing cluster with images: {images:?}, master: {master:?}, node: {node:?}");
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

    // 执行批量检查可达性
    let home = std::env::var("HOME").unwrap();
    let hosts: Vec<HostConfig> = all_addresses
        .clone()
        .into_iter()
        .map(|addr| HostConfig {
            ip: addr.to_string(),
            port: 22,
            username: "root".to_string(),
            privkey_path: Some(format!("{home}/.ssh/id_rsa")),
            password: None,
        })
        .collect();

    info!("HOSTS: {hosts:?}");
    let results = bulk_check_hosts(hosts).await;
    for result in results {
        if !result.ssh_accessible || !result.has_root_access {
            warn!(
                "Host check failed for {}: SSH accessible: {}, Root access: {}",
                result.ip, result.ssh_accessible, result.has_root_access
            );
            return Ok(());
        }
    }

    let servers: Vec<String> = all_addresses.iter().cloned().cloned().collect();
    let images_sha256 = load_image_to_server(images.clone(), servers.clone()).await?;
    tarzxf_remote_server_package(images_sha256.clone(), servers).await;

    // 主节点分组
    let (root, plane) = if !masters.is_empty() {
        let root = vec![masters[0].clone()];
        let plane = masters.iter().skip(1).cloned().collect::<Vec<_>>();
        (root, plane)
    } else {
        (Vec::new(), Vec::new())
    };
    // root节点
    let join_root_key = init_root_node(root.clone(), images_sha256.clone()).await?;

    match (
        &join_root_key.kube_api_server,
        &join_root_key.kube_join_token,
        &join_root_key.kube_ca_cert_hash,
    ) {
        (Some(api), Some(token), Some(hash)) => {
            // 主节点
            init_master_node(plane.clone(), images_sha256.clone(), api, token, hash).await;
            // 工作节点
            init_woker_node(nodes, images_sha256.clone(), api, token, hash).await;
        }
        _ => println!("There is no kube join key information available"),
    }
    Ok(())
}
