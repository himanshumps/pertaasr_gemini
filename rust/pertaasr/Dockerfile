FROM rust:1-trixie
WORKDIR /usr/src/app
ENV RUST_BACKTRACE=1
COPY Cargo.toml ./
RUN mkdir -p src/bin && echo "fn main() {}" > src/main.rs
RUN cargo build --release
COPY ./src ./src
RUN cargo build --release
CMD "./target/release/pertaasr"
