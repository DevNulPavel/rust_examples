# FROM alpine:3.7

# https://hub.docker.com/r/arm32v6/alpine
# Uses MUSL as libC 
# FROM arm32v6/alpine

# https://hub.docker.com/r/arm32v6/busybox
# FROM arm32v6/busybox:musl

# https://hub.docker.com/r/piersfinlayson/build
# FROM piersfinlayson/build:latest:armv6l

# FROM clux/muslrust
FROM jenskeiner/muslrust-builder:develop-openssl-crypto-arm

ENV PATH=/bot:$PATH
ENV TELEGRAM_TOKEN=""

# Копия файлика
COPY ./target/arm-unknown-linux-musleabi/release/telegram_bot /bot/telegram_bot
COPY ./sql/create_database.sql /bot/create_database.sql
COPY ./start.sh /bot/start.sh

# Порт наружу
# EXPOSE 5000

# Список зависимостей для приложения
# -> readelf -d telegram_bot | grep 'NEEDED'
#  0x00000001 (NEEDED)                     Shared library: [libssl.so.1.1]
#  0x00000001 (NEEDED)                     Shared library: [libcrypto.so.1.1]
#  0x00000001 (NEEDED)                     Shared library: [libdl.so.2]
#  0x00000001 (NEEDED)                     Shared library: [librt.so.1]
#  0x00000001 (NEEDED)                     Shared library: [libpthread.so.0]
#  0x00000001 (NEEDED)                     Shared library: [libgcc_s.so.1]
#  0x00000001 (NEEDED)                     Shared library: [libc.so.6]
#  0x00000001 (NEEDED)                     Shared library: [ld-linux.so.3]
#  0x00000001 (NEEDED)                     Shared library: [libm.so.6]

# Установка зависимостей
# RUN apk add --no-cache bash openssl crypto ssl

WORKDIR /bot

# Запуск
CMD ["sh", "start.sh"]

# RUN apk add --no-cache mysql-client
# ENTRYPOINT ["mysql"]
