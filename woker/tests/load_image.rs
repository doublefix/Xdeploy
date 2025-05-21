use woker::load_image::load_image;

#[tokio::test]
async fn test_get_image_sha256() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let images = vec![
        "harbor.openpaper.co/chess/kubernetes:v1.31.0".to_string(),
        "harbor.openpaper.co/chess/kubernetes:v1.31.0".to_string(), // Duplicate to test deduplication
    ];

    let sha256_values = load_image(images, None).await?;
    println!("Image SHA256 values: {sha256_values:?}");

    Ok(())
}

// "-o",
// "$HOME/code/Xdeploy/tmp",
