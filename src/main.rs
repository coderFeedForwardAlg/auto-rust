mod schema;
mod gen_docker;
mod gen_sql;
mod gen_toml;
use gen_sql::gen_sql;
use std::collections::HashMap;
use std::fmt::format;
use std::fs::OpenOptions;
use std::io::{self, BufWriter};
use convert_case::{Case, Casing};
use serde::de::value::{self, Error};
use serde::Deserialize;
use sqlx::FromRow;
use std::io::Write;
pub use schema::{extract_column_info, extract_table_schemas, extract_table_names, Col};
use std::process::{Command, Output};
use gen_docker::gen_docker;
use crate::gen_toml::gen_toml;



#[derive(Debug)]
struct Row {
    name: String,
    cols: Vec<Col>,
}

// Helper function to insert multiple PostgreSQL types for a single Rust type
fn insert_multiple(map: &mut HashMap<String, String>, rust_type: &str, postgres_types: &[&str]) {
    for pg_type in postgres_types {
        map.insert(pg_type.to_string(), rust_type.to_string());
    }
}

fn create_type_map() -> HashMap<String, String> {
    let mut type_map = HashMap::new();

    insert_multiple(&mut type_map, "bool", &["BOOL"]);
    insert_multiple(&mut type_map, "i8", &["CHAR"]);
    insert_multiple(&mut type_map, "i16", &["SMALLINT", "SMALLSERIAL", "INT2"]);
    insert_multiple(&mut type_map, "i32", &["INT", "SERIAL", "INT4", "INTEGER"]);
    insert_multiple(&mut type_map, "i64", &["BIGINT", "BIGSERIAL", "INT8"]);
    insert_multiple(&mut type_map, "f32", &["REAL", "FLOAT4"]);
    insert_multiple(&mut type_map, "f64", &["DOUBLE PRECISION", "FLOAT8"]);
    insert_multiple(&mut type_map, "String", &["VARCHAR", "CHAR(N)", "TEXT", "NAME", "CITEXT"]);
    insert_multiple(&mut type_map, "Vec<u8>", &["BYTEA"]);
    insert_multiple(&mut type_map, "()", &["VOID"]);
    // insert_multiple(&mut type_map, "PgInterval", &["INTERVAL"]);
    // insert_multiple(&mut type_map, "PgMoney", &["MONEY"]);
    // insert_multiple(&mut type_map, "PgLTree", &["LTREE"]);
    // insert_multiple(&mut type_map, "PgLQuery", &["LQUERY"]);
    // insert_multiple(&mut type_map, "PgCiText", &["CITEXT1"]);
    // insert_multiple(&mut type_map, "PgCube", &["CUBE"]);
    // insert_multiple(&mut type_map, "PgPoint", &["POINT"]);
    // insert_multiple(&mut type_map, "PgLine", &["LINE"]);
    // insert_multiple(&mut type_map, "PgLSeg", &["LSEG"]);
    // insert_multiple(&mut type_map, "PgBox", &["BOX"]);
    // insert_multiple(&mut type_map, "PgPath", &["PATH"]);
    // insert_multiple(&mut type_map, "PgPolygon", &["POLYGON"]);
    // insert_multiple(&mut type_map, "PgCircle", &["CIRCLE"]);
    // insert_multiple(&mut type_map, "PgHstore", &["HSTORE"]);

    // Add the new pairs
    // type_map.insert("NUMERIC".to_string(), "bigdecimal::Decimal".to_string());
    type_map.insert("TIMESTAMPTZ".to_string(), "chrono::DateTime<Utc>".to_string());
    type_map.insert("TIMESTAMP".to_string(), "chrono::NaiveDateTime".to_string());
    type_map.insert("DATE".to_string(), "chrono::NaiveDate".to_string());
    type_map.insert("TIME".to_string(), "chrono::NaiveTime".to_string());
    type_map.insert("TIMETZ".to_string(), "PgTimeTz".to_string());
    type_map.insert("UUID".to_string(), "uuid::Uuid".to_string());
    insert_multiple(&mut type_map, "ipnetwork::IpNetwork", &["INET", "CIDR"]);
    insert_multiple(&mut type_map, "std::net::IpAddr", &["INET", "CIDR"]);
    insert_multiple(&mut type_map, "ipnet::IpNet", &["INET", "CIDR"]);
    insert_multiple(&mut type_map, "mac_address::MacAddress", &["MACADDR"]);
    insert_multiple(&mut type_map, "bit_vec::BitVec", &["BIT", "VARBIT"]);
    // insert_multiple(&mut type_map, "Json<T>", &["JSON", "JSONB"]); //  *******  TODO:fix ********* //
    insert_multiple(&mut type_map, "serde_json::Value", &["JSON", "JSONB"]);
    insert_multiple(&mut type_map, "&serde_json::value::RawValue", &["JSON", "JSONB"]);

    // Handle PgRange<T> types
    type_map.insert("INT8RANGE".to_string(), "PgRange<i64>".to_string());
    type_map.insert("INT4RANGE".to_string(), "PgRange<i32>".to_string());
    type_map.insert("TSRANGE".to_string(), "PgRange<PgTimestamp>".to_string()); // Assuming you have a PgTimestamp type
    type_map.insert("TSTZRANGE".to_string(), "PgRange<PgTimestampTz>".to_string()); // Assuming you have a PgTimestampTz type
    type_map.insert("DATERANGE".to_string(), "PgRange<PgDate>".to_string()); // Assuming you have a PgDate type
    type_map.insert("NUMRANGE".to_string(), "PgRange<PgNumeric>".to_string()); // Assuming you have a PgNumeric type


    type_map
}


