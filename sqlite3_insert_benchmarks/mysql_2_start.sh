#! /usr/bin/env bash

export PATH=/opt/homebrew/Cellar/mysql/8.0.32/bin:$PATH

# --user=mysql 
# --database=test_db
mysqld --datadir=$(pwd)/mysql_db/
