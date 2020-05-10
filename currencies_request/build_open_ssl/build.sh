#!/bin/bash -e

rm -rf ./build
mkdir -p build
cd build
wget https://www.openssl.org/source/openssl-1.1.0h.tar.gz
tar xzf openssl-1.1.0h.tar.gz
export MACHINE=armv6
export ARCH=arm
export CC=gcc
cd openssl-1.1.0h 
./config shared
make -j 16