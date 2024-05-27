#!/bin/bash -e

# docker build -t devnul/x64_linux_openssl_musl -f Dockerfile_musl .
# docker push devnul/x64_linux_openssl_musl

docker build -t devnul/x64_linux_openssl_gnu -f Dockerfile_gnu .
docker push devnul/x64_linux_openssl_gnu