INSTALL_RUST:
	curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

BUILD_RELEASE:
	cargo build --release

RUN:
	cargo run -- --http_port=8888

TEST_JENKINS:
	cargo test -- jenkins::tests

TEST_SLACK:
	# cargo test --package slack_command_handler --bin slack_command_handler -- slack::tests::test_slack_client --exact --nocapture
	cargo test -- slack::tests::test_image_upload

TEST_BUILD_EVENTS:
	cargo test -- handlers::events

BUILD_DOCKER_IMAGE:
	docker build -t devnul/slack_command_handler .

RUN_DOCKER_IMAGE_INTERACTIVE:
	docker run -it --rm --env-file ~/slack_bot_test_docker.env --network host --name slack_command_handler devnul/slack_command_handler

RUN_DOCKER_IMAGE_DAEMON:
	docker run -d --rm --env-file ~/slack_bot_test_docker.env --network host --name slack_command_handler devnul/slack_command_handler

PUSH_DOCKER_IMAGE: BUILD_DOCKER_IMAGE
	docker push devnul/slack_command_handler

PULL_DOCKER_IMAGE: 
	docker pull devnul/slack_command_handler
