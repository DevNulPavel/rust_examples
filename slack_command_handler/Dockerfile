# Сборка с помощью пакета Rust
# https://hub.docker.com/_/rust
FROM rust:1.48 as builder
WORKDIR /usr/src/slack_command_handler
COPY . .
RUN cargo build --release

# Сборка рабочего пакета
# Test run: docker run -it --rm debian:9-slim
# apt-get install libssl1.1 -y
FROM debian:9-slim
RUN apt-get update
COPY --from=builder \
    /usr/src/slack_command_handler/target/release/slack_command_handler \
    /usr/local/bin/slack_command_handler
CMD ["slack_command_handler"]