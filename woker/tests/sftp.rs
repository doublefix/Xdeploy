use ssh2::Session;
use std::env;
use std::fs;
use std::io::Read;
use std::io::Write;
use std::net::TcpStream;
use std::path::Path;

enum AuthMethod {
    Key {
        pubkey: String,
        privkey: String,
        passphrase: Option<String>,
    },
}

struct SshConfig {
    host: String,
    port: u16,
    username: String,
    auth: AuthMethod,
}

fn connect_ssh(config: &SshConfig) -> Result<Session, Box<dyn std::error::Error>> {
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

fn upload_folder(
    sess: &Session,
    local_folder: &Path,
    remote_folder: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let sftp = sess.sftp()?;

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

fn ensure_remote_dir(
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

#[test]
fn test_sftp() -> Result<(), Box<dyn std::error::Error>> {
    let home = env::var("HOME")?;
    let config = SshConfig {
        host: "ubuntu".to_string(),
        port: 22,
        username: "root".to_string(),
        auth: AuthMethod::Key {
            pubkey: format!("{home}/.ssh/id_rsa.pub"),
            privkey: format!("{home}/.ssh/id_rsa"),
            passphrase: None,
        },
    };

    let sess = connect_ssh(&config)?;

    let source_path = "/var/tmp/chess";
    let folder_name = "ee65adc925d6d5acd33beeba4747f90fda68bec1dbea6a1dea16691fe9fdfeb8";
    let target_path = "/tmp/.chess";
    let source_path_with_folder_name = format!("{source_path}/{folder_name}");
    let target_path_with_folder_name = format!("{target_path}/{folder_name}");

    let local_path = Path::new(&source_path_with_folder_name);
    let remote_path = Path::new(&target_path_with_folder_name);

    println!("Starting folder upload...");
    upload_folder(&sess, local_path, remote_path)?;
    println!("Folder upload completed!");

    Ok(())
}
