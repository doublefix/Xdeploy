use std::{collections::HashMap, env};

use crate::ssh_cmd::{
    self, KubeJoinInfo, build_std_linux_init_node_commands, run_commands_on_multiple_hosts,
};

type Error = Box<dyn std::error::Error + Send + Sync>;
type Result<T> = std::result::Result<T, Error>;

pub async fn init_root_node(root: Vec<String>, images_sha256: Vec<String>) -> Result<KubeJoinInfo> {
    let home = env::var("HOME").unwrap();
    let run_root_cmd_configs: Vec<ssh_cmd::SshConfig> = root
        .clone()
        .into_iter()
        .map(|host| ssh_cmd::SshConfig {
            host: host.to_string(),
            port: 22,
            username: "root".to_string(),
            auth: ssh_cmd::AuthMethod::Key {
                pubkey_path: format!("{home}/.ssh/id_rsa.pub"),
                privkey_path: format!("{home}/.ssh/id_rsa"),
                passphrase: None,
            },
        })
        .collect();
    let mut root_env_vars = HashMap::new();
    root_env_vars.insert("NODE_ROLE", "root");
    let commands = build_std_linux_init_node_commands(&root_env_vars, &images_sha256);
    let _ = run_commands_on_multiple_hosts(run_root_cmd_configs.clone(), commands, true).await;

    // Get join key information
    let ssh_client = ssh_cmd::SshClient::new(run_root_cmd_configs[0].clone());
    let join_key = ssh_client.get_kube_join_info().await?;

    Ok(join_key)
}

pub async fn init_master_node(
    plane: Vec<String>,
    images_sha256: Vec<String>,
    api: &str,
    token: &str,
    hash: &str,
) {
    let home = env::var("HOME").unwrap();
    let mut mater_env_vars = HashMap::new();
    mater_env_vars.insert("NODE_ROLE", "master");
    mater_env_vars.insert("KUBE_API_SERVER", api);
    mater_env_vars.insert("KUBE_JOIN_TOKEN", token);
    mater_env_vars.insert("KUBE_CA_CERT_HASH", hash);
    let run_master_cmd_configs: Vec<ssh_cmd::SshConfig> = plane
        .clone()
        .into_iter()
        .map(|host| ssh_cmd::SshConfig {
            host: host.to_string(),
            port: 22,
            username: "root".to_string(),
            auth: ssh_cmd::AuthMethod::Key {
                pubkey_path: format!("{home}/.ssh/id_rsa.pub"),
                privkey_path: format!("{home}/.ssh/id_rsa"),
                passphrase: None,
            },
        })
        .collect();
    let commands = build_std_linux_init_node_commands(&mater_env_vars, &images_sha256);
    let _ = run_commands_on_multiple_hosts(run_master_cmd_configs, commands, true).await;
}

pub async fn init_woker_node(
    nodes: Vec<String>,
    images_sha256: Vec<String>,
    api: &str,
    token: &str,
    hash: &str,
) {
    let home = env::var("HOME").unwrap();
    let mut node_env_vars = HashMap::new();
    node_env_vars.insert("NODE_ROLE", "node");
    node_env_vars.insert("KUBE_API_SERVER", api);
    node_env_vars.insert("KUBE_JOIN_TOKEN", token);
    node_env_vars.insert("KUBE_CA_CERT_HASH", hash);
    let run_node_cmd_configs: Vec<ssh_cmd::SshConfig> = nodes
        .clone()
        .into_iter()
        .map(|host| ssh_cmd::SshConfig {
            host: host.to_string(),
            port: 22,
            username: "root".to_string(),
            auth: ssh_cmd::AuthMethod::Key {
                pubkey_path: format!("{home}/.ssh/id_rsa.pub"),
                privkey_path: format!("{home}/.ssh/id_rsa"),
                passphrase: None,
            },
        })
        .collect();
    let commands = build_std_linux_init_node_commands(&node_env_vars, &images_sha256);
    let _ = run_commands_on_multiple_hosts(run_node_cmd_configs, commands, true).await;
}
