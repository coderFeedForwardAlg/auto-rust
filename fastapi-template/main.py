import os
import httpx
from typing import List, Dict, Any, Optional
from pathlib import Path
from dotenv import load_dotenv
from fastapi import FastAPI, UploadFile, File, HTTPException
from fastapi.responses import JSONResponse
from pydantic import BaseModel
import chromadb
from chromadb.config import Settings
from chromadb.utils import embedding_functions
from sentence_transformers import SentenceTransformer
from typing import List, Dict, Any
import uuid

# Load environment variables from .env file
load_dotenv()

# Initialize FastAPI app
app = FastAPI(title="RAG Chat API")

# Initialize ChromaDB client
chroma_client = chromadb.Client(Settings(
    chroma_db_impl="duckdb+parquet",
    persist_directory=".chromadb"  # Directory where data will be stored
))

# Initialize embedding function
embedding_function = embedding_functions.SentenceTransformerEmbeddingFunction(
    model_name="all-MiniLM-L6-v2"
)

# Create or get the collection
collection_name = "documents"
try:
    collection = chroma_client.get_collection(
        name=collection_name,
        embedding_function=embedding_function
    )
except ValueError:
    # Create the collection if it doesn't exist
    collection = chroma_client.create_collection(
        name=collection_name,
        embedding_function=embedding_function
    )

class Document(BaseModel):
    text: str
    metadata: Optional[Dict[str, Any]] = None

class Query(BaseModel):
    text: str
    n_results: int = 3

def get_chat_response(prompt: str, context: str = "") -> str | None:
    """
    Send a message to the OpenAI API and get a response, optionally using RAG context.
    
    Args:
        prompt: The user's message/prompt
        context: Additional context from document retrieval
        
    Returns:
        The assistant's response as a string
    """
    # Initialize the OpenAI client
    api_key = os.getenv("OPENAI_API_KEY")
    if not api_key:
        return "Error: 'OPENAI_API_KEY' not found in environment variables"
    
    client = OpenAI(api_key=api_key, http_client=httpx.Client())
    
    try:
        # Prepare the system message with context if available
        system_message = "You are a helpful assistant."
        if context:
            system_message += f"\n\nUse the following context to answer the question. If you don't know the answer, say you don't know.\n\nContext:\n{context}"
        
        # Create a chat completion
        response = client.chat.completions.create(
            model="gpt-3.5-turbo",
            messages=[
                {"role": "system", "content": system_message},
                {"role": "user", "content": prompt}
            ]
        )
        
        # Extract and return the assistant's reply
        return response.choices[0].message.content
        
    except Exception as e:
        return f"An error occurred: {str(e)}"

# Document Management Endpoints
@app.post("/documents/")
async def add_document(document: Document):
    """Add a single document to the knowledge base."""
    try:
        doc_id = str(uuid.uuid4())
        collection.add(
            documents=[document.text],
            metadatas=[document.metadata or {}],
            ids=[doc_id]
        )
        chroma_client.persist()
        return {"status": "success", "id": doc_id}
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))

@app.post("/documents/upload/")
async def upload_document(file: UploadFile = File(...)):
    """Upload and process a text file."""
    if not file.filename.endswith('.txt'):
        raise HTTPException(status_code=400, detail="Only .txt files are supported")
    
    try:
        content = await file.read()
        text = content.decode('utf-8')
        doc_id = str(uuid.uuid4())
        
        collection.add(
            documents=[text],
            metadatas=[{"filename": file.filename}],
            ids=[doc_id]
        )
        chroma_client.persist()
        return {"status": "success", "id": doc_id}
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))

@app.get("/documents/")
async def list_documents():
    """List all documents in the knowledge base."""
    try:
        results = collection.get()
        return {
            "documents": [
                {"id": id_, "metadata": meta} 
                for id_, meta in zip(results['ids'], results['metadatas'])
            ]
        }
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))

# Chat Endpoint with RAG
@app.post("/chat/")
async def chat(query: Query):
    """Chat endpoint with RAG functionality."""
    try:
        # First, retrieve relevant documents
        results = collection.query(
            query_texts=[query.text],
            n_results=min(query.n_results, 5)  # Limit to 5 results max
        )
        
        # Format the context from retrieved documents
        context = ""
        if results and 'documents' in results and results['documents']:
            context = "\n\n".join([doc for doc in results['documents'][0]])
        
        # Get response from LLM with context
        response = get_chat_response(query.text, context)
        
        return {
            "response": response,
            "context_used": bool(context)
        }
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8000)
