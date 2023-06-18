.PHONY:
.SILENT:

####################################################################################

# Example: make RUN_REDIS_MODULE MODULE=simple_module
BASIC_RUN_REDIS_MODULE:
	export RUST_BACKTRACE=1 && \
	cargo clippy --release --all --all-targets && \
	cargo build --release --example ${MODULE} && \
	redis-server \
		--loadmodule ./target/release/examples/lib${MODULE}.dylib

####################################################################################

RUN_SIMPLE_MODULE:
	make BASIC_RUN_REDIS_MODULE MODULE=simple_module

TEST_SIMPLE_MODULE:
	redis-cli HELLO.MUL 10 20 30 40

####################################################################################

RUN_SCAN_KEYS:
	make BASIC_RUN_REDIS_MODULE MODULE=scan_keys

TEST_SCAN_KEYS:
	redis-cli HSET my_key_1 key_1 val_1 key_2 val_2 && \
	redis-cli SET my_key_2 val_3 && \
	redis-cli scan_keys