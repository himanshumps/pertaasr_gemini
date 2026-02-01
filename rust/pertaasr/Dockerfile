FROM rust:1-trixie as builder
USER 1001
WORKDIR /usr/src/app
USER 1001
ENV LANG='en_US.UTF-8'
ENV RUST_BACKTRACE=1
COPY Cargo.toml ./
COPY ./src ./src
RUN RUSTFLAGS="-C target-cpu=native" cargo build --release

FROM rust:1-trixie
USER 1001
WORKDIR /usr/src/app
USER 1001
COPY --chown=1001:1001 --chmod=0777 --from=builder /usr/src/app/target/release /usr/src/app/target/release
CMD "./target/release/pertaasr"
