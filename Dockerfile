FROM eclipse-temurin:25-jdk-jammy as jdk

FROM jdk as java_build
USER root
RUN apt-get update && apt-get install -y curl --no-install-recommends && \
    rm -rf /var/lib/apt/lists/*
USER 1001
WORKDIR /app/java
USER 1001
ARG MAVEN_VERSION=3.9.12
ENV HOME=/app/java
RUN mkdir -p /tmp/maven && curl -o apache-maven-${MAVEN_VERSION}-bin.tar.gz https://dlcdn.apache.org/maven/maven-3/${MAVEN_VERSION}/binaries/apache-maven-${MAVEN_VERSION}-bin.tar.gz && \
    tar -xzf apache-maven-${MAVEN_VERSION}-bin.tar.gz --strip-components=1 -C /tmp/maven && rm apache-maven-${MAVEN_VERSION}-bin.tar.gz
ENV PATH=/tmp/maven/bin:$PATH
COPY --chown=1001:1001 --chmod=0777 ./java/pertaasr_supplier/pom.xml ./pertaasr_supplier/pom.xml
RUN mvn install -f ./pertaasr_supplier/pom.xml
COPY --chown=1001:1001 --chmod=0777 ./java/pertaasr_ffi/pom.xml ./pertaasr_ffi/pom.xml
RUN mvn install -f ./pertaasr_ffi/pom.xml
COPY --chown=1001:1001 --chmod=0777 ./java/pertaasr_supplier/src ./pertaasr_supplier/src
RUN mvn install -f ./pertaasr_supplier/pom.xml
COPY --chown=1001:1001 --chmod=0777 ./java/pertaasr_ffi/src ./pertaasr_ffi/src
RUN mvn install -f ./pertaasr_ffi/pom.xml


# Stage 1: Build the application
FROM rust:1-trixie AS rust_builder
WORKDIR /usr/src/app
COPY rust/pertaasr/Cargo.toml ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
COPY ./rust/pertaasr/src ./src
ENV RUST_BACKTRACE=1
RUN cargo build --release

FROM debian:trixie
USER root
RUN apt-get update && apt-get install -y libjemalloc2 libjemalloc-dev  --no-install-recommends && \
    rm -rf /var/lib/apt/lists/*
USER 1001
WORKDIR /usr/src/app
USER 1001
ENV HOME=/app/java
ENV LD_PRELOAD=/usr/lib/x86_64-linux-gnu/libjemalloc.so
ENV JEMALLOC_OVERRIDE=/usr/lib/x86_64-linux-gnu/libjemalloc.so
ENV LANG='en_US.UTF-8'
ENV JAVA_HOME=/opt/java/openjdk
COPY --from=jdk $JAVA_HOME $JAVA_HOME
ENV PATH="${PATH}:${JAVA_HOME}/bin"
COPY --chown=1001:1001 --chmod=0777 --from=rust_builder /usr/src/app/target/release/ /usr/src/app/target/release/
WORKDIR /app/java
COPY --chown=1001:1001 --chmod=0777 --from=java_build /app/java/pertaasr_ffi/target/ /app/java/pertaasr_ffi/target/
WORKDIR /usr/src/app/target/release/
USER 1001
ENV RUST_BACKTRACE=1
ENV CONNECTION_COUNT=20
ENV RUN_DURATION=120
ENV TOKIO_WORKER_THREADS=2
ENV GOMAXPROCS=2
CMD ["./pertaasr"]