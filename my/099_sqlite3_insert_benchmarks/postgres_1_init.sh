#! /usr/bin/env bash

export PATH=/opt/homebrew/opt/postgresql@15/bin:$PATH

rm -rf ./postgres_db/

initdb ./postgres_db/

du -h -d 0 postgres_db/

# rm -rf postgres_db/