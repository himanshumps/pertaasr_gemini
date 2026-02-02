FROM registry.access.redhat.com/ubi10/ubi:latest
USER root
RUN yum install -y rust-toolset 
RUN dnf install -y https://dl.fedoraproject.org/pub/epel/epel-release-latest-10.noarch.rpm && \
    dnf clean all

# 2. Install jemalloc from the newly enabled EPEL repository
RUN dnf install -y jemalloc jemalloc-devel && \
    dnf clean all

# 3. Configure the environment to use jemalloc as the default allocator
# The library is typically located at /usr/lib64/libjemalloc.so.2 on RHEL-based systems
ENV LD_PRELOAD="/usr/lib64/libjemalloc.so.2"
ENV JEMALLOC_OVERRIDE="/usr/lib64/libjemalloc.so.2"
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
