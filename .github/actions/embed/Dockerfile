FROM php:8.3-bookworm

WORKDIR /tmp

RUN apt update -y && apt upgrade -y
RUN apt install lsb-release wget gnupg software-properties-common -y
RUN bash -c "$(wget -O - https://apt.llvm.org/llvm.sh)"

ENV RUSTUP_HOME=/rust
ENV CARGO_HOME=/cargo
ENV PATH=/cargo/bin:/rust/bin:$PATH

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --no-modify-path

ENTRYPOINT [ "/cargo/bin/cargo", "test", "--all", "--release", "--all-features", "--no-fail-fast" ]
