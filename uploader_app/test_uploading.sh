#! /usr/bin/env bash

# Json auth
rm -rf test_google_drive_auth.json
gpg -a -r 0x0BD10E4E6E578FB6 -o test_google_drive_auth.json -d test_google_drive_auth.json.asc

# Import test environment
eval "$(gpg -a -r 0x0BD10E4E6E578FB6 -d test_environment.env.asc)"
# gpg -a -r 0x0BD10E4E6E578FB6 -d test_environment.env.asc -o test_environment.env
# source test_environment.env

# Print environment
# env | grep -i "GOOGLE_DRIVE"

# Rust environment setup
export RUST_BACKTRACE=1
export RUST_LOG=uploader_app=trace,reqwest=trace

# App center
# target/release/uploader_app \
#     --app_center_input_file "/Users/devnul/Downloads/Island2-Android-qc-1130--2020.12.28_18.23-tf_12.10.0_giads_kinesis-400cd90.apk" \
#     --app_center_build_description "Test description" \
#     --app_center_distribution_groups "Paradise Island 2 Team","Collaborators"

# Google drive
# google_drive_subfolder_name
# --google_drive_target_owner_email "devnulpavel@gmail.com"
# --google_drive_target_domain ""
target/debug/uploader_app \
    --google_drive_files "/Users/devnul/Downloads/Island2-Android-qc-1130--2020.12.28_18.23-tf_12.10.0_giads_kinesis-400cd90.apk" \
    --google_drive_target_drive_id "0AFOEWCRBt5u2Uk9PVA" \
    --google_drive_target_folder_id "1YtSfyiMp-MxF5AVWq_VnJxGtAwiMghBF"
