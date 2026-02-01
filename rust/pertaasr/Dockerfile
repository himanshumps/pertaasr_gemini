FROM rust:1-trixie
USER root
RUN apt-get update && apt-get install -y libjemalloc2 libjemalloc-dev  --no-install-recommends && \
    rm -rf /var/lib/apt/lists/*
USER 1001
WORKDIR /usr/src/app
USER 1001
ENV LD_PRELOAD=/usr/lib/x86_64-linux-gnu/libjemalloc.so
ENV JEMALLOC_OVERRIDE=/usr/lib/x86_64-linux-gnu/libjemalloc.so
ENV LANG='en_US.UTF-8'
ENV RUST_BACKTRACE=1
COPY Cargo.toml ./
RUN mkdir -p src/bin && echo "fn main() {}" > src/main.rs
RUN cargo build --release
COPY ./src ./src
RUN cargo build --release
CMD "./target/release/pertaasr"
