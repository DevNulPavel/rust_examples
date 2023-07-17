.PHONY:
.SILENT:

####################################################################################

RUN_REDIS:
	redis-server \
		redis_7.0.conf \
		--loadmodule \
			/Users/devnul/projects/RediSearch/bin/macos-arm64v8-release/search-static/redisearch.so

RUN_TEST:
	export RUST_BACKTRACE=1 && \
	cargo clippy --release --all --all-targets && \
	cargo build --release && \
	./target/release/test91_redis_search
