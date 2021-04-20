# Каждая строка кода - это новый слой, который кешируется.
# Если ранний кеш сбросился, сбрасываются и остальные
# Поэтому нужно располагать команды от редко изменяемых, к часто изменяемым.
# Это нужно для того, чтобы сломанный кеш не увеличивал время сборки

FROM rust:1.51.0-alpine as builder
USER root
WORKDIR /usr/src/pocket_telegram_bot
COPY . .
RUN apk update
RUN apk add --no-cache libc-dev
RUN cargo build --release

FROM alpine:3.13.5
COPY --from=builder \
    /usr/src/pocket_telegram_bot/target/release/pocket_telegram_bot \
    /usr/local/bin/pocket_telegram_bot
CMD ["pocket_telegram_bot"]