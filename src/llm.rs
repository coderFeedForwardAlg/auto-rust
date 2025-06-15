
use ollama_rs::Ollama;

use ollama_rs::generation::completion::request::GenerationRequest;
pub async fn llm() -> String {
 
    let ollama = Ollama::default();

    let model = "starcoder2:latest".to_string();
    let prompt = "Why is the sky blue?".to_string();
    let res = ollama.generate(GenerationRequest::new(model, prompt)).await;
    res.unwrap().response.to_string()
}
