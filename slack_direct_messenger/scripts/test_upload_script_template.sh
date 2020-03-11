#! /usr/bin/env bash

export SLACK_API_TOKEN=""

# Slack message
#./target/debug/slack_direct_messenger \
cargo run --bin slack_direct_messenger -- \
    --slack_user_email "" \
    --slack_user "" \
    --slack_user_text "" \
    --slack_user_qr_commentary "" \
    --slack_user_qr_text ""