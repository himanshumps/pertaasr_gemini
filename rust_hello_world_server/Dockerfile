# Stage 1: Build the real application
FROM rust:1-slim-bookworm AS builder
WORKDIR /usr/src/app

# Only copy what is needed for dependency resolution first to leverage caching
COPY Cargo.toml ./
# Pre-build dependencies with a dummy file
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release

# Copy the real source and build the final binary
COPY ./src ./src
# Use 'touch' to ensure cargo sees the file change from the dummy
RUN touch src/main.rs && cargo build --release

# Stage 2: Runtime image
FROM debian:bookworm-slim
# Create a non-root user
RUN useradd -m -u 1001 appuser
USER 1001
WORKDIR /usr/src/app

# Copy the actual binary from the builder
COPY --from=builder /usr/src/app/target/release/rust_rest_server /usr/src/app/server

# Ensure the binary has execution permissions
USER root
RUN chmod +x /usr/src/app/server
USER 1001

# Expose the port used by Actix
EXPOSE 8080

# Run the server in the foreground
CMD ["/usr/src/app/server"]
