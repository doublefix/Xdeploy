use std::path::Path;

use woker::sftp::AuthMethod;
use woker::sftp::SshConfig;
use woker::sftp::concurrent_upload_folders;

#[tokio::test]
async fn test_concurrent_upload() -> Result<(), Box<dyn std::error::Error>> {
    let home = std::env::var("HOME")?;
    let configs = vec![SshConfig {
        host: "ubuntu".to_string(),
        port: 22,
        username: "root".to_string(),
        auth: AuthMethod::Key {
            pubkey: format!("{home}/.ssh/id_rsa.pub"),
            privkey: format!("{home}/.ssh/id_rsa"),
            passphrase: None,
        },
    }];

    let folders =
        vec!["ee65adc925d6d5acd33beeba4747f90fda68bec1dbea6a1dea16691fe9fdfeb8".to_string()];

    let local_base = Path::new("/var/tmp/chess");
    let remote_base = Path::new("/tmp/.chess");

    concurrent_upload_folders(configs, folders, local_base, remote_base).await
}
