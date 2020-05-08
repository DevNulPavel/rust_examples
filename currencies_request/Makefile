RUN_CLI:
	cargo run --bin currencies_request_cli

BUILD_CLI:
	cargo build --bin currencies_request_cli 

BUILD_CLI_RELEASE:
	cargo build --bin currencies_request_cli --release

INSTALL_CLI: BUILD_CLI_RELEASE
	cp -r ./target/release/currencies_request_cli /usr/local/bin/currencies_request

RUN_BOT:
	cargo run --bin telegram_bot

UPDATE_VERSIONS:
	cargo update

BACKUP_KEYS:
	mkdir -p backup/ && \
	rm -f backup/*  && \
	zip -er backup/scripts.zip ./scripts/

BUILD_AND_INSTALL_ON_PI:
	# rustup target add arm-unknown-linux-gnueabihf
	# rustup default stable-x86_64-apple-darwin
	#TARGET_CC=clang cargo build --target arm-unknown-linux-gnueabihf
	#TARGET_LD=clang cargo build --target arm-unknown-linux-gnueabihf
	export TARGET_CC=clang 
	export TARGET_LD=clang-ld
	cargo build --release --bin telegram_bot --target arm-unknown-linux-gnueabihf
	scp target/arm-unknown-linux-gnueabihf/release/telegram_bot ~/bin/telegram_bot

TREE:
	# https://github.com/sfackler/cargo-tree
	# https://github.com/seanmonstar/reqwest/issues/571
	# cargo install cargo-tree
	cargo tree -i -p native-tls
	# cargo tree -i -p openssl-sys