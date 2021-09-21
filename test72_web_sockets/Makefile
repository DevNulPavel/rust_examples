.PHONY:
.SILENT:

RUN_SERVER:
	export RUST_LOG=server=debug,info && \
	cargo clippy --bin server && \
	cargo run --bin server

TEST_SERVER_1:
	curl \
		-v -L \
		127.0.0.1:8080

TEST_SERVER_2:
	curl \
		-v -L \
		-d "test_post_data_for_send" \
		127.0.0.1:8080/echo

TEST_SERVER_3:
	curl \
		-v -L \
		-d "test_post_data_for_send" \
		127.0.0.1:8080/echo/uppercase

TEST_SERVER_4:
	curl \
		-v -L \
		-d "test_post_data_for_send" \
		127.0.0.1:8080/echo/reverse