#! /usr/bin/env bash

# Import test environment
$(gpg -a -r 0x0BD10E4E6E578FB6 -d test_environment.env.asc)

# Print environment
#env

# Rust environment setup
export RUST_BACKTRACE=1
export RUST_LOG=uploader_app=trace,app_center_client=trace,reqwest=trace

# Build app
cargo build --release

# App center
target/release/uploader_app \
    --app_center_input_file "/Users/devnul/Downloads/Island2-Android-qc-1130--2020.12.28_18.23-tf_12.10.0_giads_kinesis-400cd90.apk" \
    --app_center_build_description "Test description" \
    --app_center_distribution_groups "Paradise Island 2 Team","Collaborators"