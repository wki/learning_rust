FROM arm64v8/debian:13-slim
COPY debian.sources /etc/apt/sources.list.d/debian.sources
RUN apt update -y; \
    apt install -y build-essential pkg-config libclang-dev curl libcamera0.5 libcamera-dev; \
    cd $HOME; \
    echo "Downloading RustUp..."; \
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs > /tmp/rustup.sh; \
    chmod a+x /tmp/rustup.sh; \
    /tmp/rustup.sh -y; \
    echo "done."
