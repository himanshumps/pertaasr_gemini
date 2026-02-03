# Use a minimal Debian base image
FROM debian:trixie-slim

# Install necessary build tools and dependencies for wrk
RUN apt-get update && apt-get install -y \
    build-essential \
    git \
    curl \
    make \
    gcc \
    lua5.1 \
    libssl-dev \
    unzip \
    --no-install-recommends && \
    rm -rf /var/lib/apt/lists/*

# Clone the wrk repository and build it
WORKDIR /usr/local/src/wrk
RUN git -c http.sslVerify=false clone https://github.com/wg/wrk.git . && make

# Put the compiled binary in a directory that is in the PATH
RUN cp wrk /usr/local/bin

# Set the entrypoint to the wrk executable
ENTRYPOINT ["wrk"]
