FROM rust:1-bookworm AS builder
RUN useradd -m -u 1001 appuser
USER 1001
WORKDIR /usr/src/app
USER 1001
ENV HOME=/usr/src/app
ENV LANG='en_US.UTF-8'
ENV RUST_BACKTRACE=1
COPY --chown=1001:1001 --chmod=0777 Cargo.toml ./
COPY --chown=1001:1001 --chmod=0777 ./src ./src
RUN cargo build --release

FROM debian:bookworm
RUN useradd -m -u 1001 appuser
USER 1001
WORKDIR /usr/src/app
USER 1001
ENV HOME=/usr/src/app
ENV LANG='en_US.UTF-8'
ENV RUST_BACKTRACE=1
COPY --chown=1001:1001 --chmod=0777 --from=builder /usr/src/app/target/release /usr/src/app/target/release
ENTRYPOINT ["/usr/src/app/target/release/pertaasr"]
