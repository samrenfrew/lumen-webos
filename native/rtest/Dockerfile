FROM debian:bookworm-slim
ARG RUST
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl gcc libc6-dev libsdl2-dev libsdl2-image-dev libsdl2-ttf-dev libgles2-mesa-dev libgl1-mesa-dri xvfb xauth fonts-dejavu-core && rm -rf /var/lib/apt/lists/*
ENV RUSTUP_HOME=/opt/rustup CARGO_HOME=/opt/cargo PATH=/opt/cargo/bin:$PATH
RUN curl -fsSL https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain "$RUST" --target armv7-unknown-linux-gnueabi && chmod -R a+rX /opt/rustup /opt/cargo
