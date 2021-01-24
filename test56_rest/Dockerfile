# TODO: На ARM Macbook пока не работает Docker, поэтому требются адаптации путей в build.rs

# Сборка с помощью пакета Rust
# https://hub.docker.com/_/rust
FROM rust:1.49 as builder
WORKDIR /usr/src/test56_rest
COPY . .
RUN cargo build --release

# Сборка рабочего пакета
FROM debian:9-slim
RUN apt-get update
RUN apt-get install imagemagic
COPY --from=builder \
    /usr/src/test56_rest/target/release/test56_rest \
    /usr/local/bin/test56_rest
CMD ["test56_rest"]