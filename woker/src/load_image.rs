use clap::{Arg, ArgMatches, Command};
use std::path::PathBuf;
use tokio::fs;
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
                    eprintln!("Error processing image {image}: {e}");
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

async fn process_image(
    image: &str,
    base_output_dir: &std::path::Path,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    println!("Processing image: {image}");

    // Get image ID first to check if content already exists
    println!("Getting image ID for {image}...");
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

    println!("Image ID: {image_id}");

    let output_dir = base_output_dir.join(&image_id);

    // Check if content already exists
    if fs::metadata(&output_dir).await.is_ok() {
        println!("Content already exists at: {}", output_dir.display());
        return Ok(image_id);
    }

    // If content doesn't exist, proceed with pulling and extracting
    println!("Pulling image: {image}");
    let pull_output = run_command("nerdctl", &["pull", image]).await?;
    if !pull_output.status.success() {
        return Err(format!(
            "Failed to pull image: {}\n{}",
            image,
            String::from_utf8_lossy(&pull_output.stderr)
        )
        .into());
    }

    println!("Creating output directory: {}", output_dir.display());
    fs::create_dir_all(&output_dir).await?;

    println!("Extracting content from image...");
    let extract_output = run_command(
        "nerdctl",
        &[
            "run",
            "--rm",
            "-v",
            &format!("{}:/extract", output_dir.display()),
            image,
            "sh",
            "-c",
            "cp -r /archive/. /extract/",
        ],
    )
    .await?;

    if !extract_output.status.success() {
        return Err(format!(
            "Failed to extract content: {}\n{}",
            image,
            String::from_utf8_lossy(&extract_output.stderr)
        )
        .into());
    }
    println!(
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
