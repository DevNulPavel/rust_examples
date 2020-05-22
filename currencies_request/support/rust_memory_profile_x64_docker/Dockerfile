# Вроде бы должно работать кеширование, если не запихивать все в один вызов RUN
# FROM ubuntu:20.04 as builder
FROM rust:latest as builder

ENV TZ=Europe/Moscow \
    DEBIAN_FRONTEND="noninteractive" 

RUN ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone
RUN apt-get update && \
    apt-get install nodejs npm tzdata curl git -y
RUN apt-get remove cmdtest && \
    apt-get remove yarn && \
    curl -sS https://dl.yarnpkg.com/debian/pubkey.gpg | apt-key add - && \
    echo "deb https://dl.yarnpkg.com/debian/ stable main" | tee /etc/apt/sources.list.d/yarn.list && \
    apt-get update && \
    apt-get install yarn

# RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

ENV RUST_BACKTRACE=1 \
    CARGO_PROFILE_DEV_DEBUG=0

WORKDIR /usr/src
RUN git clone https://github.com/koute/memory-profiler.git

# WORKDIR /usr/src/memory-profiler/webui
# RUN whereis yarn
# RUN yarn --version
# RUN yarn install

WORKDIR /usr/src/memory-profiler
RUN cargo fetch
RUN cargo build --release -p memory-profiler
# TODO: Не хватает памяти похоже для компиляции release
# RUN cargo build --release -p memory-profiler-cli
# Выставлено CARGO_PROFILE_DEV_DEBUG=0, чтобы бинарник был меньше по размеру
RUN cargo build -p memory-profiler-cli

#######################################################################################################################

FROM debian:buster-slim

# RUN apt-get update && \
#     apt-get install openssl libssl-dev -y

# TODO: Без кеша
RUN apt-get update && \
    apt-get install -y wget perl build-essential

RUN mkdir /tmp/openssl && \
    cd /tmp/openssl && \
    wget https://www.openssl.org/source/openssl-1.1.1g.tar.gz && \
    tar xzf openssl-1.1.1g.tar.gz && \
    rm openssl-1.1.1g.tar.gz && \
    mv openssl-1.1.1g sources

RUN export MACHINE=x86_64 && \
    export ARCH=x86_64 && \
    export CC=x86_64-linux-gnu-gcc && \
    export TARGET=x86_64-linux-gnu && \
    cd /tmp/openssl/sources && \
    rm -rf build && \
    mkdir build && \
    cd build && \
    ../Configure linux-x86_64 && \
    make -j16 && \
    make install

RUN rm -rf /tmp/openssl

RUN apt-get remove -y wget perl build-essential

# TODO: Не хватает памяти похоже для компиляции release
# COPY --from=builder /usr/src/memory-profiler/target/release/memory-profiler-cli /usr/local/bin/memory-profiler-cli
COPY --from=builder /usr/src/memory-profiler/target/debug/memory-profiler-cli /usr/local/bin/memory-profiler-cli
COPY --from=builder /usr/src/memory-profiler/target/release/libmemory_profiler.so /usr/local/lib/libmemory_profiler.so
