# Stage 1: Build the application
FROM rust:1.78.0-alpine AS builder
RUN apk add --no-cache musl-dev
WORKDIR /usr/src/app
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release --target x86_64-unknown-linux-musl
COPY ./src ./src
RUN cargo build --release --target x86_64-unknown-linux-musl

# Stage 2: Create a minimal production image
FROM alpine:latest
WORKDIR /usr/src/app
COPY --from=builder /usr/src/app/target/x86_64-unknown-linux-musl/release/rust_rest_server ./rust_rest_server
EXPOSE 8080
CMD ["./rust_rest_server"]