fn generate_struct(row: &Row, file_path: &std::path::Path) -> Result<(), std::io::Error> {
    // Ensure parent directories exist
    if let Some(parent) = file_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    
    let type_map = create_type_map();
    let struct_name = row.name.to_case(Case::Pascal); // Convert table name to PascalCase
    let mut struct_string = format!("#[derive(Debug, Deserialize, FromRow)]\nstruct {} {{\n", struct_name);

    for col in &row.cols {
        let field_name = col.name.clone();
        let rust_type = type_map.get(&col.col_type)
            .map(|s| s.as_str())
            .unwrap_or("String"); // Default to String if type not found
        if col.auto_gen {
            struct_string.push_str(&format!("    {}: Option<{}>,\n",field_name, rust_type));
        } else {
            struct_string.push_str(&format!("    {}: {},\n", field_name, rust_type));
        }
    }

    struct_string.push_str("}\n");

    // Write the struct to the file
    // fs::write(file_path, struct_string)?;

    let mut file = OpenOptions::new()
        .write(true) // Enable writing to the file.
        .append(true) // Set the append mode.  Crucially, this makes it append.
        .create(true) // Create the file if it doesn't exist.
        .open(file_path)?; // Open the file, returning a Result.

    // Write the data to the file.
    file.write_all(struct_string.as_bytes())?; // comment for testing 
    Ok(())
}

fn create_rows_from_sql(file_path: &std::path::Path) -> Result<Vec<Row>, io::Error> {
    let table_names = extract_table_names(&file_path.display().to_string())?;
    let schemas = extract_table_schemas(&file_path.display().to_string())?;
    let mut rows: Vec<Row> = Vec::new();

    if table_names.len() != schemas.len() {
        eprintln!("Warning: Number of table names and schemas do not match!");
    }

    for (table_name, schema) in table_names.iter().zip(schemas.iter()) {
        let cleaned_name = table_name
            .split('.')
            .last()
            .unwrap_or(&table_name)
            .trim_matches('"')
            .to_string();
        let cols = extract_column_info(schema);
        //let cols = c.into_iter().filter(|col| {
            //  !col.auto_gen
        //}).collect::<Vec<_>>();
        let row = Row {
            name: cleaned_name,
            cols,
        };
        rows.push(row);
    }


    Ok(rows)
}


