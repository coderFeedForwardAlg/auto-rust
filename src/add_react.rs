use std::process::{Command};
use std::io::{self};
use std::path::Path;

pub fn create_react_app<P: AsRef<Path>>(directory: P) -> io::Result<()> {
    let dir = directory.as_ref();
    println!("Creating a new React application in '{}' directory...", dir.display());
    
    if !dir.exists() {
        std::fs::create_dir_all(dir)?;
    }

    let status = Command::new("cp")
    .arg("-r")
    .arg("./frontend")
    .arg(dir.display().to_string())
    .status()?;
    if status.success() {
        println!("\n✅ Successfully created React application in '{}'!", dir.display());
    } else {
        eprintln!("\n❌ Failed to create React application in '{}'", dir.display());
    }

    Ok(())
}