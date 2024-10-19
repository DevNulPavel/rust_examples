#!/bin/sh

CMD=$1

# Смещаем аргументы на 1 аргумент
shift

# Аналог match в bash
case ${CMD} in
  server)
    cargo run --release --features server,rpc --bin busrtd -- -B /tmp/busrt.sock \
      -B 0.0.0.0:9924 -B fifo:/tmp/busrt.fifo $*
    ;;
  cli)
    cargo run --release --bin busrt --features cli -- /tmp/busrt.sock $*
    ;;
  *)
    echo "command unknown: ${CMD}"
    ;;
esac
