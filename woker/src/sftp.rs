use futures::stream::{FuturesUnordered, StreamExt};
use log::{info, warn};
use ssh2::Session;
use std::{
    fs,
    io::{Read, Write},
    net::TcpStream,
    path::{Path, PathBuf},
};
use tokio::task;

#[derive(Clone)]
pub enum AuthMethod {
    Key {
        pubkey: String,
        privkey: String,
        passphrase: Option<String>,
    },
}

#[derive(Clone)]
pub struct SshConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub auth: AuthMethod,
}

fn connect_ssh(config: &SshConfig) -> Result<Session, Box<dyn std::error::Error + Send + Sync>> {
    let tcp = TcpStream::connect(format!("{}:{}", config.host, config.port))?;
    let mut sess = Session::new()?;
    sess.set_tcp_stream(tcp);
    sess.handshake()?;

    match &config.auth {
        AuthMethod::Key {
            pubkey,
            privkey,
            passphrase,
        } => {
            sess.userauth_pubkey_file(
                &config.username,
                Some(Path::new(pubkey)),
                Path::new(privkey),
                passphrase.as_deref(),
            )?;
        }
    }

    if !sess.authenticated() {
        return Err("Authentication failed".into());
    }

    Ok(sess)
}

fn upload_folder_sync(
    sess: &Session,
    local_folder: &Path,
    remote_folder: &Path,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let sftp = sess.sftp()?;
    if sftp.stat(remote_folder).is_err() {
        ensure_remote_dir(&sftp, remote_folder)?;
    }

    for entry in fs::read_dir(local_folder)? {
        let entry = entry?;
        let path = entry.path();
        let remote_path = remote_folder.join(entry.file_name());

        if path.is_dir() {
            upload_folder_sync(sess, &path, &remote_path)?;
        } else if sftp.stat(&remote_path).is_err() {
            let mut local_file = fs::File::open(&path)?;
            let mut remote_file = sftp.create(&remote_path)?;

            let mut buffer = Vec::new();
            local_file.read_to_end(&mut buffer)?;
            remote_file.write_all(&buffer)?;
        }
    }

    Ok(())
}

fn ensure_remote_dir(
    sftp: &ssh2::Sftp,
    remote_dir: &Path,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut current = PathBuf::from("/");
    for part in remote_dir.components() {
        current = current.join(part);
        if sftp.stat(&current).is_err() {
            sftp.mkdir(&current, 0o755).ok();
        }
    }
    Ok(())
}

async fn upload_task(config: SshConfig, local_path: PathBuf, remote_path: PathBuf) {
    // Clone for use after move into closure
    let config_host = config.host.clone();
    let local_path_disp = local_path.display().to_string();
    let remote_path_disp = remote_path.display().to_string();

    let result = task::spawn_blocking(move || {
        let sess = connect_ssh(&config)?;
        upload_folder_sync(&sess, &local_path, &remote_path)?;
        Ok::<_, Box<dyn std::error::Error + Send + Sync>>(())
    })
    .await;

    match result {
        Ok(Ok(())) => {
            info!("Upload success to {config_host}: {local_path_disp} -> {remote_path_disp}")
        }
        Ok(Err(e)) => warn!("Upload failed to {config_host}: {e}"),
        Err(e) => warn!("Upload failed to {config_host}: JoinError: {e}"),
    }
}

pub async fn concurrent_upload_folders(
    configs: Vec<SshConfig>,
    folders: Vec<String>,
    local_base: &Path,
    remote_base: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut tasks = FuturesUnordered::new();

    for config in configs.into_iter() {
        for folder in folders.iter() {
            let local_path = local_base.join(folder);
            let remote_path = remote_base.join(folder);
            tasks.push(upload_task(config.clone(), local_path, remote_path));
        }
    }
    while (tasks.next().await).is_some() {}

    Ok(())
}
