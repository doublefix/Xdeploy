use std::{
    fs, io,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use clap::{Parser, Subcommand};

use log::{info, warn};

use crate::{
    cluster_config::{
        Cluster, Metadata, Servers, Spec, get_active_cluster, get_active_cluster_config,
        list_cluster_names, switch_cluster,
    },
    cluster_images::{load_image_to_server, tarzxf_remote_server_package},
    cluster_node::{
        init_cluster_master_node, init_cluster_root_node, init_cluster_woker_node, init_common_node,
    },
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
        #[arg(long)]
        cluster: Option<String>,
    },
    Ps {},
    Use {
        #[arg(required = true, help = "Name of the cluster to switch to")]
        clustername: String,
    },
}

pub async fn handle_command(command: Commands) -> Result<()> {
    match command {
        Commands::Run {
            images,
            master,
            node,
            cluster,
        } => {
            let cluster_name = if let Some(c) = cluster {
                info!("Current cluster name: {c}");
                c
            } else {
                let current_cluster_name = get_active_cluster()?;
                info!("Current cluster name: {current_cluster_name}");
                get_active_cluster_config()?
            };

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

            let cluster = Cluster {
                api_version: "chess.io/v1".to_string(),
                kind: "Cluster".to_string(),
                metadata: Metadata {
                    name: cluster_name.clone(),
                },
                spec: Spec {
                    servers: vec![
                        Servers {
                            roles: vec!["master".to_string()],
                            ips: masters,
                        },
                        Servers {
                            roles: vec!["node".to_string()],
                            ips: nodes,
                        },
                    ],
                    images: images.clone(),
                },
            };

            let has_master = cluster
                .spec
                .servers
                .iter()
                .any(|server| server.roles.iter().any(|role| role == "master"));

            let has_woker = cluster
                .spec
                .servers
                .iter()
                .any(|server| server.roles.iter().any(|role| role == "node"));

            // 指定集群, 指定节点
            if has_master || has_woker {
                info!("Initializing common images {images:?}");
                init_cluster(&cluster).await?;

                if has_master {
                    load_cluster_config(&cluster).await?;
                }
            }

            if !has_master && !has_woker {
                info!("No master or worker nodes specified.");
            }

            info!("Initialization completed successfully");
            Ok(())
        }
        Commands::Ps {} => {
            let clusters: Vec<String> = list_cluster_names()?;
            let active_cluster: String = get_active_cluster()?;

            if clusters.is_empty() {
                println!("No clusters found.");
                return Ok(());
            }

            let max_name_len = clusters
                .iter()
                .map(|s| s.len())
                .max()
                .unwrap_or(0)
                .clamp(12, 50);

            println!(
                "{:<10} {:<width$}",
                "CURRENT",
                "CLUSTE RNAME",
                width = max_name_len
            );

            for cluster in &clusters {
                let current_mark = if cluster == &active_cluster { "*" } else { " " };
                println!("{current_mark:<10} {cluster:<max_name_len$}");
            }

            Ok(())
        }
        Commands::Use { clustername } => {
            let _ = switch_cluster(&clustername);
            Ok(())
        }
    }
}

async fn load_cluster_config(cluster: &Cluster) -> Result<()> {
    let cluster_name = cluster.metadata.name.clone();
    let home_dir = dirs::home_dir().expect("Could not find home directory");
    let config_path = home_dir
        .join(".chess")
        .join(&cluster_name)
        .join("cluster.yaml");

    if config_path.exists() {
        let current_cluster_config = Cluster::load_from_file(&cluster_name).await?;
        if !current_cluster_config.is_same_configuration(cluster) {
            if let Some(backup_path) = backup_cluster_config(&home_dir, &cluster_name)? {
                info!("Backed up existing cluster.yaml to {backup_path:?}");
            }
            let _ = cluster.save_to_file().await;
        }
    } else {
        let _ = cluster.save_to_file().await;
    }

    Ok(())
}

// Load common images

