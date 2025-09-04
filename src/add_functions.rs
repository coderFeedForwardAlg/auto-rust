use std::fs::OpenOptions;
use std::io;
use std::io::BufWriter;
use std::io::Write;


use crate::schema::Col;
use convert_case::{Case, Casing};
use crate::base_structs;
use crate::create_type_map;
use crate::schema;


pub fn add_get_all_func(
    row: &base_structs::Row,
    file_path: &std::path::Path,
) -> Result<String, io::Error> {
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
    
    // API layer function - calls data layer and can add business logic
    let api_func = format!(r###"

#[derive(Deserialize)]
struct {row_name}QueryParams {{
    order_by: Option<String>,
    direction: Option<String>, // "asc" or "desc"
}}


pub async fn {func_name}(
    extract::State(pool): extract::State<PgPool>,
    Query(query_params): Query<{row_name}QueryParams>,
) -> Result<Json<Value>, (StatusCode, String)> {{
    // Call data function from data module 
    // Other business logic can also be handled here 
    let result = data_{func_name}(extract::State(pool), axum::extract::Query(query_params)).await;
    result
}}
"###);
// owned_str.push_str(borrows_str)
    // Data layer function - handles database operations
    let data_func = format!(r###"


pub async fn data_{func_name}(
    extract::State(pool): extract::State<PgPool>,
    query_params: axum::extract::Query<{row_name}QueryParams>,
) -> Result<Json<Value>, (StatusCode, String)> {{
    let mut query = "SELECT * FROM {row_name}".to_owned();
    // Validate and apply ordering if provided
    if let Some(order_by) = &query_params.order_by {{
        // Validate order_by column name to prevent SQL injection
        // Only allow alphanumeric characters and underscores
        if order_by.chars().all(|c| c.is_alphanumeric() || c == '_') {{
            // Validate direction parameter
            let direction = match &query_params.direction {{
                Some(dir) if dir.to_lowercase() == "desc" => "DESC",
                _ => "ASC",
            }};
            
            query.push_str(&format!(" ORDER BY {{}} {{}}", order_by, direction));
        }} else {{
            return Err((StatusCode::BAD_REQUEST, "Invalid order_by parameter".to_string()));
        }}
    }}

    let q = sqlx::query_as::<_, {struct_name}>(&query);

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

    // Write both functions to the same file
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(file_path)?;

    // Write API function first, then data function
    file.write_all(api_func.as_bytes())?;
    file.write_all(data_func.as_bytes())?;

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
    let bind_fields = cols_list.iter().enumerate().map(|(i, col)| 
        format!("\t\t.bind(payload.{})", cols_list[i]))
        .collect::<Vec<_>>().join("\n");
    let fields = cols_list.iter().enumerate().map(|(i, col)| format!("${}, ", i + 1)).collect::<String>()
        .trim_end_matches(", ").to_string();
    
    // API layer function - calls data layer and can add business logic
    let api_func = format!(r###"
pub async fn {funk_name}(
    extract::State(pool): extract::State<PgPool>,
    Json(payload): Json<{struct_name}>,
) -> Json<Value> {{
    // Call data function from data module 
    // Other business logic can also be handled here 
    let result = data_{funk_name}(extract::State(pool), Json(payload)).await;
    result
}}
"###);

    // Data layer function - handles database operations
    let data_func = format!(r###"
pub async fn data_{funk_name}(
    extract::State(pool): extract::State<PgPool>,
    Json(payload): Json<{struct_name}>,
) -> Json<Value> {{
    let query = "INSERT INTO {table_name} ({cols}) VALUES ({fields}) RETURNING *";
    
    let q = sqlx::query_as::<_, {struct_name}>(&query)
{bind_fields};
    
    let result = q.fetch_one(&pool).await;

    match result {{
        Ok(value) => Json(json!({{"res": "success", "data": value}})),
        Err(e) => Json(json!({{"res": format!("error: {{}}", e)}}))
    }}
}}
"###);

    // Write both functions to the same file
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(file_path)?;

    // Write API function first, then data function
    file.write_all(api_func.as_bytes())?;
    file.write_all(data_func.as_bytes())?;

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

    // Query struct definition
    let query_struct = format!(r###"
#[derive(Debug, Deserialize)]
struct {row_name}{col_name}Query {{
    {col_name}: {col_type},
}}
"###);

    // API layer function - calls data layer and can add business logic
    let api_func = format!(r###"
pub async fn {func_name}(
    extract::State(pool): extract::State<PgPool>,
    match_val: Query<{row_name}{col_name}Query>,
) -> Result<Json<Value>, (StatusCode, String)> {{
    // Call data function from data module 
    // Other business logic can also be handled here 
    let result = data_{func_name}(extract::State(pool), match_val).await;
    result
}}
"###);

    // Data layer function - handles database operations
    let data_func = format!(r###"
pub async fn data_{func_name}(
    extract::State(pool): extract::State<PgPool>,
    match_val: Query<{row_name}{col_name}Query>,
) -> Result<Json<Value>, (StatusCode, String)> {{
    let query = format!("SELECT * FROM {row_name} WHERE {col_name} = $1");
    let q = sqlx::query_as::<_, {struct_name}>(&query).bind(match_val.{col_name}.clone());

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

    // Write all parts to the same file
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(file_path)?;

    // Write query struct, API function, then data function
    file.write_all(query_struct.as_bytes())?;
    file.write_all(api_func.as_bytes())?;
    file.write_all(data_func.as_bytes())?;

    Ok(func_name.to_string())
}

