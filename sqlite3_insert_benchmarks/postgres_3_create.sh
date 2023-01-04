#! /usr/bin/env bash

export PATH=/opt/homebrew/opt/postgresql@15/bin:$PATH

createdb -h 127.0.0.1 -p 5432 bench_test

du -h -d 0 postgres_db/

# rm -rf postgres_db/