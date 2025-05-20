use clap::{Arg, ArgMatches, Command};
use std::fs;
use std::path::PathBuf;
use std::process::{Command as ProcessCommand, Output};

pub fn build_cli() -> Command {
    Command::new("Docker Image Content Extractor")
        .version("1.0")
        .author("Your Name")
        .about("Extracts content from a Docker image using nerdctl")
        .arg(
            Arg::new("image")
                .short('i')
                .long("image")
                .value_name("IMAGE")
                .help("Docker image to extract content from")
                .default_value("harbor.openpaper.co/chess/kubernetes:v1.31.0"),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("OUTPUT_DIR")
                .help("Base directory to extract content to"),
        )
}

pub fn run(matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let image = matches.get_one::<String>("image").unwrap();

    println!("Pulling image: {image}");
    let pull_output = run_command("nerdctl", &["pull", image])?;
    if !pull_output.status.success() {
        return Err(format!(
            "Failed to pull image: {}\n{}",
            image,
            String::from_utf8_lossy(&pull_output.stderr)
        )
        .into());
    }

    println!("Getting image ID...");
    let inspect_output = run_command(
        "nerdctl",
        &["image", "inspect", "--format", "{{.ID}}", image],
    )?;
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

    let base_output_dir = if let Some(output) = matches.get_one::<String>("output") {
        PathBuf::from(output)
    } else {
        PathBuf::from("/var/tmp/chess")
    };

    let output_dir = base_output_dir.join(&image_id);
    println!("Creating output directory: {}", output_dir.display());
    fs::create_dir_all(&output_dir)?;

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
    )?;

    if !extract_output.status.success() {
        return Err(format!(
            "Failed to extract content: {}\n{}",
            image,
            String::from_utf8_lossy(&extract_output.stderr)
        )
        .into());
    }

    println!("Listing extracted files:");
    let ls_output = run_command("ls", &["-l", &output_dir.to_string_lossy()])?;
    if !ls_output.status.success() {
        return Err(format!(
            "Failed to list files: {}\n{}",
            output_dir.display(),
            String::from_utf8_lossy(&ls_output.stderr)
        )
        .into());
    }

    println!("{}", String::from_utf8_lossy(&ls_output.stdout));
    println!(
        "Content extracted successfully to: {}",
        output_dir.display()
    );

    Ok(())
}

fn run_command(program: &str, args: &[&str]) -> Result<Output, Box<dyn std::error::Error>> {
    let output = ProcessCommand::new(program).args(args).output()?;

    if !output.stdout.is_empty() {
        print!("{}", String::from_utf8_lossy(&output.stdout));
    }

    if !output.stderr.is_empty() && !output.status.success() {
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
    }

    Ok(output)
}

#[test]
fn test_load_image() -> Result<(), Box<dyn std::error::Error>> {
    let args = vec![
        "extractor", // argv[0]
        "-i",
        "harbor.openpaper.co/chess/kubernetes:v1.31.0",
        // "-o",
        // "$HOME/code/Xdeploy/tmp",
    ];
    let matches = build_cli().try_get_matches_from(args)?;
    run(&matches)?;
    Ok(())
}
