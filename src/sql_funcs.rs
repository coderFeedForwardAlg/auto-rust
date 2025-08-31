
use crate::base_structs::Row;
use std::{fs::OpenOptions, io::{self, Write}};
use crate::add_functions;
use crate::base_structs::create_type_map;
use convert_case::{Case, Casing};

fn generate_struct(row: &Row, file_path: &std::path::Path) -> Result<(), std::io::Error> {
    // Ensure parent directories exist
    if let Some(parent) = file_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    
    let type_map = create_type_map();
    let struct_name = row.name.to_case(Case::Pascal); // Convert table name to PascalCase
    let mut struct_string = format!("#[derive(Debug, Serialize, Deserialize, FromRow)]\nstruct {} {{\n", struct_name);

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


pub fn add_basic_sql_funcs(
    rows: Vec<Row>,
    path: &std::path::Path,
    func_names: &mut Vec<String>
) -> Result<(), io::Error> {


   // re do each one to have layers and return the endpoint layer (api layer)  
    for row in rows {
        generate_struct(&row, &path)?;
        func_names.push(add_functions::add_insert_func(&row, &path)?);
        func_names.push(add_functions::add_get_all_func(&row, &path)?);
        for col in &row.cols {
            func_names.push(add_functions::add_get_one_func(&row, col, &path)?);
        }
    }
    Ok(())
}