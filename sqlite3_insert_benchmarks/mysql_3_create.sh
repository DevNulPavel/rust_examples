#! /usr/bin/env bash

export PATH=/opt/homebrew/Cellar/mysql/8.0.32/bin:$PATH

mysql --user=root -e "CREATE DATABASE test_db"; 
