use std::env;
use std::path::Path;
use woker::sftp::AuthMethod;
use woker::sftp::SshConfig;
use woker::sftp::connect_ssh;
use woker::sftp::upload_folder;

#[test]
fn test_sftp() -> Result<(), Box<dyn std::error::Error>> {
    let home = env::var("HOME")?;
    let config = SshConfig {
        host: "47.76.42.207".to_string(),
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
