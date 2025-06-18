
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
    Ok(reqwest::get(format!("https://wttr.in/{city}?format=%C+%t"))
        .await?
        .text()
        .await?)
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
        .add_tool(get_weather);
        
    let user_message = ChatMessage::user("what is the weather in seattle".to_owned());
    let resp = coordinator.chat(vec![user_message]).await?;
    Ok(resp.message.content.to_string())
}
