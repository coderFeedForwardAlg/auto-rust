
use std::{fs::{self, File}, io::Write};

use ollama_rs::{coordinator::Coordinator, generation::chat::ChatMessage, Ollama};

/// Interacts with the Ollama LLM to process SQL generation requests.
///
/// This function sets up a coordinator with the Ollama model and
/// generates SQL based on the provided prompt.
///
/// # Arguments
///
/// * `project_dir` - The directory where the project is located
/// * `file_name` - The name of the project
///
/// # Returns
///
/// Returns a `Result` containing the generated SQL as a string, or an error if the operation fails.
pub async fn gen_sql(project_dir: std::path::PathBuf, file_name: String, sql_task: String) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let model = "llama3.2:latest".to_string();
    
    let ollama = Ollama::default();
    let history = vec![];
    let mut coordinator = Coordinator::new(ollama, model, history);
        
    let prompt = format!(r#"you are a postgresSQL database designer. Here is how you should write postgres SQL code to define a database.
    
    Tables should be defined with CREATE TABLE IF NOT EXISTS. 
    Only use these datatypes: 
    - BOOL, CHAR, SMALLINT, SMALLSERIAL, INT2, INT, SERIAL, INT4, BIGINT, 
    - BIGSERIAL, INT8, REAL, FLOAT4, DOUBLE PRECISION, FLOAT8, VARCHAR, 
    - CHAR(N), TEXT, NAME, CITEXT, BYTEA, VOID, INTERVAL, 
    - INT8RANGE, INT4RANGE, TSRANGE, TSTZRANGE, DATERANGE, 
    - TIMESTAMPTZ, TIMESTAMP, DATE, TIME, TIMETZ, 
    - UUID, INET, CIDR, MACADDR, BIT, VARBIT, JSON, JSONB

    Rules:
    - Use UNIQUE where necessary (inline, not at the bottom of the table)
    - Use gen_random_uuid() when using UUIDs
    - Don't use NUMERIC, instead use INT or FLOAT
    - Don't use table names like `public.\"user\"`
    - All tables should have a UUID primary key that auto-increments
    - Don't use any comments
    - Output only the sql code, nothing else.

    Example:
    If I say "define a postgresSQL database that stores work sessions for users. 
    Each user has a start time, duration, break time, and a user. Each user has an email and a name. 
    Each work session has exactly one user and each user can have many work sessions."

    You should output:
    
    CREATE TABLE IF NOT EXISTS users (
        user_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        email VARCHAR(255) UNIQUE NOT NULL,
        name VARCHAR(255)
    );

    CREATE TABLE IF NOT EXISTS work_sessions (
        work_session_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        user_id UUID NOT NULL REFERENCES users(user_id),
        start_time TIMESTAMPTZ NOT NULL,
        duration_seconds INT NOT NULL,
        break_duration_seconds INT NOT NULL DEFAULT 0
    );

    Example 2:
    if i say "define a postgresSQL database that stores users and runs. 
    a user has a name, email, and favoret shoe. 
    a run has a user, and started at date/time, and distance and a duration. 
    each run should have exactly one user, but a user can have many runs. 
    output only the sql code, nothing else."
    
    you should output:
    
    CREATE TABLE IF NOT EXISTS users (
        user_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        email VARCHAR(255) UNIQUE NOT NULL,
        name VARCHAR(255)
    );

    CREATE TABLE IF NOT EXISTS runs (
        run_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        user_id UUID NOT NULL REFERENCES users(user_id),
        start_time TIMESTAMPTZ NOT NULL,
        distance_km FLOAT NOT NULL,
        duration_seconds INT NOT NULL
    );

    

    now {}"#, sql_task);

    let user_message = ChatMessage::user(prompt.to_owned());
    let resp = coordinator.chat(vec![user_message]).await?;
    
    let migrations_dir = project_dir.join("migrations");
    let sql_path = migrations_dir.join("0001_data.sql");
    
    println!("Creating SQL file at: {}", sql_path.display());
    
    // Create parent directories
    println!("Creating directory: {}", migrations_dir.display());
    fs::create_dir_all(&migrations_dir).map_err(|e| {
        eprintln!("Error creating directory: {}", e);
        e
    })?;
    
    // Create and write to the file
    println!("Creating file: {}", sql_path.display());
    let mut file = File::create(&sql_path).map_err(|e| {
        eprintln!("Error creating file: {}", e);
        e
    })?;

    let content = resp.message.content.to_string();
    println!("Writing SQL content ({} bytes)", content.len());
    file.write_all(content.as_bytes()).map_err(|e| {
        eprintln!("Error writing to file: {}", e);
        e
    })?;
    Ok(resp.message.content.to_string())
}
