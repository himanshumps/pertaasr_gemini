FROM rust:1-trixie as builder
USER root
RUN apt-get update && apt-get install -y libjemalloc2 libjemalloc-dev  --no-install-recommends && \
    rm -rf /var/lib/apt/lists/*
RUN mkdir -p /usr/src/app && chown 1001:0 /usr/src/app && chmod -R 777 /usr/src/app
USER 1001
WORKDIR /usr/src/app
USER 1001
ENV HOME=/usr/src/app
ENV LD_PRELOAD=/usr/lib/x86_64-linux-gnu/libjemalloc.so
ENV JEMALLOC_OVERRIDE=/usr/lib/x86_64-linux-gnu/libjemalloc.so
ENV LANG='en_US.UTF-8'
ENV RUST_BACKTRACE=1
COPY --chown=1001:1001 --chmod=0777 Cargo.toml ./
COPY --chown=1001:1001 --chmod=0777 ./src ./src
RUN cargo build --release

FROM rust:1-trixie
USER root
RUN apt-get update && apt-get install -y libjemalloc2 libjemalloc-dev  --no-install-recommends && \
    rm -rf /var/lib/apt/lists/*
USER 1001
WORKDIR /usr/src/app
USER 1001
ENV HOME=/usr/src/app
ENV LD_PRELOAD=/usr/lib/x86_64-linux-gnu/libjemalloc.so
ENV JEMALLOC_OVERRIDE=/usr/lib/x86_64-linux-gnu/libjemalloc.so
ENV LANG='en_US.UTF-8'
ENV RUST_BACKTRACE=1
USER 1001
COPY --chown=1001:1001 --chmod=0777 --from=builder /usr/src/app/target/release /usr/src/app/target/release
CMD "./target/release/pertaasr"
