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

    // Create remote folder if it doesn't exist
    if sftp.stat(remote_folder).is_err() {
        sftp.mkdir(remote_folder, 0o755)?;
    }

    for entry in fs::read_dir(local_folder)? {
        let entry = entry?;
        let path = entry.path();
        let remote_path = remote_folder.join(entry.file_name());

        if path.is_dir() {
            // Recursively upload subdirectories
            upload_folder(sess, &path, &remote_path)?;
        } else {
            // Upload files
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

    let path = "jupyter";
    let source_path = format!("{home}/{path}");
    let target_path = format!("/root/{path}");

    let local_path = Path::new(&source_path);
    let remote_path = Path::new(&target_path);

    println!("Starting folder upload...");
    upload_folder(&sess, local_path, remote_path)?;
    println!("Folder upload completed!");

    Ok(())
}
