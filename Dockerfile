# Build stage
FROM rust:1.83-bookworm as builder

WORKDIR /app

# First, copy only Cargo manifests to leverage Docker layer caching for dependencies
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs so that we can build and cache dependencies
RUN mkdir -p src && \
    echo 'fn main() { println!("dummy build"); }' > src/main.rs

# Build once to compile and cache dependencies
RUN cargo build --release

# Now remove the dummy source and copy the actual project files
RUN rm -rf src
COPY . .

# Build the real application in release mode
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install necessary runtime dependencies
RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Copy the compiled binary from builder
COPY --from=builder /app/target/release/chess-rust /usr/local/bin/chess-rust

# Expose port 3000
EXPOSE 3000

# Run the application
CMD ["chess-rust"]
