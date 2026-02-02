FROM rust:1-trixie as builder
USER 1001
WORKDIR /usr/src/app
USER 1001
ENV HOME=/usr/src/app
COPY Cargo.toml ./
COPY ./src ./src
RUN cargo build --release

FROM rust:1-trixie
USER 1001
WORKDIR /usr/src/app
USER 1001
ENV HOME=/usr/src/app
USER 1001
COPY --chown=1001:1001 --chmod=0777 --from=builder /usr/src/app/target/release /usr/src/app/target/release
CMD "./target/release/pertaasr"
