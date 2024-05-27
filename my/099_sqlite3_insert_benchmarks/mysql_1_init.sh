#! /usr/bin/env bash

export PATH=/opt/homebrew/Cellar/mysql/8.0.32/bin:$PATH

rm -rf ./mysql_db/

mkdir ./mysql_db/

# --user=devnul
# --database=test_db 
mysqld --initialize-insecure --datadir=$(pwd)/mysql_db/

du -h -d 0 mysql_db/
