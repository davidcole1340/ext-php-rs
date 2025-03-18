FROM php:zts-bullseye

# Install rust
RUN apt-get update && apt-get install -y curl clang

# Add app user with uid 1000
RUN useradd -m -u 1000 app
USER app

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

ENV PATH="/home/app/.cargo/bin:${PATH}"
