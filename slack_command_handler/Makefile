INSTALL_RUST:
	curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

BUILD_RELEASE:
	cargo build --release

ENCRYPT_TEST_ENV:
	# -a: ASCII
	# -r: Key fingerpring
	# -e: Encrypt file
	# -o: Output file
	gpg -a -r 0x0BD10E4E6E578FB6 -o test_environment.env.asc -e test_environment.env

DECRYPT_TEST_ENV:
	# -a: ASCII
	# -r: Key fingerpring
	# -d: Encrypt file
	# -o: Output file
	gpg -a -r 0x0BD10E4E6E578FB6 -o test_environment.env -d test_environment.env.asc

RUN:
	$(shell gpg -a -r 0x0BD10E4E6E578FB6 -d test_environment.env.asc) && \
	cargo run -- --http_port=8888

TEST_JENKINS:
	$(shell gpg -a -r 0x0BD10E4E6E578FB6 -d test_environment.env.asc) && \
	cargo test -- jenkins::tests

TEST_SLACK:
	# cargo test --package slack_command_handler --bin slack_command_handler -- slack::tests::test_slack_client --exact --nocapture
	$(shell gpg -a -r 0x0BD10E4E6E578FB6 -d test_environment.env.asc) && \	
	cargo test -- slack::tests::test_find_user

TEST_BUILD_EVENTS:
	$(shell gpg -a -r 0x0BD10E4E6E578FB6 -d test_environment.env.asc) && \
	cargo test -- handlers::events

BUILD_DOCKER_IMAGE:
	docker build -t devnul/slack_command_handler .

RUN_DOCKER_IMAGE_INTERACTIVE:
	docker run -it --rm --env-file ~/slack_bot_test_docker.env --network host --name slack_command_handler devnul/slack_command_handler

RUN_DOCKER_IMAGE_DAEMON:
	docker run -d --env-file ~/slack_bot_test_docker.env --network host --restart unless-stopped --name slack_command_handler devnul/slack_command_handler

PUSH_DOCKER_IMAGE: BUILD_DOCKER_IMAGE
	docker push devnul/slack_command_handler

PULL_DOCKER_IMAGE: 
	docker pull devnul/slack_command_handler
