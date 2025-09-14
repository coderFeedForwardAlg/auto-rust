use std::process::{Command, Stdio};
use std::io::{self, Write};
use std::path::Path;

pub fn create_react_app<P: AsRef<Path>>(directory: P) -> io::Result<()> {
    let dir = directory.as_ref();
    println!("Creating a new React application in '{}' directory...", dir.display());
    
    // Create the directory if it doesn't exist
    if !dir.exists() {
        std::fs::create_dir_all(dir)?;
    }

    // // Execute npx create-react-app in the specified directory
    // let status = Command::new("npx")
    //     .current_dir(dir)
    //     .args(["create-react-app@latest", ".", "--template", "typescript"])
    //     .stdout(Stdio::inherit())
    //     .stderr(Stdio::inherit())
    //     .status()?;

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