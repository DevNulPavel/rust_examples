# FROM sdthirlwall/raspberry-pi-cross-compiler
FROM thewtex/cross-compiler-linux-armv6

# Install wget, openssl, libssl-dev, pkg-config
RUN apt-get update && \
    apt-get install -y wget openssl libssl-dev pkg-config curl
# Set env variables
ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH \
    RUST_VERSION=1.43.1 \
    ARMV6_LINUX_GNUEABIHF_OPENSSL_LIB_DIR=/tmp/openssl-1.1.0h \
    ARMV6_LINUX_GNUEABIHF_OPENSSL_INCLUDE_DIR=/tmp/openssl-1.1.0h/include
# Install rustc, rustup and cargo
RUN set -eux; \
    \
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs > rustup.sh; \
    sh rustup.sh -y --no-modify-path; \
    chmod -R a+xw $RUSTUP_HOME $CARGO_HOME; \
    rustup target add arm-unknown-linux-gnueabihf; \
    rustup --version; \
    cargo --version; \
    rustc --version;
# Download open-ssl from source and cross compiles it for arm7
RUN cd /tmp && \
    wget https://www.openssl.org/source/openssl-1.1.1g.tar.gz && \
    tar xzf openssl-1.1.1g.tar.gz && \
    export MACHINE=armv6 && \
    export ARCH=arm && \
    export CC=gcc && \
    cd openssl-1.1.1g && ./config shared && make -j8