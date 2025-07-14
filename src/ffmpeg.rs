use std::{fs::{File, OpenOptions}, io::Write};

#[macro_export]
macro_rules! make_ffmpeg_function {
    ( $( $x:expr ),* ) => {
        {
        let mut y = String::new();
        $(
            y.push_str(&format!("{}", $x.to_string()));
        )*
        y
        }
    }
}
pub async fn gen_ffmpeg(project_dir: &std::path::PathBuf, params: Vec<String>) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let function = "hi";
    let x = make_ffmpeg_function!({1 + 2}, {2 + 2});
    println!("{}", x.to_string());
 
    let mut file = OpenOptions::new()
        .write(true) // Enable writing to the file.
        .append(true) // Set the append mode.  Crucially, this makes it append.
        .create(true) // Create the file if it doesn't exist.
        .open(project_dir.join("ffmpeg.rs"))?; // Open the file, returning a Result.

    file.write_all(function.as_bytes()).map_err(|e| {
        eprintln!("Error writing to file: {}", e);
        e
    })?;
    Ok(function.to_string())
}
    



