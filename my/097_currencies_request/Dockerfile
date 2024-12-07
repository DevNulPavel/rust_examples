# FROM alpine:3.7

# https://hub.docker.com/r/arm32v6/alpine
# Uses MUSL as libC 
FROM arm32v6/alpine:latest

# https://hub.docker.com/r/arm32v6/busybox
# MUSL
# FROM arm32v6/busybox:musl
# GNU
# FROM arm32v6/busybox:1.31.1

# https://hub.docker.com/r/piersfinlayson/build
# FROM piersfinlayson/build:latest:armv6l

# FROM clux/muslrust
# FROM jenskeiner/muslrust-builder:develop-openssl-crypto-arm

ENV PATH=/bot:$PATH \
    TELEGRAM_DATABASE_PATH="/bot/database/telegram_bot.sqlite" \
    TELEGRAM_TOKEN="" \
    RUST_BACKTRACE=1 \
    RUST_LOG=telegram_bot=trace

ENV TZ Europe/Moscow

# MUSL valid libraries paths
# Список зависимостей для приложения 1
# https://stackoverflow.com/questions/34729748/installed-go-binary-not-found-in-path-on-alpine-linux-docker
# https://stackoverflow.com/a/35613430/6021994
# 
# Не хватает библиотеки ld в musl сборке - нужно создать символьную ссылку
# $ ldd telegram_bot
#         /usr/lib/ld.so.1 (0xb6e7f000)
#         libssl.so.1.1 => /lib/libssl.so.1.1 (0xb6e05000)
#         libcrypto.so.1.1 => /lib/libcrypto.so.1.1 (0xb6c1f000)
#         libc.musl-armhf.so.1 => /usr/lib/ld.so.1 (0xb6e7f000)
# 
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
# 
# https://www.ibm.com/developerworks/ru/library/l-lpic1-v3-102-3/index.html
# Для запуска приложения можно вызвать /lib/ld-musl-armhf.so.1 telegram_bot
# - для получения списка зависимостей /lib/ld-musl-armhf.so.1 --list telegram_bot
# - для проверки зависимостей /lib/ld-linux.so.3 --verify /bin/bash
# так как на самом деле - это не библиотека, а бинарник
RUN ln -s /lib/ld-musl-armhf.so.1 /usr/lib/ld.so.1

# Копия файлика
# MUSL
COPY ./target/arm-unknown-linux-musleabi/release/telegram_bot /bot/telegram_bot

# GNU
# COPY ./open_ssl_gnu_armv6_compiled/libcrypto.so.1.1 /lib/libcrypto.so.1.1
# COPY ./open_ssl_gnu_armv6_compiled/libssl.so.1.1 /lib/libssl.so.1.1
# COPY ./open_ssl_gnu_armv6_compiled/libcrypto.so.1.1 /bot/libcrypto.so.1.1
# COPY ./open_ssl_gnu_armv6_compiled/libssl.so.1.1 /bot/libssl.so.1.1
# COPY ./open_ssl_gnu_armv6_compiled/libcrypto.so.1.1 /bot/libcrypto.so
# COPY ./open_ssl_gnu_armv6_compiled/libssl.so.1.1 /bot/libssl.so
# COPY ./target/arm-unknown-linux-gnueabi/release/telegram_bot /bot/telegram_bot

# Копия ресурсов
COPY ./sql/create_database.sql /bot/sql/create_database.sql
COPY ./support/bot_start.sh /bot/bot_start.sh

# Порт наружу
# EXPOSE 5000

# Установка зависимостей
# apk search 
# gcc
RUN apk update && \
    apk add --no-cache openssl

WORKDIR /bot

# Запуск
CMD ["sh", "bot_start.sh"]

# RUN apk add --no-cache mysql-client
# ENTRYPOINT ["mysql"]
