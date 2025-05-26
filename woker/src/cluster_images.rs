use std::{env, path::Path};

use log::info;

use crate::{
    load_image::load_image,
    sftp::{AuthMethod, SshConfig, concurrent_upload_folders},
    ssh_cmd::{self, build_std_linux_tarzxvf_filetoroot_commands, run_commands_on_multiple_hosts},
};

type Error = Box<dyn std::error::Error + Send + Sync>;
type Result<T> = std::result::Result<T, Error>;

pub async fn load_image_to_server(
    images: Vec<String>,
    servers: Vec<String>,
) -> Result<Vec<String>> {
    let home = env::var("HOME").unwrap();
    info!("Loading images to server: {servers:?}");
    let images_sha256 = load_image(images, None).await?;
    info!("{images_sha256:?}");

    let sftp_configs: Vec<SshConfig> = servers
        .clone()
        .into_iter()
        .map(|addr| SshConfig {
            host: addr.to_string(),
            port: 22,
            username: "root".to_string(),
            auth: AuthMethod::Key {
                pubkey: format!("{home}/.ssh/id_rsa.pub"),
                privkey: format!("{home}/.ssh/id_rsa"),
                passphrase: None,
            },
        })
        .collect();
    let local_base = Path::new("/var/tmp/chess");
    let remote_base = Path::new("/tmp/.chess");
    let _ = concurrent_upload_folders(sftp_configs, images_sha256.clone(), local_base, remote_base)
        .await;

    Ok(images_sha256)
}

pub async fn tarzxf_remote_server_package(images_sha256: Vec<String>, all_addresses: Vec<String>) {
    let home = env::var("HOME").unwrap();
    let commands = build_std_linux_tarzxvf_filetoroot_commands(&images_sha256);
    let run_cmd_configs: Vec<ssh_cmd::SshConfig> = all_addresses
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
    let _ = run_commands_on_multiple_hosts(run_cmd_configs, commands, false).await;
}
