RUN:
	cargo run

BUILD:
	cargo build

BUILD_RELEASE:
	cargo build --release

INSTALL: BUILD_RELEASE
	cp -r ./target/release/currencies_request_cli /usr/local/bin/currencies_request

UPDATE_VERSIONS:
	cargo update