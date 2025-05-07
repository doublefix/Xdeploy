use std::{env, process::Command};

#[test]
fn test_ansible() {
    let mut home_path = "".to_string();
    if let Ok(home) = env::var("HOME") {
        println!("$HOME: {home}");
        home_path = home;
    } else {
        eprintln!("$HOME is not set");
    }
    let playbook = format!("{home_path}/code/Xdeploy/playbooks/cmd.yml");
    let inventory = format!("{home_path}/code/Xdeploy/inventory/test");
    let target_host = "debian-root";
    let extra_vars = r#"{"cmd": ["echo", "Hello", "World"]}"#;

    let output = Command::new("ansible-playbook")
        .arg(playbook)
        .arg("-i")
        .arg(inventory)
        .arg("-l")
        .arg(target_host)
        .arg("-e")
        .arg(extra_vars)
        .arg("-v")
        .output()
        .expect("Failed to execute ansible-playbook");

    println!("status: {}", output.status);
    println!("stdout:\n{}", String::from_utf8_lossy(&output.stdout));
    eprintln!("stderr:\n{}", String::from_utf8_lossy(&output.stderr));
}
