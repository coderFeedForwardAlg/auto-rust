

use std::{fs::File, io::Write};

/// Generates a TOML configuration file for a Rust project.
///
/// This function creates a TOML file with specific dependencies for a Rust project 
/// located at the given `project_dir` with the specified `file_name`.
///
/// # Arguments
///
/// * `project_dir` - The directory where the project is located.
/// * `file_name` - The name of the project.
///
/// # Returns
///
/// Returns a `Result` containing the generated TOML content as a string, 
/// or an error if the operation fails.
pub async fn gen_toml(project_dir: &std::path::PathBuf, file_name: String) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let deps = "
    axum = { version = \"0.7\", features = [\"macros\"] }
tokio = { version = \"1\", features = [\"full\", \"time\"] }
serde = { version = \"1\", features = [\"derive\"] }
serde_json = \"1\"
sqlx = { version = \"0.7\", features = [\"runtime-tokio-rustls\", \"postgres\", \"chrono\", \"uuid\"] } # Added \"uuid\" feature as it's often used with database interactions.
dotenv = \"0.15\" # Useful for loading environment variables like your database URL
tower-http = { version = \"0.5\", features = [\"cors\"] } # For CorsLayer
chrono = { version = \"0.4\", features = [\"serde\"] } # For Utc
uuid = { version = \"1\", features = [\"serde\", \"v4\"] } # For UUID generation and serialization


    ";
 
    let mut file = File::create(project_dir.join(file_name)).map_err(|e| {
        eprintln!("Error creating file: {}", e);
        e
    })?;

    file.write_all(deps.as_bytes()).map_err(|e| {
        eprintln!("Error writing to file: {}", e);
        e
    })?;
    Ok(deps.to_string())
}
    
