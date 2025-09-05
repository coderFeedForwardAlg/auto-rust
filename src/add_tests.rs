use std::fs;
use std::io;
use std::path::Path;

pub fn create_test_directory_structure(project_root: &Path) -> Result<(), io::Error> {
    // Create the tests directory
    let tests_dir = project_root.join("tests");
    fs::create_dir_all(&tests_dir)?;
    
    // Create a basic integration test file
    let integration_test_file = tests_dir.join("api_integration_tests.rs");
    let integration_test_content = r###"//! Integration tests for the generated API endpoints

use reqwest;
use serde_json::Value;
use std::process::Command;

// Helper function to start the test server
async fn start_test_server() -> Result<tokio::process::Child, Box<dyn std::error::Error>> {
    let mut cmd = Command::new("cargo");
    cmd.arg("run");
    // You might want to set a different port for testing
    cmd.env("PORT", "8082");
    
    let child = tokio::process::Command::from(cmd)
        .spawn()
        .map_err(|e| format!("Failed to start test server: {}", e))?;
        
    // Give the server some time to start
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    
    Ok(child)
}

// Helper function to create a test client
fn create_test_client() -> reqwest::Client {
    reqwest::Client::new()
}

#[tokio::test]
async fn test_health_endpoint() {
    // Start the test server
    let _server_handle = start_test_server().await.expect("Failed to start test server");
    
    // Create a client
    let client = create_test_client();
    
    // Make a request to the health endpoint
    let response = client
        .get("http://localhost:8082/health")
        .send()
        .await
        .expect("Failed to send request");
        
    assert_eq!(response.status(), 200);
    
    let body = response
        .text()
        .await
        .expect("Failed to read response body");
        
    assert_eq!(body, "healthy");
}
"###;
    
    fs::write(&integration_test_file, integration_test_content)?;
    
    // Create a test utilities module
    let utils_file = tests_dir.join("test_utils.rs");
    let utils_content = r###"//! Test utilities for database setup, data seeding, and cleanup

use sqlx::{PgPool, PgPoolOptions};
use std::env;
use std::process::Command;
use tokio::time::{sleep, Duration};

/// Create a test database connection
pub async fn create_test_database_pool() -> Result<PgPool, Box<dyn std::error::Error>> {
    // Use a separate test database URL, or default to the main one with a test suffix
    let database_url = env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://dbuser:p@localhost:1111/test_data".to_string());
    
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;
    
    Ok(pool)
}

/// Seed the database with test data
pub async fn seed_test_data(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    // This function would be customized based on the generated API's data model
    // For now, we'll provide a generic template
    
    // Example of how to insert test data:
    /*
    sqlx::query!(
        "INSERT INTO users (id, name, email) VALUES ($1, $2, $3)",
        uuid::Uuid::new_v4(),
        "Test User",
        "test@example.com"
    )
    .execute(pool)
    .await?;
    */
    
    Ok(())
}

/// Clean up test data
pub async fn cleanup_test_data(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    // This function would be customized based on the generated API's data model
    // For now, we'll provide a generic template
    
    // Example of how to clean up test data:
    /*
    sqlx::query!("DELETE FROM users WHERE email LIKE 'test_%'")
        .execute(pool)
        .await?;
    */
    
    Ok(())
}

/// Set up a test database with migrations
pub async fn setup_test_database() -> Result<PgPool, Box<dyn std::error::Error>> {
    let pool = create_test_database_pool().await?;
    
    // Run migrations
    sqlx::migrate!("../migrations")
        .run(&pool)
        .await
        .map_err(|e| format!("Failed to run migrations: {}", e))?;
    
    // Seed with test data
    seed_test_data(&pool).await?;
    
    Ok(pool)
}

/// Test server management utilities
pub struct TestServer {
    child: Option<tokio::process::Child>,
    port: u16,
}

impl TestServer {
    /// Create a new test server instance
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Self::with_port(8082).await
    }
    
    /// Create a new test server instance with a specific port
    pub async fn with_port(port: u16) -> Result<Self, Box<dyn std::error::Error>> {
        let mut cmd = Command::new("cargo");
        cmd.arg("run");
        cmd.env("PORT", port.to_string());
        cmd.env("DATABASE_URL", "postgres://dbuser:p@localhost:1111/test_data");
        
        let child = tokio::process::Command::from(cmd)
            .spawn()
            .map_err(|e| format!("Failed to start test server: {}", e))?;
            
        // Give the server some time to start
        sleep(Duration::from_secs(3)).await;
        
        Ok(TestServer {
            child: Some(child),
            port,
        })
    }
    
    /// Get the base URL for the test server
    pub fn base_url(&self) -> String {
        format!("http://localhost:{}", self.port)
    }
    
    /// Get a reqwest client configured for testing
    pub fn client(&self) -> reqwest::Client {
        reqwest::Client::new()
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            // Attempt to kill the child process
            let _ = child.start_kill();
        }
    }
}
"###;
    
    fs::write(&utils_file, utils_content)?;
    
    // Create a CRUD tests module
    let crud_tests_file = tests_dir.join("crud_tests.rs");
    let crud_tests_content = r###"//! CRUD operation tests for generated API endpoints

