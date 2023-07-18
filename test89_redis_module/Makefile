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
	redis-cli HSET hash_key_1 key_1 val_1 key_2 val_2 && \
	redis-cli SET set_key_2 val_3 && \
	redis-cli scan_keys

####################################################################################

RUN_RESPONSE:
	make BASIC_RUN_REDIS_MODULE MODULE=response

TEST_RESPONSE:
	redis-cli HSET hash_key_1 key_1 val_1 key_2 val_2 && \
	redis-cli HSET hash_key_2 key_3 val_3 key_4 val_4 key_5 val_3 && \
	redis-cli MAP.MGET hash_key_1 key_1 && \
	redis-cli MAP.UNIQUE hash_key_2 key_3 key_4 key_5

####################################################################################

RUN_KEYS_POS:
	make BASIC_RUN_REDIS_MODULE MODULE=keys_pos

TEST_KEYS_POS:
	redis-cli KEYS_POS key_1 1 key_2 2 key_3 3

####################################################################################

RUN_BLOCK:
	make BASIC_RUN_REDIS_MODULE MODULE=block

TEST_BLOCK:
	redis-cli block

####################################################################################

RUN_DATA_TYPE:
	make BASIC_RUN_REDIS_MODULE MODULE=data_type

TEST_DATA_TYPE:
	redis-cli alloc.set test_key_1 5 && \
	redis-cli alloc.get test_key_1

####################################################################################

RUN_EVENTS:
	make BASIC_RUN_REDIS_MODULE MODULE=events

TEST_EVENTS:
	redis-cli events.send && \
	redis-cli GET aaa && \
	redis-cli events.num_key_miss