// id wil lcouse problems
fn add_insert_func(row: &Row, file_path: &std::path::Path) -> Result<String, io::Error> {
    // Ensure parent directories exist
    if let Some(parent) = file_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    
    let funk_name = format!("add_{}", row.name.clone());
    let struct_name = row.name.clone().to_case(Case::Pascal);
    let table_name = row.name.clone();
    let cols_list = row.cols.iter()
    .filter(|col| !col.auto_gen )
    .map(|col| { // filter based on if auto generated 
            col.name.clone()
    }).collect::<Vec<_>>();
    

    let cols: String = cols_list.iter().map(|col| format!("{}, ", col).to_string()).collect::<String>()
        .trim_end_matches(", ").to_string();
        let bind_feilds = cols_list.iter().enumerate().map(|(i, col)| 
        format!("\t.bind(payload.{})", cols_list[i]))
        //format!("payload.{}, ", cols_list[i]))
        .collect::<Vec<_>>().join("\n");
    let feilds = cols_list.iter().enumerate().map(|(i, col)| format!("${}, ", i + 1)).collect::<String>()
        .trim_end_matches(", ").to_string();
    let funk = format!(r###"

async fn {funk_name}(
    extract::State(pool): extract::State<PgPool>,
    Json(payload): Json<{struct_name}>,
) -> Json<Value> {{
    let query = "INSERT INTO {table_name} ({cols}) VALUES ({feilds}) RETURNING *";
    
    let q = sqlx::query_as::<_, {struct_name}>(&query)
        {bind_feilds};
    
    let result = q.fetch_one(&pool).await;

    match result {{
        Ok(value) => Json(json!({{"res": "sucsess" }})), // maybe bad code??
        Err(e) => Json(json!({{"res": format!("error: {{}}", e)}}))

    }}
}}
"###);


    let mut file = OpenOptions::new()
        .write(true) // Enable writing to the file.
        .append(true) // Set the append mode.  Crucially, this makes it append.
        .create(true) // Create the file if it doesn't exist.
        .open(file_path)?; // Open the file, returning a Result.



    file.write_all(funk.as_bytes())?; // comment for testing 

    Ok(funk_name.to_string())
}




fn add_get_one_func(row: &Row, col: &Col, file_path: &std::path::Path) -> Result<String, io::Error> {
    let type_map = create_type_map();
    let row_name = row.name.clone();
    let col_name = col.name.clone();
    let end_index = col.col_type.find('(').unwrap_or(col.col_type.len());
    let col_type = match type_map.get(&col.col_type[..end_index]) {
        Some(t) => t.to_string(),
        None => {
            eprintln!("Warning: Type '{}' not found in type map for column '{}'. Defaulting to String.", col.col_type, col_name);
            "f64".to_string() // Default to String if type not found
        }
    };
    let func_name = format!("get_one_{}{}", row.name.clone(), col_name.clone());
    let struct_name = row.name.clone().to_case(Case::Pascal);
    let cols: String = row.cols.iter().map(|col| format!("\t\"{}\": elemint.{}, \n", 
        col.name, col.name)
        .to_string()).collect::<String>()
        .trim_end_matches(", ").to_string();
    // let val = val.unwrap();
    // use match not res_json var
    let funk_str = format!(r###"
#[derive(Debug, Deserialize)]
struct {row_name}{col_name}Query {{
    {col_name}: {col_type},
}}

async fn {func_name}(
    extract::State(pool): extract::State<PgPool>,
    match_val: Query<{row_name}{col_name}Query>, // Assuming col is a path parameter
) -> Result<Json<Value>, (StatusCode, String)> {{
    let query = format!("SELECT * FROM {row_name} WHERE {col_name} = $1");
    let q = sqlx::query_as::<_, {struct_name}>(&query).bind(match_val.{col_name}.clone()); // todo: fugure out when .conle is needed

    let elemint = q.fetch_optional(&pool).await.map_err(|e| {{
        (StatusCode::INTERNAL_SERVER_ERROR, format!("Database err{{}}", e))
    }})?;

    match elemint {{
        Some(elemint) => Ok(Json(json!({{
            "payload": {{
                {cols}
            }}
        }}))),
        None => Err((StatusCode::NOT_FOUND, format!("No record found with {col_name} = the value"))),
    }}
}}
"###);

    let mut file = OpenOptions::new()
        .write(true) // Enable writing to the file.
        .append(true) // Set the append mode.  Crucially, this makes it append.
        .create(true) // Create the file if it doesn't exist.
        .open(file_path)?; // Open the file, returning a Result.

    file.write_all(funk_str.as_bytes())?; // comment for testing 


    Ok(func_name.to_string())
}

fn add_get_all_func(row: &Row, file_path: &std::path::Path) -> Result<String, io::Error> {
    // Ensure parent directories exist
    if let Some(parent) = file_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    
    let row_name = row.name.clone();
    let func_name = format!("get_{}", row.name.clone());
    let struct_name = row.name.clone().to_case(Case::Pascal);
    
    // Generate the JSON fields for the response
    let cols: String = row.cols.iter()
        .map(|col| format!("\t\"{}\": elemint.{}, \n", col.name, col.name))
        .collect::<String>()
        .trim_end_matches(", \n")
        .to_string();
    
    let funk_str = format!(r###"

async fn {func_name}(
    extract::State(pool): extract::State<PgPool>,
) -> Result<Json<Value>, (StatusCode, String)> {{
    let query = "SELECT * FROM {row_name}";
    let q = sqlx::query_as::<_, {struct_name}>(query);

    let elemints: Vec<{struct_name}> = q.fetch_all(&pool).await.map_err(|e| {{
        (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {{}}", e))
    }})?;

    let res_json: Vec<Value> = elemints.into_iter().map(|elemint| {{
        json!({{
    {cols}
        }})
    }}).collect();

    Ok(Json(json!({{ "payload": res_json }})))
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
    file.write_all(funk_str.as_bytes())?;

    Ok(func_name.to_string())
}

fn add_top_boilerplate(file_path: &std::path::Path) -> Result<(), io::Error> {
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
    extract::{self, Extension, Query},  
    routing::{get, post},                                                                                                                                                        
    Json, Router,                        
};                                                                                                                                    
use serde::Deserialize;                                                                                                                                                          
use serde_json::{json, Value};                                                                                                                                                   
use sqlx::PgPool;                                                                                                                                                                
use sqlx::{postgres::PgPoolOptions, prelude::FromRow};                                                                                                                           
use std::env;                                                                                                                                                                    
use std::net::SocketAddr;                                                                                                                                                        
use std::result::Result;                                                                                                                                                         
use std::sync::Arc;                                                                                                                                                              
use axum::http::StatusCode;                                                                                                                                                      
use sqlx::types::chrono::Utc; 
use tower_http::cors::{AllowOrigin, CorsLayer};
use axum::http::Method;
"###;
    file.write_all(top_boiler.as_bytes())?; // comment for testing 

    Ok(())
} 

fn add_axum_end(funcs: Vec<String>, file_path: &std::path::Path) -> Result<(), io::Error> {
    // Ensure parent directories exist
    if let Some(parent) = file_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    
    let mut file = OpenOptions::new()
        .write(true) // Enable writing to the file.
        .create(true) // Create the file if it doesn't exist.
        .append(true) // Set the append mode.
        .open(file_path)
        .map_err(|e| {
            eprintln!("Error opening file {}: {}", file_path.display(), e);
            e
        })?;
    let routs: String = funcs.iter().map(|func| {
        let http_method = if func.starts_with("get") { "get" } else { "post" };
        format!("\t.route(\"/{func}\", {http_method}({func}))\n").to_string()
    }).collect::<String>();
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

// todo: kick off postgress 
// https://users.rust-lang.org/t/how-to-execute-a-root-command-on-linux/50066/7
// docker run --name some-postgres -e POSTGRES_USER=dbuser -e POSTGRES_PASSWORD=p -e POSTGRES_DB=work -p 1111:5432 -d postgres
#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    // Get project name from user
    let mut file_name = String::new();
    println!("Enter project name: ");
    io::stdin().read_line(&mut file_name)?;
    let file_name = file_name.trim().to_string();
    
    // Save current directory
    let current_dir = std::env::current_dir()?;
    
    // Get the parent directory path
    let parent_dir = std::env::current_dir()?.parent()
        .ok_or_else(|| std::io::Error::new(
            std::io::ErrorKind::Other,
            "Cannot get parent directory"
        ))?.to_path_buf();
    
    let project_dir = parent_dir.join(&file_name);

    let _ = gen_toml(&project_dir, file_name.clone()).await;
    
    // Create new cargo project
    let output = Command::new("cargo")
        .current_dir(&parent_dir)
        .arg("new")
        .arg(&file_name)
        .output()?;
        
    if !output.status.success() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to create new project: {}", String::from_utf8_lossy(&output.stderr))
        ));
    }
    
    // Generate SQL and create necessary files
    println!("Generating SQL...");
    match gen_sql::gen_sql(project_dir.clone(), file_name.clone()).await {
        Ok(content) => {
            println!("Successfully generated SQL ({} bytes)", content.len());
            println!("SQL content preview: {}", content.chars().take(100).collect::<String>());
        },
        Err(e) => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to generate SQL: {}", e)
            ));
        }
    }
    
    // Change back to the original directory
    println!("Changing back to original directory: {:?}", current_dir);
    std::env::set_current_dir(&current_dir)?;
    
    // Process the generated SQL file
    let sql_path = project_dir.join("migrations/0001_data.sql");
    println!("Attempting to read SQL file from: {}", sql_path.display());
    
    // Verify file exists
    if !std::path::Path::new(&sql_path).exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("SQL file does not exist at: {}", sql_path.display())
        ));
    }
    
    let r = create_rows_from_sql(&sql_path);
    let rows = match r {
        Ok(rows) => {
            println!("Successfully parsed {} table definitions from SQL", rows.len());
            rows
        },
        Err(e) => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Error parsing SQL file at {}: {}", sql_path.display(), e)
            ));
        }
    };
    
    let path = project_dir.join("src/main.rs");
    let mut func_names = Vec::new();
    let toml_path = project_dir.join("Cargo.toml");
    // let _ = gen_toml(&toml_path, file_name.clone()).await;
    add_top_boilerplate(&path)?;
    for row in rows {
        println!("Row: {:?} \n", row);
        generate_struct(&row, &path)?;
        func_names.push(add_insert_func(&row, &path)?);
        func_names.push(add_get_all_func(&row, &path)?);
        for col in &row.cols {
            func_names.push(add_get_one_func(&row, col, &path)?);
        }
    }
    add_axum_end(func_names, &path)?;
    let docker_res = gen_docker(project_dir.file_name().expect("Failed to get file name").to_str().unwrap());
    match docker_res {
        Ok(_) => println!("Dockerfile created at {}", path.to_str().unwrap().to_owned() + "/Dockerfile"),
        Err(e) => eprintln!("Error creating Dockerfile: {}", e),
    }
    print!("docker run --name work -e POSTGRES_USER=dbuser   -e POSTGRES_PASSWORD=p   -e POSTGRES_DB=work   -p 1111:5432   -d postgres:latest");
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::io;
    use std::fs;

    #[test]
    fn test_extract_table_schemas() -> Result<(), io::Error> {
        let sql_content = r#"
        CREATE TABLE public."user" (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            favorite_color VARCHAR(50),
            height NUMERIC,
            age INTEGER,
            job VARCHAR(100)
        );

        CREATE TABLE product_details (
            product_id INTEGER PRIMARY KEY,
            description TEXT,
            price DECIMAL(10, 2)
        );


        create table order_items (
            order_id INTEGER,
            item_id INTEGER,
            quantity INTEGER
        );
        "#;
        fs::write("test.sql", sql_content)?;

        let expected_schemas = vec![
            "id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            favorite_color VARCHAR(50),
            height NUMERIC,
            age INTEGER,
            job VARCHAR(100)",
            "product_id INTEGER PRIMARY KEY,
            description TEXT,
            price DECIMAL(10, 2)",
            "order_id INTEGER,
            item_id INTEGER,
            quantity INTEGER",
        ];

        let schemas = extract_table_schemas("test.sql")?;
        assert_eq!(schemas.len(), expected_schemas.len());
        for (i, schema) in schemas.iter().enumerate() {
            assert_eq!(schema.trim(), expected_schemas[i].trim());
        }

        Ok(())
    }
}

// CICD plan 
// make a docker file that exposese port
// make docker compose yaml to start postgres (and volume), and rust (and exposse to internet)
//

// add ai to make desisions about what to add 
// * test ollama based on videos 
// * get function calling working 
// * use funciton calling to call functions to generate code 

// make call other arbitary apis like with requests.
// maybe function that takes in a url and schema struct and makes function that hits hits that url
//      with data in the structs format 
//   would consiter this working when can hit open ai api tools 


// at some point should ... 
// should add RTC streams,and sockets (will help for streaming llm stuff) 


// auto make unit tests for all functions


// add function to call ollama/apis  (can probably use comsom url in ollama_rs to hit open router endpoints) 

// call python code that writen in a python file (just in case)