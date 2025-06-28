use std::process::Command;

use std::path::Path;

use ollama_rs::{coordinator::Coordinator, generation::chat::ChatMessage, Ollama};


#[ollama_rs::function]
/// Get the CPU temperature in Celsius.
///
/// This is a mock function that returns a hardcoded temperature value.
/// In a real implementation, this would read from system sensors.
///
/// # Returns
///
/// Returns a `Result` containing the temperature as a string in Celsius.
async fn get_cpu_temperature() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    // TODO: Implement actual CPU temperature reading
    Ok("42.7".to_string())
}


#[ollama_rs::function]
/// Get the weather for a given city.
///
/// # Arguments
///
/// * `city` - The name of the city to get the weather for.
///
/// # Returns
///
/// Returns a `Result` containing the weather information as a string.
async fn get_weather(city: String) -> Result<String, Box<dyn std::error::Error + Sync + Send>> {
    // Ok(reqwest::get(format!("https://wttr.in/{city}?format=%C+%t"))
    //     .await?
    //     .text()
    //     .await?)
    Ok("50 degrees c".to_string())
}


#[ollama_rs::function]
/// A function that cuts a segment of a video and saves that segment 
/// 
/// # Arguments
/// 
/// * `input_path` - a refrence to the name of the file where the input video can be found
/// * `output_path` - the name of the file where the resulting video will be saved
/// * `strt_time` - the time in the imput video to begin the cut
/// * `end_time` - the time the cut should end
///
/// # Returns
///
/// Returns a `Result` that can be ok or an error.
async fn cut_video_segment(input_path: String, output_path: String, start_time: String, end_time: String) -> Result<(), Box<dyn std::error::Error>> {
    if !Path::new(&input_path).exists() {
        return Err(format!("Input file {} does not exist", input_path).into());
    }

    // Move -ss before -i for more accurate seeking
    let status = Command::new("ffmpeg")
        .args([
            "-ss", &start_time,
            "-i", &input_path,
            "-to", &end_time,
            "-c:v", "libx264",
            "-c:a", "aac",
            "-avoid_negative_ts", "1",
            &output_path
        ])
        .status()
        .map_err(|e| format!("Failed to execute ffmpeg: {}", e))?;

    if status.success() {
        Ok("okay".into())
    } else {
        Err("FFmpeg process failed".into())
    }
}




/// Interacts with the Ollama LLM to process queries.
///
/// This function sets up a coordinator with the Ollama model and
/// demonstrates tool usage by asking about CPU temperature.
///
/// # Returns
///
/// Returns a `Result` containing the response string, or an error if the operation fails.
pub async fn llm() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let model = "llama3.2:latest".to_string();
    
    let ollama = Ollama::default();
    let history = vec![];
    let mut coordinator = Coordinator::new(ollama, model, history)
        .add_tool(get_cpu_temperature)
        .add_tool(get_weather)
        .add_tool(cut_video_segment);
        
    // Ok("ran".to_string())
    let user_message = ChatMessage::user("
    cut some part of the src/videos/test1.mp4 video".to_owned());
    let resp = coordinator.chat(vec![user_message]).await?;
    Ok(resp.message.content.to_string())
}
