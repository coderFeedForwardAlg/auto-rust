import os
import httpx
from openai import OpenAI
from dotenv import load_dotenv
from fastapi import FastAPI
from langchaing.document_loaders import DirectoryLoader

DATA_PATH = "data"

# Load environment variables from .env file
load_dotenv()

def load_documents() -> List[Document]:
    loader = DirectoryLoader(DATA_PATH, glob="*.pdf")
    documents = loader.load()
    return documents


def get_chat_response(prompt: str) -> str | None:
    """
    Send a message to the OpenAI API and get a response.
    
    Args:
        prompt: The user's message/prompt
        
    Returns:
        The assistant's response as a string
    """
    # Initialize the OpenAI client
    api_key = os.getenv("OPENAI_API_KEY")
    if not api_key:
        return "Error: OPENAI_API_KEY not found in environment variables"
    
    client = OpenAI(api_key=api_key, http_client=httpx.Client())
    
    try:
        # Create a chat completion
        response = client.chat.completions.create(
            model="gpt-3.5-turbo",
            messages=[
                {"role": "system", "content": "You are a helpful assistant."},
                {"role": "user", "content": prompt}
            ]
        )
        
        # Extract and return the assistant's reply
        return response.choices[0].message.content
        
    except Exception as e:
        return f"An error occurred: {str(e)}"


app = FastAPI()

@app.post("/chat")
async def chat(request: dict) -> str | None:
    if "prompt" not in request:
        return "Error: 'prompt' field is required"
    return get_chat_response(request["prompt"])

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8000)
