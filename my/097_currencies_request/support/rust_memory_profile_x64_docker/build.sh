#!/bin/bash -e

docker build -m 6g -t devnul/rust_memory_profile_x64_docker .
docker push devnul/rust_memory_profile_x64_docker