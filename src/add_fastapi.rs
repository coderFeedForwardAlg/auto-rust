use fs_extra::dir::{copy, CopyOptions};
use std::{fmt::format, path::Path};
pub fn add_fastapi(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let from = Path::new("fastapi-template");
    let destination = format!("../{}", path);
    let to = Path::new(&destination);

    let options = CopyOptions::new(); // Use default options

    println!("Attempting to copy directory from {:?} to {:?}", from, to);

    // This function handles creating destination folders and copying all files/subdirectories


    let copy_res = copy(from, to, &options);

    match copy_res{
        Ok(_) => println!("copyed fast api worked "),
        Err(e) => println!("error happend: {}", e)
    }

    println!("✅ Directory copied successfully!");
    Ok(())
}