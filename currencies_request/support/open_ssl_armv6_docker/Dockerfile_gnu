FROM rustembedded/cross:arm-unknown-linux-gnueabi-0.2.0

# ENV PKG_CONFIG_ALLOW_CROSS=1
# ENV CC=arm-linux-gnueabi-gcc
# ENV TARGET=arm-unknown-linux-gnueabi
# ENV OPENSSL_DIR=/openssl/result

RUN apt-get install -y wget

RUN mkdir /armv6_openssl && \
    cd /armv6_openssl && \
    wget https://www.openssl.org/source/openssl-1.1.1g.tar.gz && \
    tar xzf openssl-1.1.1g.tar.gz && \
    rm openssl-1.1.1g.tar.gz && \
    mv openssl-1.1.1g sources

RUN apt-get remove -y wget

RUN export MACHINE=armv6 && \
    export ARCH=arm && \
    export CC=arm-linux-gnueabi-gcc && \
    export TARGET=arm-unknown-linux-gnueabi && \
    cd /armv6_openssl/sources && \
    mkdir build && \
    cd build && \
    ../config --prefix=/armv6_openssl/result && \
    make -j16 && \
    make install && \
    cd /armv6_openssl && \
    rm -rf sources