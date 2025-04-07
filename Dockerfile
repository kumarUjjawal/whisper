# Use official Rust image as builder
FROM rust:1.81 as builder

# Create app directory
WORKDIR /app

# Copy the source and build dependencies first
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Build in release mode
RUN cargo build --release

# Use a smaller runtime image
FROM debian:bookworm-slim

# Install required libraries (if any)
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy compiled binary
COPY --from=builder /app/target/release/whisper ./app

# Expose your backend port (adjust if different)
EXPOSE 3000

# Run the binary
CMD ["app"]