use reqwest;
use serde_json::Value;

// Helper function to create a test client
fn create_test_client() -> reqwest::Client {
    reqwest::Client::new()
}

// TODO: Generate specific CRUD tests based on the API endpoints
// This file will be auto-generated with tests for each table's CRUD operations
"###;
    
    fs::write(&crud_tests_file, crud_tests_content)?;
    
    // Create a query parameter tests module
    let query_tests_file = tests_dir.join("query_param_tests.rs");
    let query_tests_content = r###"//! Query parameter tests for generated API endpoints

use reqwest;
use serde_json::Value;

// Helper function to create a test client
fn create_test_client() -> reqwest::Client {
    reqwest::Client::new()
}

// TODO: Generate specific query parameter tests based on the API endpoints
// This file will be auto-generated with tests for ordering, filtering, etc.
"###;
    
    fs::write(&query_tests_file, query_tests_content)?;
    
    // Create an error handling tests module
    let error_tests_file = tests_dir.join("error_handling_tests.rs");
    let error_tests_content = r###"//! Error handling and edge case tests for generated API endpoints

use reqwest;
use serde_json::Value;

// Helper function to create a test client
fn create_test_client() -> reqwest::Client {
    reqwest::Client::new()
}

// TODO: Generate specific error handling tests based on the API endpoints
// This file will be auto-generated with tests for various error conditions
"###;
    
    fs::write(&error_tests_file, error_tests_content)?;
    
    // Create a mod.rs file to organize tests
    let mod_file = tests_dir.join("mod.rs");
    let mod_content = r###"//! Test module organization

// Test utilities
pub mod test_utils;

// CRUD operation tests
mod crud_tests;

// Query parameter tests
mod query_param_tests;

// Error handling tests
mod error_handling_tests;

// Integration tests
mod api_integration_tests;
"###;
    
    fs::write(&mod_file, mod_content)?;
    
    Ok(())
}

pub fn add_test_dependencies_to_cargo_toml(project_root: &Path) -> Result<(), io::Error> {
    let cargo_toml_path = project_root.join("Cargo.toml");
    
    // Read the existing Cargo.toml
    let mut content = fs::read_to_string(&cargo_toml_path)?;
    
    // Add test dependencies to the [dev-dependencies] section
    let dev_dependencies = r###"
[dev-dependencies]
tokio-test = "0.4"
reqwest = { version = "0.11", features = ["json"] }
serde_json = "1.0"
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres", "uuid", "chrono", "json"] }
"###;
    
    // If there's no [dev-dependencies] section, add it
    if !content.contains("[dev-dependencies]") {
        content.push_str(dev_dependencies);
    } else {
        // If there is a [dev-dependencies] section, add our dependencies to it
        // This is a simple approach - in a real implementation, you'd want to be more careful
        // about merging dependencies
        content = content.replace("[dev-dependencies]", "[dev-dependencies]");
        if !content.contains("sqlx = { version = \"0.7\"") {
            content = content.replace("[dev-dependencies]", &format!("[dev-dependencies]{}", dev_dependencies.lines().skip(4).collect::<Vec<_>>().join("\n")));
        }
    }
    
    // Write the updated Cargo.toml
    fs::write(&cargo_toml_path, content)?;
    
    Ok(())
}

/// Generate database-specific test utilities based on the generated API's data model
pub fn generate_database_test_utilities(project_root: &Path, table_names: &[String]) -> Result<(), io::Error> {
    let tests_dir = project_root.join("tests");
    let utils_file = tests_dir.join("test_utils.rs");
    
    // Read the existing test_utils.rs
    let mut content = fs::read_to_string(&utils_file)?;
    
    // Generate seed_test_data function based on table names
    let seed_function = generate_seed_function(table_names);
    let cleanup_function = generate_cleanup_function(table_names);
    
    // Replace the placeholder functions with generated ones
    if let Some(start) = content.find("/// Seed the database with test data") {
        if let Some(end) = content[start..].find("/// Clean up test data") {
            let before_seed = &content[..start];
            let after_seed = &content[start + end..];
            content = format!("{}{}{}", before_seed, seed_function, after_seed);
        }
    }
    
    if let Some(start) = content.find("/// Clean up test data") {
        if let Some(end) = content[start..].find("/// Set up a test database with migrations") {
            let before_cleanup = &content[..start];
            let after_cleanup = &content[start + end..];
            content = format!("{}{}{}", before_cleanup, cleanup_function, after_cleanup);
        }
    }
    
    fs::write(&utils_file, content)?;
    
    Ok(())
}

