FROM rust:1.92

RUN apt-get update && apt-get install -y \
    cmake \
    libclang-dev \
    && rm -rf /var/lib/apt/lists/*

# Taskfile support
RUN curl -1sLf 'https://dl.cloudsmith.io/public/task/task/setup.deb.sh' | bash
RUN apt-get install -y task

RUN rustup component add clippy rustfmt
