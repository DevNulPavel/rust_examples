#! /usr/bin/env bash

BUILD_URL=https://asdda.asdad.sda
BUILD_NUMBER=123
GIT_COMMIT=
GIT_BRANCH=

TEST_URL=http://localhost:8888

curl \
    -X POST \
    -H "Content-Type: application/x-www-form-urlencoded" \
    --data-urlencode "build_number=$BUILD_NUMBER" \
    --data-urlencode "git_commit=$GIT_COMMIT" \
    --data-urlencode "git_branch=$GIT_BRANCH" \
    --data-urlencode "build_file_link=asdads" \
    --data-urlencode "build_job_url=$BUILD_URL" \
    "$TEST_URL/jenkins/build_finished"