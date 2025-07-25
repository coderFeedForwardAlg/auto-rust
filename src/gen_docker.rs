use std::{fs::OpenOptions, io::Write, fs::File};


pub fn gen_docker(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    print!("{}\n", path);
let docker = format!(
"
# -----------------------------------------------------------------------------
#  Multi-stage Dockerfile for \"testing\" Rust / Axum API (production)
# -----------------------------------------------------------------------------
# 1. Builder image: compile the binary in release mode
# -----------------------------------------------------------------------------
FROM rustlang/rust:nightly-slim AS builder

# Install build dependencies that some crates (e.g. sqlx / openssl) may need
RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config libssl-dev ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create app directory inside the container
WORKDIR /app

# Cache dependencies first – copy manifest files only
COPY Cargo.toml Cargo.lock ./

# Set the default toolchain to nightly
RUN rustup default nightly

# Dummy main to build dependency layers and speed up subsequent builds
# RUN echo \"fn main() {{}}\" > src/main.rs \
#     && cargo build --release \
#     && rm -rf src

# Copy the actual source tree and build the real binary
COPY . .
RUN cargo build --release

# -----------------------------------------------------------------------------
# 2. Runtime image: copy the binary into a minimal base image
# -----------------------------------------------------------------------------
FROM debian:bookworm-slim AS runtime

# Install certificates (TLS) & clean apt caches
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create an unprivileged user to run the app
RUN useradd -m -u 10001 appuser

WORKDIR /app

# Copy compiled binary & any runtime assets (e.g. migrations)
COPY --from=builder /app/target/release/{path} ./
COPY --from=builder /app/migrations ./migrations

# Ensure the binary is executable
RUN chown -R appuser:appuser /app && chmod +x /app/{path}

USER appuser

# The application listens on port 8081 – expose it to the host
EXPOSE 8081

# Start the server
CMD [\"/app/{path}\"]

");
  // Create the directory if it doesn't exist
  //std::fs::create_dir_all(path)?;
  
  let dockerfile_path = format!("../{}/Dockerfile", path);
  let mut file = File::create(&dockerfile_path)?;
  file.write_all(docker.as_bytes())?;
  
  println!("Dockerfile created at {}", dockerfile_path);
  Ok(())
}

// docker build -t pangolin-testing .
