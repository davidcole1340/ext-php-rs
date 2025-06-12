FROM rust:latest AS base
ARG PHP_VERSION=8.4
WORKDIR /tmp
RUN <<EOF
set -e

apt update -y
apt install -y \
  libclang-dev \
  bison \
  re2c

# Build PHP
git clone --depth 1 -b PHP-${PHP_VERSION} https://github.com/php/php-src.git
cd php-src
# by default you will be on the master branch, which is the current
# development version. You can check out a stable branch instead:
./buildconf
./configure \
    --enable-debug \
    --disable-all --disable-cgi
make -j "$(nproc)"
make install
EOF

FROM base AS docsrs_bindings_builder
WORKDIR /src
RUN rustup component add rustfmt
RUN --mount=type=bind,target=/src,rw <<EOF
set -e
cargo clean
cargo build
cp target/debug/build/ext-php-rs-*/out/bindings.rs /docsrs_bindings.rs
rustfmt /docsrs_bindings.rs
EOF
ENTRYPOINT ["/generate.sh"]

FROM scratch AS docsrs_bindings
COPY --from=docsrs_bindings_builder /docsrs_bindings.rs /
