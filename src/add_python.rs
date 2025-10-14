use std::{fs::OpenOptions, io};
use std::io::Write;


pub fn add_python_func(file_path: &std::path::Path) -> Result<(), io::Error> {
let python_func = format!(r###"


async fn python() -> Result<Json<Value>, (StatusCode, String)> {{
    // Call the Python FastAPI service
    let client = reqwest::Client::new();
    let res = client
        .get("http://python:8003/chat")  // Use service name and correct port
        .send()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Request failed: {{}}", e)))?;

    if res.status().is_client_error() || res.status().is_server_error() {{
        return Err((StatusCode::BAD_REQUEST, format!("Error from Python service: {{}}", res.status())));
    }}

    let json_response: Value = res
        .json()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to parse JSON: {{}}", e)))?;

    Ok(Json(json!({{"payload": json_response}})))
}}

"###);
    let mut file = OpenOptions::new()
        .write(true) // Enable writing to the file.
        .append(true) // Set the append mode.  Crucially, this makes it append.
        .create(true) // Create the file if it doesn't exist.
        .open(file_path)?; // Open the file, returning a Result.


    file.write_all(python_func.as_bytes())?;
    Ok(())
}