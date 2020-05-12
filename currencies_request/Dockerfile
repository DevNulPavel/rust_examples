# FROM alpine:3.7

# https://hub.docker.com/r/arm32v6/alpine
# Uses MUSL as libC 
# FROM arm32v6/alpine

# https://hub.docker.com/r/arm32v6/busybox
# MUSL
# FROM arm32v6/busybox:musl
# GNU
FROM arm32v6/busybox:1.31.1

# https://hub.docker.com/r/piersfinlayson/build
# FROM piersfinlayson/build:latest:armv6l

# FROM clux/muslrust
# FROM jenskeiner/muslrust-builder:develop-openssl-crypto-arm

ENV PATH=/bot:$PATH
ENV TELEGRAM_TOKEN=""

# Копия файлика
# MUSL
#COPY ./target/arm-unknown-linux-musleabi/release/telegram_bot /bot/telegram_bot
# GNU
COPY ./open_ssl_gnu_armv6_compiled/libcrypto.so.1.1 /lib/libcrypto.so.1.1
COPY ./open_ssl_gnu_armv6_compiled/libssl.so.1.1 /lib/libssl.so.1.1
COPY ./open_ssl_gnu_armv6_compiled/libcrypto.so.1.1 /bot/libcrypto.so.1.1
COPY ./open_ssl_gnu_armv6_compiled/libssl.so.1.1 /bot/libssl.so.1.1
COPY ./open_ssl_gnu_armv6_compiled/libcrypto.so.1.1 /bot/libcrypto.so
COPY ./open_ssl_gnu_armv6_compiled/libssl.so.1.1 /bot/libssl.so
COPY ./target/arm-unknown-linux-gnueabi/release/telegram_bot /bot/telegram_bot
COPY ./sql/create_database.sql /bot/create_database.sql
COPY ./bot_start.sh /bot/bot_start.sh

# Порт наружу
# EXPOSE 5000

# Список зависимостей для приложения 1
# ldd telegram_bot
#         /usr/lib/ld.so.1 (0xb6eea000)
#         libssl.so.1.1 => /lib/libssl.so.1.1 (0xb6e70000)
#         libcrypto.so.1.1 => /lib/libcrypto.so.1.1 (0xb6c8a000)
#         libc.musl-armhf.so.1 => /usr/lib/ld.so.1 (0xb6eea000)

# Список зависимостей для приложения 2
# -> readelf -d telegram_bot | grep 'NEEDED'
#  0x00000001 (NEEDED)                     Shared library: [libssl.so.1.1]
#  0x00000001 (NEEDED)                     Shared library: [libcrypto.so.1.1]
# + #  0x00000001 (NEEDED)                     Shared library: [libdl.so.2] 
# + #  0x00000001 (NEEDED)                     Shared library: [librt.so.1]
# + #  0x00000001 (NEEDED)                     Shared library: [libpthread.so.0]
# + #  0x00000001 (NEEDED)                     Shared library: [libgcc_s.so.1]
# + #  0x00000001 (NEEDED)                     Shared library: [libc.so.6]
# + #  0x00000001 (NEEDED)                     Shared library: [ld-linux.so.3]
# + #  0x00000001 (NEEDED)                     Shared library: [libm.so.6]

# Установка зависимостей
# apk search 
# gcc
# RUN apk update && \
#     apk add --no-cache bash openssl

WORKDIR /bot

# Запуск
CMD ["sh", "bot_start.sh"]

# RUN apk add --no-cache mysql-client
# ENTRYPOINT ["mysql"]