fn generate_seed_function(table_names: &[String]) -> String {
    let mut seed_statements = Vec::new();
    
    for table_name in table_names {
        // Generate example seed statements for each table
        seed_statements.push(format!(
            "    // Example data for {} table\n    // sqlx::query!(\"INSERT INTO {} (...) VALUES (...)\", ...)\n    //     .execute(pool)\n    //     .await?;",
            table_name, table_name
        ));
    }
    
    format!(
        r###"/// Seed the database with test data
pub async fn seed_test_data(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {{
    // This function is auto-generated based on your API's data model
{}
    
    Ok(())
}}
"###,
        seed_statements.join("\n")
    )
}

fn generate_cleanup_function(table_names: &[String]) -> String {
    let mut cleanup_statements = Vec::new();
    
    for table_name in table_names {
        // Generate example cleanup statements for each table
        cleanup_statements.push(format!(
            "    // Clean up {} table\n    // sqlx::query!(\"DELETE FROM {} WHERE ...\")\n    //     .execute(pool)\n    //     .await?;",
            table_name, table_name
        ));
    }
    
    format!(
        r###"/// Clean up test data
pub async fn cleanup_test_data(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {{
    // This function is auto-generated based on your API's data model
{}
    
    Ok(())
}}
"###,
        cleanup_statements.join("\n")
    )
}

/// Generate CRUD operation tests for the API endpoints
pub fn generate_crud_tests(project_root: &Path, table_names: &[String]) -> Result<(), io::Error> {
    let tests_dir = project_root.join("tests");
    let crud_tests_file = tests_dir.join("crud_tests.rs");
    
    let mut test_content = String::from(r###"//! CRUD operation tests for generated API endpoints

use reqwest;
use serde_json::Value;
use crate::test_utils;

// Helper function to create a test client
fn create_test_client() -> reqwest::Client {
    reqwest::Client::new()
}

"###);
    
    // Generate tests for each table
    for table_name in table_names {
        let pascal_case_name = to_pascal_case(table_name);
        let kebab_case_name = to_kebab_case(table_name);
        
        test_content.push_str(&generate_table_crud_tests(table_name, &pascal_case_name, &kebab_case_name));
    }
    
    fs::write(&crud_tests_file, test_content)?;
    
    Ok(())
}

fn generate_table_crud_tests(table_name: &str, pascal_case_name: &str, kebab_case_name: &str) -> String {
    format!(
        r###"
#[tokio::test]
async fn test_{}_crud_operations() {{
    // Setup
    let _server_handle = test_utils::start_test_server().await.expect("Failed to start test server");
    let client = create_test_client();
    let base_url = "http://localhost:8082";
    
    // Test create operation
    let create_response = client
        .post(&format!("{{}}/add_{{}}", base_url, "{}"))
        .json(&serde_json::json!({{ /* TODO: Add sample data */ }}))
        .send()
        .await
        .expect("Failed to send create request");
    
    assert_eq!(create_response.status(), 200);
    
    // Test get all operation
    let get_all_response = client
        .get(&format!("{{}}/get_{{}}", base_url, "{}"))
        .send()
        .await
        .expect("Failed to send get all request");
    
    assert_eq!(get_all_response.status(), 200);
    
    let get_all_body: Value = get_all_response
        .json()
        .await
        .expect("Failed to parse get all response");
    
    assert!(get_all_body.get("payload").is_some());
    
    // TODO: Add tests for get by ID, update, and delete operations
}}

"###,
        table_name.to_lowercase(),
        kebab_case_name,
        kebab_case_name
    )
}

/// Generate query parameter tests for the API endpoints
pub fn generate_query_param_tests(project_root: &Path, table_names: &[String]) -> Result<(), io::Error> {
    let tests_dir = project_root.join("tests");
    let query_tests_file = tests_dir.join("query_param_tests.rs");
    
    let mut test_content = String::from(r###"//! Query parameter tests for generated API endpoints

use reqwest;
use serde_json::Value;
use crate::test_utils;

// Helper function to create a test client
fn create_test_client() -> reqwest::Client {
    reqwest::Client::new()
}

"###);
    
    // Generate tests for each table
    for table_name in table_names {
        let kebab_case_name = to_kebab_case(table_name);
        
        test_content.push_str(&generate_table_query_tests(table_name, &kebab_case_name));
    }
    
    fs::write(&query_tests_file, test_content)?;
    
    Ok(())
}

fn generate_table_query_tests(table_name: &str, kebab_case_name: &str) -> String {
    format!(
        r###"#[tokio::test]
async fn test_{}_ordering() {{
    // Setup
    let _server_handle = test_utils::start_test_server().await.expect("Failed to start test server");
    let client = create_test_client();
    let base_url = "http://localhost:8082";
    
    // Test ordering by a column (ascending)
    let order_asc_response = client
        .get(&format!("{{}}/get_{{}}?order_by=id&direction=asc", base_url, "{}"))
        .send()
        .await
        .expect("Failed to send order asc request");
    
    assert_eq!(order_asc_response.status(), 200);
    
    // Test ordering by a column (descending)
    let order_desc_response = client
        .get(&format!("{{}}/get_{{}}?order_by=id&direction=desc", base_url, "{}"))
        .send()
        .await
        .expect("Failed to send order desc request");
    
    assert_eq!(order_desc_response.status(), 200);
    
    // TODO: Add assertions to verify the ordering is correct
    // This would require seeding the database with known data and checking the order
}}

#[tokio::test]
async fn test_{}_invalid_order_by() {{
    // Setup
    let _server_handle = test_utils::start_test_server().await.expect("Failed to start test server");
    let client = create_test_client();
    let base_url = "http://localhost:8082";
    
    // Test invalid order_by parameter (should return 400 Bad Request)
    let invalid_order_response = client
        .get(&format!("{{}}/get_{{}}?order_by=invalid_column; DROP TABLE users;", base_url, "{}"))
        .send()
        .await
        .expect("Failed to send invalid order request");
    
    // Should return a 400 Bad Request due to validation
    assert_eq!(invalid_order_response.status(), 400);
}}

"###,
        table_name.to_lowercase(),
        kebab_case_name,
        kebab_case_name,
        table_name.to_lowercase(),
        kebab_case_name
    )
}

/// Generate error handling tests for the API endpoints
pub fn generate_error_handling_tests(project_root: &Path, table_names: &[String]) -> Result<(), io::Error> {
    let tests_dir = project_root.join("tests");
    let error_tests_file = tests_dir.join("error_handling_tests.rs");
    
    let mut test_content = String::from(r###"//! Error handling and edge case tests for generated API endpoints

use reqwest;
use serde_json::Value;
use crate::test_utils;

// Helper function to create a test client
fn create_test_client() -> reqwest::Client {
    reqwest::Client::new()
}

"###);
    
    // Generate tests for each table
    for table_name in table_names {
        let kebab_case_name = to_kebab_case(table_name);
        
        test_content.push_str(&generate_table_error_tests(table_name, &kebab_case_name));
    }
    
    fs::write(&error_tests_file, test_content)?;
    
    Ok(())
}

fn generate_table_error_tests(table_name: &str, kebab_case_name: &str) -> String {
    format!(
        r###"#[tokio::test]
async fn test_{}_database_error_handling() {{
    // Setup
    let _server_handle = test_utils::start_test_server().await.expect("Failed to start test server");
    let client = create_test_client();
    let base_url = "http://localhost:8082";
    
    // Test database connection error handling
    // This would require mocking or simulating a database connection failure
    // TODO: Implement database error simulation and testing
}}

#[tokio::test]
async fn test_{}_invalid_json_payload() {{
    // Setup
    let _server_handle = test_utils::start_test_server().await.expect("Failed to start test server");
    let client = create_test_client();
    let base_url = "http://localhost:8082";
    
    // Test invalid JSON payload for POST requests
    let invalid_json_response = client
        .post(&format!("{{}}/add_{{}}", base_url, "{}"))
        .body("{{ invalid json }}")
        .header("Content-Type", "application/json")
        .send()
        .await
        .expect("Failed to send invalid JSON request");
    
    // Should return a 400 Bad Request
    assert_eq!(invalid_json_response.status(), 400);
}}

#[tokio::test]
async fn test_{}_missing_required_fields() {{
    // Setup
    let _server_handle = test_utils::start_test_server().await.expect("Failed to start test server");
    let client = create_test_client();
    let base_url = "http://localhost:8082";
    
    // Test missing required fields in JSON payload
    let missing_fields_response = client
        .post(&format!("{{}}/add_{{}}", base_url, "{}"))
        .json(&serde_json::json!({{ /* Intentionally missing required fields */ }}))
        .send()
        .await
        .expect("Failed to send missing fields request");
    
    // Should return a 400 Bad Request or similar error
    // assert_eq!(missing_fields_response.status(), 400);
}}

"###,
        table_name.to_lowercase(),
        table_name.to_lowercase(),
        kebab_case_name,
        table_name.to_lowercase(),
        kebab_case_name
    )
}

fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect()
}

fn to_kebab_case(s: &str) -> String {
    s.replace("_", "-")
}
