FROM registry.access.redhat.com/ubi10/ubi:latest
USER root
RUN yum install -y rust-toolset 
RUN mkdir -p /usr/src/app && chown 1001:0 /usr/src/app && chmod -R 777 /usr/src/app
USER 1001
WORKDIR /usr/src/app
USER 1001
ENV HOME=/usr/src/app
ENV LANG='en_US.UTF-8'
ENV RUST_BACKTRACE=1
COPY --chown=1001:1001 --chmod=0777 Cargo.toml ./
COPY --chown=1001:1001 --chmod=0777 ./src ./src
RUN cargo build --release
CMD "./target/release/pertaasr"
