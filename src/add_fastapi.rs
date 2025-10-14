use fs_extra::dir::{copy, CopyOptions};
use std::path::Path;
pub fn add_fastapi(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let from = Path::new("fastapi-template");
    let destination = format!("../{}", path);
    let to = Path::new(&destination);

    let options = CopyOptions::new(); // Use default options

    println!("Attempting to copy directory from {:?} to {:?}", from, to);

    // This function handles creating destination folders and copying all files/subdirectories


    copy(from, to, &options).map_err(|e| Box::<dyn std::error::Error>::from(e))?;
    println!("✅ Directory copied successfully!");
    Ok(())
}