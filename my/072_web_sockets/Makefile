.PHONY:
.SILENT:

RUN_SERVER:
	export RUST_SPANTRACE=1 \
	export RUST_BACKTRACE=0 \
	export RUST_LOG=server=trace,info && \
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

TEST_SERVER_5:
	websocat \
		-t \
		-E \
		--ping-interval 5 \
		--base64 \
		ws://127.0.0.1:8080/web_socket