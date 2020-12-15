BUILD:
	cargo build

BUILD_RELEASE:
	cargo build --release

BENCH:
	cargo run --release --features=flame_it

INSTALL: BUILD_RELEASE
	cp -r ./target/release/habr_headers /usr/local/bin/habr_headers

TEST_WORKERS_POOL:
	cargo test -- workers_pool::tests

TEST_CLIENT:
	cargo test -- client::tests