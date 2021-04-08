# Сборка с помощью пакета Rust
# https://hub.docker.com/_/rust
FROM rust:1.51.0 as builder
ENV SQLX_OFFLINE true
WORKDIR /usr/src/test58_oauth
COPY . ./
RUN cargo build --release

# Сборка рабочего пакета
FROM debian:10.9
WORKDIR /oauth_server
COPY --from=builder \
    /usr/src/test58_oauth/target/release/test58_oauth \
    test58_oauth
COPY --from=builder \
    /usr/src/test58_oauth/migrations \
    migrations
COPY --from=builder \
    /usr/src/test58_oauth/static \
    static
COPY --from=builder \
    /usr/src/test58_oauth/templates \
    templates
CMD ["./test58_oauth"]