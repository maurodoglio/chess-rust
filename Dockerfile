# Build stage
FROM rust:1.82-bookworm as builder

WORKDIR /app

# Copy all source files
COPY . .

# Build the application in release mode
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
