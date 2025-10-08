
mod llm;
mod schema;
mod gen_docker;
mod gen_sql;
mod gen_toml;
mod add_functions;
mod add_tests;
mod base_structs;
mod sql_funcs;
mod add_compose;
mod add_object;
mod add_minio;
mod boilerplate;
mod add_react;
mod gen_examples;


use gen_examples::gen_examples;
use add_react::create_react_app;
use gen_toml::gen_toml;
use add_minio::add_minio;
use add_object::add_object;
use add_compose::add_compose;
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
use boilerplate::{add_axum_end, add_top_boilerplate};
pub use base_structs::{Row, create_type_map};
pub use sql_funcs::add_basic_sql_funcs;
use std::net::{TcpListener, SocketAddr};


// This function is now in base_structs.rs
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


// todo: kick off postgress 
// https://users.rust-lang.org/t/how-to-execute-a-root-command-on-linux/50066/7
// docker run --name some-postgres -e POSTGRES_USER=dbuser -e POSTGRES_PASSWORD=p -e POSTGRES_DB=work -p 1111:5432 -d postgres
#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let mut file_name = String::new();
    println!("Enter project name: ");
    io::stdin().read_line(&mut file_name)?;
    let file_name = file_name.trim().to_string();
    
    let parent_dir = std::env::current_dir()?.parent()
        .ok_or_else(|| std::io::Error::new(
            std::io::ErrorKind::Other,
            "Cannot get parent directory"
        ))?.to_path_buf();
    
    let project_dir = parent_dir.join(&file_name);
    println!("Project directory: {}", project_dir.display());
    println!("Parent directory: {}", parent_dir.display());

    
    // Create new cargo project
    let output = Command::new("cargo")
        .current_dir(&parent_dir)
        .arg("new")
        .arg(&file_name)
        .output()?;

 
    if !output.status.success() {
        eprintln!("Failed to create new project: {}", String::from_utf8_lossy(&output.stderr));
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to create new project: {}", String::from_utf8_lossy(&output.stderr))
        ));
    }


    let gen_toml_res = gen_toml::gen_toml(&project_dir).await;
    match gen_toml_res {
        Ok(_) => println!("Successfully generated TOML"),
        Err(e) => eprintln!("Failed to generate TOML: {}", e),
    };
           
    // Generate SQL and create necessary files
    let mut sql_task = String::new();
    println!("Enter the specific task for the SQL database (e.g., 'make SQL to store users and their favored food'): ");
    io::stdin().read_line(&mut sql_task)?;
    let mut sql_task = sql_task.trim().to_string();
    if sql_task == "" {
        sql_task = "make a database to track infomation about hosts and renters for an airBnB like aplication. there are hosts that have a zip code, name, email, and password hash. there are also renters that have all the same colums expet the zip code.".to_string();
        println!("using default test string");
    }

    match gen_sql::gen_sql(project_dir.clone(), file_name.clone(), sql_task).await {
        Ok(content) => {
            println!("Successfully generated SQL ({} bytes)", content.len());
        },
        Err(e) => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to generate SQL: {}", e)
            ));
        }
    }
    
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
    add_top_boilerplate(&path)?;
    

    // TODO: rename, this creates select all, select one, and add functions. 
    add_basic_sql_funcs(rows, &path , &mut func_names)?;
    println!("function names after basic sql are {:?}", func_names);

    // TODO: this looks like a dublicat of the add_minio function 
    // add_object(&path);
    add_axum_end(func_names.clone(), &path)?;
    let docker_res = gen_docker(project_dir.file_name().expect("Failed to get file name").to_str().unwrap());
    match docker_res {
        Ok(_) => println!("Dockerfile created at {}", project_dir.to_str().unwrap().to_owned()),
        Err(e) => eprintln!("Error creating Dockerfile: {}", e),
    }
    println!("function names after axum end are {:?}", func_names);
    let compose = add_compose(project_dir.file_name().expect("Failed to get file name").to_str().unwrap());
    match compose {
        Ok(_) => println!("Docker compose created at {}", project_dir.to_str().unwrap().to_owned()),
        Err(e) => eprintln!("Error creating Docker compose: {}", e),
    }
    let minio = add_minio(&project_dir.join("src/main.rs"));
    match minio {
        Ok(_) => println!("Minio added at {}", project_dir.to_str().unwrap().to_owned()),
        Err(e) => eprintln!("Error adding Minio: {}", e),
    }

    let _ = create_react_app("../".to_owned() + project_dir.file_name().expect("Failed to get file name").to_str().unwrap());

    let gen_examples_res = gen_examples(&project_dir.file_name().expect("Failed to get file name").to_str().unwrap(), func_names.clone());
    println!("function names after gen examples are {:?}", func_names); 
    match gen_examples_res {
        Ok(_) => println!("Examples generated at {}", project_dir.to_str().unwrap().to_owned()),
        Err(e) => eprintln!("Error generating examples: {}", e),
    }
    let addr: SocketAddr = "0.0.0.0:8081".parse().unwrap();
    match TcpListener::bind(&addr) {
        // If the bind operation is successful, it means the port was available.
        Ok(listener) => {
            println!("✅ Port 8081 is NOT in use.");
            // It's important to explicitly drop the listener to free up the port immediately.
            // This allows the program to exit cleanly.
            drop(listener);
        }
        // If the bind operation fails, an error is returned.
        // We can inspect the error kind to determine if the port is already in use.
        Err(e) => {
            // A common error is `AddrInUse`, which indicates the port is already taken.
            if e.kind() == std::io::ErrorKind::AddrInUse {
                println!("❌ Port 8081 is already in uses!!!!!!!!!!!!!!");
                println!("!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!");
                println!("!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!");
                println!("!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!");
                println!("!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!");
                println!("!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!");
                println!("!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!");
            } else {
                // Handle other potential errors, such as permissions issues.
                eprintln!("An unexpected error occurred: {}", e);
                println!("!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!");
                println!("!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!");
                println!("!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!");
                println!("!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!");
                println!("!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!");
                println!("!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!");
                println!("!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!");


            }
        }
    }
    Ok(())
}


