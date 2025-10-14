use std::fs::OpenOptions;
use std::io::{self, Write};

pub fn add_top_boilerplate(file_path: &std::path::Path) -> Result<(), io::Error> {
    // Ensure parent directories exist
    if let Some(parent) = file_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    
    let mut file = OpenOptions::new()
        .write(true) // Enable writing to the file.
        .create(true) // Create the file if it doesn't exist.
        .truncate(true) // Clear the file if it exists
        .open(file_path)
        .map_err(|e| {
            eprintln!("Error opening file {}: {}", file_path.display(), e);
            e
        })?;
    let top_boiler = r###"
use axum::{                                                                                                                                                                      
    extract::{self, Path, Query},  
    routing::{get, post},                                                                                                                                                        
    Json, Router,                        
};       
use minio_rsc::{Minio, provider::StaticProvider, client::PresignedArgs};
use serde::{Deserialize, Serialize};                                                                                                                                                          
use serde_json::{json, Value};                                                                                                                                                  
use sqlx::PgPool;                                                                                                                                                               
use sqlx::{postgres::PgPoolOptions, prelude::FromRow};                                                                                                                           
use std::env;                                                                                                                                                                    
use std::net::SocketAddr;                                                                                                                                                        
use std::result::Result;                                                                                                                                                         
use std::sync::Arc;                                                                                                                                                              
use axum::http::StatusCode;                  
use sqlx::types::chrono::Utc; 
use std::collections::HashMap;
use tower_http::cors::{AllowOrigin, CorsLayer};
use axum::http::Method;
use reqwest;


"###;
    file.write_all(top_boiler.as_bytes())?;

    Ok(())
} 

pub fn add_axum_end(funcs: Vec<String>, file_path: &std::path::Path) -> Result<(), io::Error> {
    // Ensure parent directories exist
    if let Some(parent) = file_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    
    // Create tests directory structure
    let project_root = file_path.parent().unwrap().parent().unwrap();
    crate::add_tests::create_test_directory_structure(project_root)?;
    crate::add_tests::add_test_dependencies_to_cargo_toml(project_root)?;
    
    // TODO: Pass table names to generate database-specific test utilities
    // crate::add_tests::generate_database_test_utilities(project_root, &table_names)?;
    // crate::add_tests::generate_crud_tests(project_root, &table_names)?;
    // crate::add_tests::generate_query_param_tests(project_root, &table_names)?;
    // crate::add_tests::generate_error_handling_tests(project_root, &table_names)?;
    
    let mut file = OpenOptions::new()
        .write(true) // Enable writing to the file.
        .create(true) // Create the file if it doesn't exist.
        .append(true) // Set the append mode.
        .open(file_path)
        .map_err(|e| {
            eprintln!("Error opening file {}: {}", file_path.display(), e);
            e
        })?;
    let mut routs: String = funcs.iter().map(|func| {
        let http_method = if func.starts_with("get") { "get" } else { "post" };
        format!("\t.route(\"/{func}\", {http_method}({func}))\n").to_string()
    }).collect::<String>();
    routs.push_str("\t.route(\"/signed-urls/:video_path\", get(get_signed_url))\n");
    let ending = format!(r###"
async fn health() -> String {{"healthy".to_string() }}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {{
    let db_url = env::var("DATABASE_URL")
     .unwrap_or_else(|_| "postgres://dbuser:p@localhost:1111/data".to_string());
    let pool = PgPoolOptions::new()
        .max_connections(100)
        .connect(&db_url)
        .await?;

    let migrate = sqlx::migrate!("./migrations").run(&pool).await;

    match migrate {{
        Ok(_) => println!("Migrations applied successfully."),
        Err(e) => eprintln!("Error applying migrations: {{}}", e),
    }};

    let app = Router::new()
    .route("/health", get(health))
    {routs}
    .route("/python", get(python))
    .layer(
        CorsLayer::new()
            .allow_origin(AllowOrigin::list(vec![
                "http://localhost:3000".parse().unwrap(),
                "https://example.com".parse().unwrap(),
            ]))
            .allow_methods([Method::GET, Method::POST])
            .allow_headers(tower_http::cors::Any)
    )
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8081").await.unwrap();

    axum::serve(listener, app).await.unwrap();
    Ok(())
}}


"###);  //https://tidelabs.github.io/tidechain/tower_http/cors/struct.CorsLayer.html (may help with auth) 
    
    file.write_all(ending.as_bytes())?; // comment for testing 
    Ok(())
}
