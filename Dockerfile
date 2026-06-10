# =============================================================================
# KairosOS Docker Builder Image
# Provides: Rust toolchain, kernel headers, Python, build utilities
# =============================================================================
FROM debian:bookworm-slim AS base

ENV DEBIAN_FRONTEND=noninteractive
ENV RUSTUP_HOME=/opt/rustup
ENV CARGO_HOME=/opt/cargo
ENV PATH="/opt/cargo/bin:${PATH}"

RUN apt-get update && apt-get install -y \
    bash bc binutils bison build-essential bzr ca-certificates cmake cpio \
    curl debianutils file flex g++ gcc git gperf graphviz kmod \
    libelf-dev libncurses-dev libssl-dev \
    linux-headers-amd64 \
    lz4 make openssl patch perl \
    python3 python3-pip python3-dev python3-venv \
    rsync sed tar unzip vim wget xz-utils zlib1g-dev zstd \
    gcc-aarch64-linux-gnu libc6-dev-arm64-cross \
    && rm -rf /var/lib/apt/lists/*

# Rust toolchain (stable)
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --no-modify-path \
    --default-toolchain stable --profile minimal
RUN rustup target add x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu wasm32-wasi
RUN rustup component add clippy rustfmt
RUN wget -q https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-24/wasi-sdk-24.0-x86_64-linux.tar.gz \
    && tar xzf wasi-sdk-24.0-x86_64-linux.tar.gz -C /opt \
    && rm wasi-sdk-24.0-x86_64-linux.tar.gz
ENV WASI_SDK=/opt/wasi-sdk-24.0-x86_64-linux
ENV WASI_CC=${WASI_SDK}/bin/clang

# Python virtualenv with test deps
RUN python3 -m venv /opt/venv
ENV PATH="/opt/venv/bin:${PATH}"
RUN pip install --upgrade pip setuptools wheel && \
    pip install ruff mypy pytest pytest-asyncio pytest-timeout

# Shell check
RUN apt-get update && apt-get install -y shellcheck && rm -rf /var/lib/apt/lists/*

# Verify installations
RUN cargo --version && rustc --version && python3 --version

WORKDIR /build
CMD ["/bin/bash"]
