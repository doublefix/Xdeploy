use ssh2::Session;
use std::fs;
use std::io::Read;
use std::io::Write;
use std::net::TcpStream;
use std::path::Path;

pub enum AuthMethod {
    Key {
        pubkey: String,
        privkey: String,
        passphrase: Option<String>,
    },
}

pub struct SshConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub auth: AuthMethod,
}

pub fn connect_ssh(config: &SshConfig) -> Result<Session, Box<dyn std::error::Error>> {
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

pub fn upload_folder(
    sess: &Session,
    local_folder: &Path,
    remote_folder: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let sftp = sess.sftp()?;

    // Check if remote folder exists first
    if remote_folder_exists(&sftp, remote_folder)? {
        println!(
            "Remote folder already exists: {}, skipping upload",
            remote_folder.display()
        );
        return Ok(());
    }

    ensure_remote_dir(&sftp, remote_folder)?;

    for entry in fs::read_dir(local_folder)? {
        let entry = entry?;
        let path = entry.path();
        let remote_path = remote_folder.join(entry.file_name());

        if path.is_dir() {
            upload_folder(sess, &path, &remote_path)?;
        } else {
            let mut local_file = fs::File::open(&path)?;
            let mut remote_file = sftp.create(&remote_path)?;

            let mut buffer = Vec::new();
            local_file.read_to_end(&mut buffer)?;
            remote_file.write_all(&buffer)?;

            println!("Uploaded: {} -> {}", path.display(), remote_path.display());
        }
    }

    Ok(())
}

fn remote_folder_exists(
    sftp: &ssh2::Sftp,
    remote_folder: &Path,
) -> Result<bool, Box<dyn std::error::Error>> {
    match sftp.stat(remote_folder) {
        Ok(_) => Ok(true),
        Err(e) => {
            if e.code() == ssh2::ErrorCode::Session(-31) {
                // SSH_FX_NO_SUCH_FILE
                Ok(false)
            } else {
                Err(Box::new(e))
            }
        }
    }
}

pub fn ensure_remote_dir(
    sftp: &ssh2::Sftp,
    remote_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut current = std::path::PathBuf::from("/");
    for part in remote_dir.components() {
        current = current.join(part);
        if sftp.stat(&current).is_err() {
            match sftp.mkdir(&current, 0o755) {
                Ok(_) => {}
                Err(e) => {
                    if e.code() != ssh2::ErrorCode::Session(-31) {
                        return Err(Box::new(e));
                    }
                }
            }
        }
    }
    Ok(())
}