fn backup_cluster_config(home_dir: &Path, cluster_name: &String) -> io::Result<Option<PathBuf>> {
    let chess_dir = home_dir.join(".chess").join(cluster_name);
    let config_file = chess_dir.join("cluster.yaml");

    if !config_file.exists() {
        return Ok(None);
    }

    let history_dir = chess_dir.join(".history");
    if !history_dir.exists() {
        fs::create_dir_all(&history_dir)?;
    }

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(io::Error::other)?
        .as_secs();

    let backup_filename = format!("cluster.yaml.{timestamp}");
    let backup_path = history_dir.join(backup_filename);

    fs::copy(&config_file, &backup_path)?;

    Ok(Some(backup_path))
}

// 找到第一个重复元素
fn find_first_duplicate<'a, T: Eq + std::hash::Hash>(items: &'a [&'a T]) -> Option<&'a T> {
    let mut seen = std::collections::HashSet::new();
    items.iter().find(|&&item| !seen.insert(item)).copied()
}

async fn init_cluster(cluster: &Cluster) -> Result<()> {
    let masters: Vec<&Servers> = cluster
        .spec
        .servers
        .iter()
        .filter(|s| s.roles.contains(&"master".to_string()))
        .collect();

    let nodes: Vec<&Servers> = cluster
        .spec
        .servers
        .iter()
        .filter(|s| s.roles.contains(&"node".to_string()))
        .collect();

    let all_servers: Vec<&Servers> = cluster
        .spec
        .servers
        .iter()
        .filter(|s| {
            s.roles.contains(&"master".to_string()) || s.roles.contains(&"node".to_string())
        })
        .collect();

    info!("All master addresses: {masters:?}");
    info!("All node addresses: {nodes:?}");

    // 合并所有地址并检查重复
    let all_ips: Vec<&String> = masters
        .iter()
        .flat_map(|server| &server.ips)
        .chain(nodes.iter().flat_map(|server| &server.ips))
        .collect();

    if let Some(dup) = find_first_duplicate(&all_ips) {
        warn!("Duplicate IP address found: {dup}");
        return Ok(());
    }

    // 执行批量检查可达性
    let home = std::env::var("HOME").unwrap();
    let hosts: Vec<HostConfig> = all_ips
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

    // 传输镜像、解压二进制制品
    let servers: Vec<String> = all_ips.iter().cloned().cloned().collect();
    let images_sha256 = load_image_to_server(cluster.spec.images.clone(), servers.clone()).await?;
    tarzxf_remote_server_package(images_sha256.clone(), servers).await;
    info!("Images loaded and prepared: {images_sha256:?}");

    // 所有节点执行公共部分
    init_common_node(all_servers, images_sha256.clone()).await?;

    // 主从架构需要分组执行
    let has_master = masters.iter().any(|s| !s.ips.is_empty());
    if has_master {
        init_distributed_cluster(masters.clone(), nodes.clone(), images_sha256.clone()).await?;
    }

    Ok(())
}

async fn init_distributed_cluster(
    masters: Vec<&Servers>,
    nodes: Vec<&Servers>,
    images_sha256: Vec<String>,
) -> Result<()> {
    let (root, plane) = if !masters.is_empty() {
        let root = vec![masters[0]];
        let plane = masters.iter().skip(1).cloned().collect::<Vec<_>>();
        (root, plane)
    } else {
        (Vec::new(), Vec::new())
    };
    // root节点
    let join_root_key = init_cluster_root_node(root.clone(), images_sha256.clone()).await?;

    match (
        &join_root_key.kube_api_server,
        &join_root_key.kube_join_token,
        &join_root_key.kube_ca_cert_hash,
    ) {
        (Some(api), Some(token), Some(hash)) => {
            info!("Kube join key information found: API: {api}, Token: {token}, Hash: {hash}");
            init_cluster_master_node(plane.clone(), images_sha256.clone(), api, token, hash).await;
            init_cluster_woker_node(nodes, images_sha256.clone(), api, token, hash).await;
        }
        _ => info!("There is no kube join key information available"),
    }
    Ok(())
}
