BUILD:
	cargo build

BUILD_RELEASE:
	cargo build --release

BENCH:
	cargo run --release --features=flame_it

INSTALL: BUILD_RELEASE
	cp -r ./target/release/habr_headers_async /usr/local/bin/habr_headers_async

TEST_ENV_PARAMETERS:
	cargo test -- env_parameters::tests

EXAMPLE_APP_PARAMETERS:
	cargo run --bin example_cli_params -- --help