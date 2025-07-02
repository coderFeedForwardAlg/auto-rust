use std::{fs::OpenOptions, io::{self, BufWriter, Write}};




pub fn add_minio(file_path: &std::path::Path) -> Result<String, io::Error> {
    // Ensure parent directories exist
    if let Some(parent) = file_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    
    let funk_str = format!(r###"
async fn generate_signed_url(object_key: String) -> Result<String, anyhow::Error> {{
    let endpoint = env::var("MINIO_ENDPOINT")
        .unwrap_or_else(|_| "localhost:9001".to_string());
    let access_key = env::var("MINIO_ACCESS_KEY").unwrap_or_else(|_| "minioadmin".to_string());
    let secret_key = env::var("MINIO_SECRET_KEY").unwrap_or_else(|_| "minioadmin".to_string());
    let bucket = env::var("MINIO_BUCKET").unwrap_or_else(|_| "bucket".to_string());
    let endpoint = env::var("MINIO_ENDPOINT").unwrap_or_else(|_| "localhost:9000".to_string());
    let secure = env::var("MINIO_SECURE")
        .map(|s| s.to_lowercase() == "true")
        .unwrap_or(false);

    let provider = StaticProvider::new(&access_key, &secret_key, None);

    let minio = Minio::builder()
        .endpoint(&endpoint)
        .provider(provider)
        .secure(secure)
        .region("us-east-1".to_string())  // Explicitly set region to match MinIO default
        .build()
        .map_err(|e| anyhow::anyhow!("Failed to create MinIO client: {{}}", e))?;

    let presigned_url = minio
        .presigned_get_object(
            PresignedArgs::new(bucket, object_key)
                .expires(3600),  // 1 hour in seconds
        )
        .await
        .map_err(|e| anyhow::anyhow!("Failed to generate presigned URL: {{}}", e))?;
    Ok(presigned_url)
}}
    
use axum::response::IntoResponse;

async fn get_signed_url(
    Path(video_path): Path<String>,
) -> impl IntoResponse {{
    let object_key = video_path; 
    println!("Environment variables:");
    println!("MINIO_ENDPOINT: {{}}", env::var("MINIO_ENDPOINT").unwrap_or_else(|_| "not set".to_string()));
    println!("MINIO_BUCKET: {{}}", env::var("MINIO_BUCKET").unwrap_or_else(|_| "not set, using default 'test'".to_string()));
    
    match generate_signed_url(object_key).await {{
        Ok(url) => (StatusCode::OK, url).into_response(),
        Err(e) => {{
            eprintln!("Error generating signed URL: {{}}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to generate signed URL: {{}}", e)).into_response()
        }}
    }}
}}
async fn upload_video(
    // mut multipart: Multipart,
) -> Result<Json<Value>, (StatusCode, String)> {{
    let provider = StaticProvider::new("minioadmin", "minioadmin", None);
    let minio = Minio::builder()
        .endpoint("minio:9000")
        .provider(provider)
        .secure(false)
        .build()
        .unwrap();

        let _data = "hello minio";

        let upload_result = minio.put_object("bucket", "file.txt", _data.into()).await;
        
        return Ok(Json(json!({{
            "status": upload_result.is_ok(),
            "message": if upload_result.is_ok() {{
                "File uploaded successfully"
            }} else {{
                "Failed to upload file"
            }},
            "file_name": "file.txt"
        }})));
}}
    "###);

    // Open file with proper error handling
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open(file_path)
        .map_err(|e| {
            eprintln!("Error opening file {}: {}", file_path.display(), e);
            e
        })?;
        
    let mut file = BufWriter::new(file);
    file.write(funk_str.as_bytes())?;

    Ok("upload_video".to_string())
}

