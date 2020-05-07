RUN_CLI:
	cargo run --bin currencies_request_cli

BUILD_CLI:
	cargo build --bin currencies_request_cli 

BUILD_CLI_RELEASE:
	cargo build --bin currencies_request_cli --release

INSTALL_CLI: BUILD_RELEASE
	cp -r ./target/release/currencies_request_cli /usr/local/bin/currencies_request

RUN_BOT:
	cargo run --bin telegram_bot

UPDATE_VERSIONS:
	cargo update