use std::{fs::OpenOptions, io::Write, fs::File};


pub fn add_compose(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    print!("{}\n", path);
let compose = format!(
"

version: '3.8'

services:
  nginx:
    image: nginx:alpine
    ports:
      - \"3002:80\"
    volumes:
      - ./nginx/nginx.conf:/etc/nginx/nginx.conf:ro
    depends_on:
      - app
      - frontend

  db:
    image: postgres:15-alpine
    environment:
      POSTGRES_USER: dbuser
      POSTGRES_PASSWORD: p
      POSTGRES_DB: data
    # ports:
      # - \"1111:5432\"
    volumes:
      - postgres_data:/var/lib/postgresql/data
    healthcheck:
      test: [\"CMD-SHELL\", \"pg_isready -U dbuser -d data\"]
      interval: 5s
      timeout: 5s
      retries: 10

  minio:
    image: minio/minio:latest
    command: server /data --console-address \":9001\"
    environment:
      MINIO_ROOT_USER: minioadmin
      MINIO_ROOT_PASSWORD: minioadmin
    # ports:
      # - \"9000:9000\"  # API port
      # - \"9001:9001\"  # Console port
    volumes:
      - minio_data:/data
    healthcheck:
      test: [\"CMD\", \"curl\", \"-f\", \"http://localhost:9000/minio/health/live\"]
      interval: 30s
      timeout: 20s
      retries: 3
  
  createbuckets:
    image: minio/mc
    depends_on:
      - minio
    entrypoint: >
      /bin/sh -c \"
      sleep 10;
        /usr/bin/mc alias set myminio http://minio:9000 minioadmin minioadmin || exit 1;
        /usr/bin/mc mb myminio/bucket || true;
        /usr/bin/mc anonymous set public myminio/bucket || exit 1;
        echo 'Bucket creation completed successfully';
      exit 0;
      \"

  app:
    build:
      context: .
      dockerfile: Dockerfile
    environment:
      DATABASE_URL: \"postgres://dbuser:p@db:5432/data\"
      DATABASE_CONNECT_TIMEOUT: \"30\"
      SQLX_OFFLINE: \"true\"
    depends_on:
      db:
        condition: service_healthy
    restart: on-failure
    healthcheck:
      test: [\"CMD\", \"wget\", \"--spider\", \"http://localhost:8081/health\"]
      interval: 10s
      timeout: 5s
      retries: 5
      start_period: 30s


  frontend:
    build:
      context: ./frontend
    depends_on:
      - app

volumes:
  postgres_data:
  minio_data:


"
);
  // Create the directory if it doesn't exist
  //std::fs::create_dir_all(path)?;
  
  let compose_path = format!("../{}/docker-compose.yaml", path);
  let mut file = File::create(&compose_path)?;
  file.write_all(compose.as_bytes())?;
  
  println!("compose created at {}", compose_path);

  // nginx 

  let nginx = format!(
    "events {{}}

http {{
    server {{
        listen 80;

        location / {{
            proxy_pass http://frontend:3000;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
        }}

        location /api/ {{
            proxy_pass http://app:8081/;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
        }}

        location /python/ {{
            proxy_pass http://python:8003/;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
        }}
    }}
}}"
  );

  let nginx_path = format!("../{}/nginx/nginx.conf", path);
  std::fs::create_dir_all(format!("../{}/nginx", path))?;
  let mut file = File::create(&nginx_path)?;
  file.write_all(nginx.as_bytes())?;
  
  println!("nginx created at {}", nginx_path);

  Ok(())
}

// docker build -t pangolin-testing .
