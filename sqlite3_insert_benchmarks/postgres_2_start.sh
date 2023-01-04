#! /usr/bin/env bash

export PATH=/opt/homebrew/opt/postgresql@15/bin:$PATH

postgres -D ./postgres_db/ -k ./