#! /usr/bin/env bash

BUILD_URL=https://asdda.asdad.sda
BUILD_NUMBER=123
GIT_COMMIT=
GIT_BRANCH=
BUILD_USER_ID=pershov
BUILD_USER="Pavel Ershov"
BUILD_USER_EMAIL=pershov@game-insight.com
#DEFAULT_CHANNEL="#mur-test_node_upload"
DEFAULT_CHANNEL=""

TEST_URL=http://localhost:8888

curl \
    -X POST \
    -H "Content-Type: application/x-www-form-urlencoded" \
    --data-urlencode "build_number=$BUILD_NUMBER" \
    --data-urlencode "git_commit=$GIT_COMMIT" \
    --data-urlencode "git_branch=$GIT_BRANCH" \
    --data-urlencode "build_file_link=http://devnulpavel.ddns.net:8888" \
    --data-urlencode "build_file_commentary=http://devnulpavel.ddns.net:8888" \
    --data-urlencode "build_job_url=$BUILD_URL" \
    --data-urlencode "build_user_id=$BUILD_USER_ID" \
    --data-urlencode "build_user_name=$BUILD_USER" \
    --data-urlencode "build_user_email=$BUILD_USER_EMAIL" \
    --data-urlencode "default_channel=$DEFAULT_CHANNEL" \
    "$TEST_URL/jenkins/build_finished"