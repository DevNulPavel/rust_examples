RUN_CLI:
	cargo run --bin currencies_request_cli

BUILD_CLI:
	cargo build --bin currencies_request_cli 

BUILD_CLI_RELEASE:
	cargo build --bin currencies_request_cli --release

INSTALL_CLI: BUILD_CLI_RELEASE
	cp -r ./target/release/currencies_request_cli /usr/local/bin/currencies_request

RUN_BOT:
	while true ; do \
	cargo run --bin telegram_bot; \
	sleep 5;\
	done;

RUN_BOT_RELEASE:
	while true ; do \
	cargo run --release --bin telegram_bot; \
	sleep 15;\
	done;

UPDATE_VERSIONS:
	cargo update

BACKUP_KEYS:
	mkdir -p backup/ && \
	rm -f backup/*  && \
	zip -er backup/scripts.zip ./scripts/

# rustup target add arm-unknown-linux-gnueabihf
# rustup default stable-x86_64-apple-darwin
#TARGET_CC=clang cargo build --target arm-unknown-linux-gnueabihf
#TARGET_LD=clang cargo build --target arm-unknown-linux-gnueabihf
#export TARGET_CC=clang 
#export TARGET_LD=clang-ld
BUILD_AND_INSTALL_ON_PI:
	ARM_UNKNOWN_LINUX_GNUEABIHF_OPENSSL_LIB_DIR=/Users/devnul/Projects/Rust_Examples/currencies_request/open_ssl_armv6 \
    ARM_UNKNOWN_LINUX_GNUEABIHF_OPENSSL_INCLUDE_DIR=/Users/devnul/Projects/Rust_Examples/currencies_request/open_ssl_armv6/include \
	cargo build --release --bin telegram_bot --target arm-unknown-linux-gnueabihf;
	scp target/arm-unknown-linux-gnueabihf/release/telegram_bot ~/bin/telegram_bot

BUILD_AND_INSTALL_ON_PI_DOCKER:
	# https://hackernoon.com/seamlessly-cross-compiling-rust-for-raspberry-pis-ede5e2bd3fe2
	# https://hackernoon.com/compiling-rust-for-the-raspberry-pi-49fdcd7df658
	# https://chacin.dev/blog/cross-compiling-rust-for-the-raspberry-pi/
	docker build -t custom-x-compile .
	docker run custom-x-compile > ./rpxc
	chmod +x rpxc
	./rpxc cargo --version
	#./rpxc cargo build --release --bin telegram_bot --target arm-unknown-linux-gnueabihf
	rm rpxc

TREE:
	# https://github.com/sfackler/cargo-tree
	# https://github.com/seanmonstar/reqwest/issues/571
	# cargo install cargo-tree
	cargo tree -i -p native-tls
	# cargo tree -i -p openssl-sys