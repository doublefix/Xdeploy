use clap::{Arg, ArgMatches, Command};
use log::{info, warn};
use std::error::Error;
use std::path::{Path, PathBuf};
use tokio::fs as async_fs;
use tokio::process::Command as ProcessCommand;

pub fn build_cli() -> Command {
    Command::new("Docker Image Content Extractor")
        .version("1.0")
        .author("Your Name")
        .about("Extracts content from Docker images using nerdctl")
        .arg(
            Arg::new("images")
                .short('i')
                .long("image")
                .value_name("IMAGE")
                .help("Docker images to extract content from (comma-separated or multiple flags)")
                .default_value("harbor.openpaper.co/chess/kubernetes:v1.31.0")
                .value_delimiter(',')
                .num_args(1..),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("OUTPUT_DIR")
                .help("Base directory to extract content to"),
        )
}

pub async fn run(
    matches: &ArgMatches,
) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
    let images = matches
        .get_many::<String>("images")
        .unwrap()
        .map(|s| s.to_string())
        .collect::<Vec<_>>();

    // Determine output directory
    let base_output_dir = if let Some(output) = matches.get_one::<String>("output") {
        PathBuf::from(output)
    } else {
        PathBuf::from("/var/tmp/chess")
    };

    // Process images concurrently and collect SHA256 values
    let mut handles = vec![];
    for image in images {
        let base_output_dir = base_output_dir.clone();
        let handle = tokio::spawn(async move {
            match process_image(&image, base_output_dir.as_path()).await {
                Ok(sha256) => Ok(sha256),
                Err(e) => {
                    warn!("Error processing image {image}: {e}");
                    Err(e)
                }
            }
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete and collect results
    let mut sha256_values = Vec::new();
    for handle in handles {
        match handle.await {
            Ok(Ok(sha256)) => sha256_values.push(sha256),
            Ok(Err(e)) => return Err(e),
            Err(e) => return Err(e.into()),
        }
    }

    // Deduplicate SHA256 values while preserving order
    sha256_values.sort();
    sha256_values.dedup();

    Ok(sha256_values)
}

pub async fn process_image(
    image: &str,
    base_output_dir: &Path,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    info!("Processing image: {image}");

    // Pull image
    info!("Pulling image: {image}");
    let pull_output = run_command("nerdctl", &["pull", image]).await?;
    if !pull_output.status.success() {
        return Err(format!(
            "Failed to pull image: {}\n{}",
            image,
            String::from_utf8_lossy(&pull_output.stderr)
        )
        .into());
    }

    // Inspect image to get ID
    info!("Getting image ID for {image}...");
    let inspect_output = run_command(
        "nerdctl",
        &["image", "inspect", "--format", "{{.ID}}", image],
    )
    .await?;
    if !inspect_output.status.success() {
        return Err(format!(
            "Failed to inspect image: {}\n{}",
            image,
            String::from_utf8_lossy(&inspect_output.stderr)
        )
        .into());
    }

    let image_id = String::from_utf8_lossy(&inspect_output.stdout)
        .trim()
        .strip_prefix("sha256:")
        .ok_or("Unexpected image ID format")?
        .to_string();

    info!("Image ID: {image_id}");

    let output_dir = base_output_dir.join(&image_id);

    // Skip if already exists
    if async_fs::metadata(&output_dir).await.is_ok() {
        info!("Content already exists at: {}", output_dir.display());
        return Ok(image_id);
    }

    // Create output dir
    info!("Creating output directory: {}", output_dir.display());
    async_fs::create_dir_all(&output_dir).await?;

    // Step 1: Start container that stays alive
    let container_name = format!("extract-{}", uuid::Uuid::new_v4());
    info!("Starting container: {container_name}");

    let create_output = run_command(
        "nerdctl",
        &[
            "run",
            "-d",
            "--name",
            &container_name,
            image,
            "tail",
            "-f",
            "/dev/null",
        ],
    )
    .await?;
    if !create_output.status.success() {
        return Err(format!(
            "Failed to start container: {}\n{}",
            image,
            String::from_utf8_lossy(&create_output.stderr)
        )
        .into());
    }

    // Step 2: Copy /archive from container to output_dir
    info!("Copying files from container...");
    let copy_output = run_command(
        "nerdctl",
        &[
            "cp",
            &format!("{container_name}:/archive/."),
            output_dir.to_str().ok_or("Invalid output_dir")?,
        ],
    )
    .await?;

    if !copy_output.status.success() {
        // Consider leaving container for debug?
        return Err(format!(
            "Failed to copy content from container: {}\n{}",
            container_name,
            String::from_utf8_lossy(&copy_output.stderr)
        )
        .into());
    }

    // Step 3: Remove container
    info!("Removing container: {container_name}");
    let rm_output = run_command("nerdctl", &["rm", "-f", &container_name]).await?;
    if !rm_output.status.success() {
        return Err(format!(
            "Failed to remove container: {}\n{}",
            container_name,
            String::from_utf8_lossy(&rm_output.stderr)
        )
        .into());
    }

    info!(
        "Content extracted successfully to: {}",
        output_dir.display()
    );

    Ok(image_id)
}

async fn run_command(
    program: &str,
    args: &[&str],
) -> Result<std::process::Output, Box<dyn std::error::Error + Send + Sync>> {
    let output = ProcessCommand::new(program).args(args).output().await?;

    if !output.stdout.is_empty() {
        print!("{}", String::from_utf8_lossy(&output.stdout));
    }

    if !output.stderr.is_empty() && !output.status.success() {
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
    }

    Ok(output)
}

pub async fn load_image(
    images: Vec<String>,
    output_dir: Option<String>,
) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
    let mut args = vec!["extractor".to_string(), "-i".to_string()];
    args.push(images.join(","));

    if let Some(dir) = output_dir {
        args.push("-o".to_string());
        args.push(dir);
    }
    let cli_args: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

    let matches = build_cli().try_get_matches_from(cli_args)?;
    let sha256_values = run(&matches).await?;

    Ok(sha256_values)
}
