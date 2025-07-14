use std::{fs::{File, OpenOptions}, io::{BufWriter, Write}};


pub fn add_object(path:  &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    print!("{}\n", path.display());
let object = format!(
"
async fn generate_signed_url(object_key: String) -> Result<String, anyhow::Error> {{
    println!(\"Generating signed URL for object key: {{}}\", object_key);
    // Get MinIO configuration from environment
    let endpoint = env::var(\"MINIO_ENDPOINT\")
        .unwrap_or_else(|_| \"localhost:9001\".to_string());
    // Get configuration from environment variables with sensible defaults
    let access_key = env::var(\"MINIO_ACCESS_KEY\").unwrap_or_else(|_| \"minioadmin\".to_string());
    let secret_key = env::var(\"MINIO_SECRET_KEY\").unwrap_or_else(|_| \"minioadmin\".to_string());
    let bucket = env::var(\"MINIO_BUCKET\").unwrap_or_else(|_| \"bucket\".to_string());
    let endpoint = env::var(\"MINIO_ENDPOINT\").unwrap_or_else(|_| \"localhost:9000\".to_string());
    let secure = env::var(\"MINIO_SECURE\")
        .map(|s| s.to_lowercase() == \"true\")
        .unwrap_or(false);

    // Create credentials provider with the actual credentials
    let provider = StaticProvider::new(&access_key, &secret_key, None);

    // Create MinIO client with the configured endpoint and path-style addressing
    let minio = Minio::builder()
        .endpoint(&endpoint)
        .provider(provider)
        .secure(secure)
        .region(\"us-east-1\".to_string())  // Explicitly set region to match MinIO default
        .build()
        .map_err(|e| anyhow::anyhow!(\"Failed to create MinIO client: {{}}\", e))?;

    // Generate pre-signed URL (valid for 1 hour)
    let presigned_url = minio
        .presigned_get_object(
            PresignedArgs::new(bucket, object_key)
                .expires(3600),  // 1 hour in seconds
        )
        .await
        .map_err(|e| anyhow::anyhow!(\"Failed to generate presigned URL: {{}}\", e))?;
    Ok(presigned_url)
}}

use axum::response::IntoResponse;


async fn get_signed_url(
    Path(video_path): Path<String>,
) -> impl IntoResponse {{
    let object_key = video_path;
    // Log environment variables for debugging
    println!(\"Environment variables:\");
    println!(\"MINIO_ENDPOINT: {{}}\", env::var(\"MINIO_ENDPOINT\").unwrap_or_else(|_| \"not set\".to_string()));
    println!(\"MINIO_BUCKET: {{}}\", env::var(\"MINIO_BUCKET\").unwrap_or_else(|_| \"not set, using default 'bucket'\".to_string()));

    match generate_signed_url(object_key).await {{
        Ok(url) => (StatusCode::OK, url).into_response(),
        Err(e) => {{
            eprintln!(\"Error generating signed URL: {{}}\", e);
            (StatusCode::INTERNAL_SERVER_ERROR, format!(\"Failed to generate signed URL: {{}}\", e)).into_response()
}}
}}
}}

");
  
    // Open file with proper error handling
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| {
            eprintln!("Error opening file {}: {}", path.display(), e);
            e
        })?;
        
    let mut file = BufWriter::new(file);
    file.write_all(object.as_bytes())?;

    Ok(())
}


