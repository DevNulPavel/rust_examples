# Сборка с помощью пакета Rust
# https://hub.docker.com/_/rust
FROM rust:1.56.1 as builder
WORKDIR /usr/src/file_upload_proxy
COPY ./src/ ./src/
COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock
RUN \
    ls -la && \
    cargo build --release

# Сборка рабочего пакета
FROM debian:11.1
WORKDIR /file_upload_proxy
COPY --from=builder \
    /usr/src/file_upload_proxy/target/release/file_upload_proxy \
    file_upload_proxy
CMD ["./file_upload_proxy -vv"]