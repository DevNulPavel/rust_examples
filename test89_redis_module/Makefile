.PHONY: TEST
.SILENT:

# Example: make RUN_REDIS_MODULE MODULE=simple_module
BASIC_RUN_REDIS_MODULE:
	export RUST_BACKTRACE=1 && \
	cargo clippy --all --all-targets && \
	cargo build --release --example ${MODULE} && \
	redis-server \
		--loadmodule ./target/release/examples/lib${MODULE}.dylib

RUN_SIMPLE_MODULE:
	make BASIC_RUN_REDIS_MODULE MODULE=simple_module

TEST_SIMPLE_MODULE:
	redis-cli HELLO.MUL 10 20 30 40