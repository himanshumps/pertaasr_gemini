FROM rust:1-trixie
USER 1001
WORKDIR /usr/src/app
USER 1001
ENV LANG='en_US.UTF-8'
ENV RUST_BACKTRACE=1
COPY Cargo.toml ./
RUN mkdir -p src/bin && echo "fn main() {}" > src/main.rs
RUN cargo build --release
COPY ./src ./src
RUN cargo build --release
CMD "./target/release/pertaasr"