// need to: 
// re-facter 
// minio for more than just text 
// use sql-gen crate 
// get rid of port mapings (besided 8081) they are not needed, jsut use :minio or :backend
// ** curent code uses minio:9000 and but should change this
// ** https://gemini.google.com/app/61d9393cfe723e22?is_sa=1&is_sa=1&android-min-version=301356232&ios-min-version=322.0&campaign_id=bkws&utm_source=sem&utm_source=google&utm_medium=paid-media&utm_medium=cpc&utm_campaign=bkws&utm_campaign=2024enUS_gemfeb&pt=9008&mt=8&ct=p-growth-sem-bkws&gclsrc=aw.ds&gad_source=1&gad_campaignid=22908443171&gclid=Cj0KCQjw5c_FBhDJARIsAIcmHK8DwmYDLpVH8zs9IJmb2i1lSZtVT5NVUQvPOMa7tcObjfkuMQJdX3kaAsNBEALw_wcB




// CICD plan 
// make a docker file that exposese port
// make docker compose yaml to start postgres (and volume), and rust (and exposse to internet)
//

// add ai to make desisions about what to add 
// * test ollama based on videos 
// * get function calling working 
// * use funciton calling to call functions to generate code 
// combin stuff with joins and filtering 

// make call other arbitary apis like with requests.
// maybe function that takes in a url and schema struct and makes function that hits hits that url
//      with data in the structs format 
//   would consiter this working when can hit open ai api tools 


// at some point should ... 
// should add RTC streams,and sockets (will help for streaming llm stuff) 


// auto make unit tests for all functions


// add function to call ollama/apis  (can probably use comsom url in ollama_rs to hit open router endpoints) 
// * maybe do langchain in another container that cals rust?
//  could be slow thought if using network between containers
// * or have langchain run in a proces kicked off my rust. 
//  actor based model to comunicate between procesis 




// call python code that writen in a python file (just in case)
