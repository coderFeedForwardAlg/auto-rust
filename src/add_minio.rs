


pub fn add_minio(file_path: &std::path::Path) -> Result<String, io::Error> {
    // Ensure parent directories exist
    if let Some(parent) = file_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    
    
    
    
    
    
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

