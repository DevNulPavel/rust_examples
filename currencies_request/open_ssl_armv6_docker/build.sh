#!/bin/bash -e

docker build -t devnul/armv6_openssl .
docker push devnul/armv6_openssl