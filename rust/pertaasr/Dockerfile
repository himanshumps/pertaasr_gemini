FROM rust:1-trixie
WORKDIR /usr/src/app
COPY Cargo.toml ./
RUN mkdir -p src/bin && echo "fn main() {}" > src/main.rs && echo "fn main() {}" > src/bin/reqwest_test.rs
RUN cargo build --release
COPY ./src ./src
ENV RUST_BACKTRACE=1
RUN RUSTFLAGS="-C target-cpu=native" cargo build --release
CMD "RUSTFLAGS=\"-C target-cpu=native\" cargo run --release --bin reqwest_test"
