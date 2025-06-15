
pub fn gen_docker() -> String {
let docker = format!(
"
version: '3.8'

services:
  backend:
    build:
      context: ./testing 
      dockerfile: Dockerfile
    ports:
      - \"8081:8081\"
    environment:
      - NEXT_PUBLIC_API_URL=/api
      - FRONT_END_URL=http://backend:8000
    depends_on:
      - postgres 
    restart: unless-stopped

");
docker
}

// docker build -t pangolin-testing .