#!/bin/bash -e

docker build -t devnul/armv6_openssl_musl -f Dockerfile_musl .
docker push devnul/armv6_openssl_musl

# docker build -t devnul/armv6_openssl_gnu -f Dockerfile_gnu .
# docker push devnul/armv6_openssl_gnu