# Stage 1: Build the application
FROM rust:1-slim-trixie AS builder
WORKDIR /usr/src/app
COPY Cargo.toml ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
COPY ./src ./src
ENV RUST_BACKTRACE=1
RUN cargo build --release

FROM debian:trixie-slim
USER 1001
WORKDIR /usr/src/app
USER 1001
COPY --chown=1001:1001 --chmod=0755 --from=builder /usr/src/app/target/release/rust_rest_server /usr/src/app/target/release/rust_rest_server
CMD ["./target/release/rust_rest_server"]