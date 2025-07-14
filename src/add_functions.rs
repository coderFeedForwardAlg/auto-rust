use std::fs::OpenOptions;
use std::io;
use std::io::BufWriter;
use std::io::Write;


use convert_case::{Case, Casing};
use crate::base_structs;
use crate::create_type_map;
use crate::schema;
pub fn add_get_all_func(row: &base_structs::Row, file_path: &std::path::Path) -> Result<String, io::Error> {
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


pub fn add_insert_func(row: &base_structs::Row, file_path: &std::path::Path) -> Result<String, io::Error> {
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




pub fn add_get_one_func(row: &base_structs::Row, col: &schema::Col, file_path: &std::path::Path) -> Result<String, io::Error> {
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

