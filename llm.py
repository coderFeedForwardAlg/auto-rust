import sys
from transformers import AutoModelForCausalLM, AutoTokenizer
import re
import os
from openai import OpenAI
from dotenv import load_dotenv
load_dotenv()
api_key = os.getenv("OPENAI_API_KEY")

if not api_key:
    raise ValueError("OPENAI_API_KEY not found in environment variables. Please create a .env file.")

client = OpenAI(api_key=api_key)

def get_completion(prompt):
    """
    Sends a prompt to the OpenAI API and returns the response.
    """
    try:
        response = client.chat.completions.create(
            model="gpt-3.5-turbo",
            messages=[
                {"role": "user", "content": prompt}
            ]
        )
        return response.choices[0].message.content
    except Exception as e:
        return f"An error occurred: {e}"



def main():
    if len(sys.argv) < 2:
        print("Error: Please provide a prompt as a command-line argument.", file=sys.stderr)
        sys.exit(1)
        
    prompt = sys.argv[1]
    completion = get_completion(prompt)        
   
    print(completion)
if __name__ == "__main__":
    main()
