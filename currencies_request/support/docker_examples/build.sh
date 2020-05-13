#!/bin/bash -e

docker build --tag custom-compile:1.0 -f Dockerfile .
# docker build -o - . > out.tar
docker run --name bb custom-compile:1.0 > ./rpxc
chmod +x rpxc
# ./rpxc sh -c "cd /; pwd; ls -la /usr/local/;"
# ./rpxc ls -la /usr/local/
./rpxc -- cargo build --release --bin telegram_bot --target arm-unknown-linux-gnueabihf
rm rpxc
docker rm --force bb
# docker rm --force 3d2aa594cb154200205c3b5421748bf9ab8413f0c9a6b21d1be68b819682b3e7
# docker system prune -a

# docker run \
#     --volume $(pwd):/home/cross/project \
#     ragnaroek/rust-raspberry:1.43.0 \
#     build 