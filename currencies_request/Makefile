BUILD_COMMAND=cargo
# BUILD_COMMAND=cargo remote -c --


RUN_CLI: BUILD_CLI
	./target/debug/currencies_request_cli

BUILD_CLI:
	# export SCCACHE_MEMCACHED="tcp://192.168.1.2:11211";
	# export SCCACHE_REDIS=""; 
	export RUSTC_WRAPPER=/usr/local/bin/sccache; \
	$(BUILD_COMMAND) build --bin currencies_request_cli 

BUILD_CLI_RELEASE:
	$(BUILD_COMMAND) build --bin currencies_request_cli --release

INSTALL_CLI: BUILD_CLI_RELEASE
	cp -r ./target/release/currencies_request_cli /usr/local/bin/currencies_request

RUN_BOT:
	# RUST_LOG=error
	# RUST_LOG=info
	# RUST_LOG=debug
	# RUST_LOG=trace
	# https://docs.rs/env_logger/0.7.1/env_logger/
	# export SCCACHE_MEMCACHED="tcp://192.168.1.2:11211";
	export RUST_LOG=telegram_bot=trace; \
	export RUSTC_WRAPPER=/usr/local/bin/sccache; \
	while true ; do \
	$(BUILD_COMMAND) build --bin telegram_bot && \
	./target/debug/telegram_bot; \
	sleep 5;\
	done;

RUN_BOT_RELEASE:
	while true ; do \
	$(BUILD_COMMAND) build --bin telegram_bot && \
	./target/release/telegram_bot; \
	sleep 15;\
	done;

UPDATE_VERSIONS:
	$(BUILD_COMMAND) update

BACKUP_KEYS:
	mkdir -p backup/ && \
	rm -f backup/*  && \
	zip -er backup/scripts.zip ./scripts/

BUILD_FOR_PI:
	export RUSTC_WRAPPER="";
	# https://github.com/rust-embedded/cross
	# https://docs.rs/crate/openssl-sys/0.9.19
	# Need to create armv6 open ssl docker build

	# MUSL build !!!
	export OPENSSL_DIR=/armv6_openssl/result && \
	cross build --release --bin telegram_bot --target arm-unknown-linux-musleabi;

	# GNU LibC build !!
	# export OPENSSL_DIR=/armv6_openssl/result && \
	# cross build --release --bin telegram_bot --target arm-unknown-linux-gnueabi;

	# Docker required
	# $(BUILD_COMMAND) install cross
	# OPENSSL_DIR=/Users/devnul/Projects/Rust_Examples/currencies_request/open_ssl_armv6; 
	# export OPENSSL_LIB_DIR=/Users/devnul/Projects/Rust_Examples/currencies_request/open_ssl_armv6/lib; \
    # export OPENSSL_INCLUDE_DIR=/Users/devnul/Projects/Rust_Examples/currencies_request/open_ssl_armv6/include;
	# export OPENSSL_DIR=/Users/devnul/Projects/Rust_Examples/currencies_request/open_ssl_armv6; \
	# cross build --release --bin telegram_bot --target arm-unknown-linux-gnueabi;

	# export ARM_UNKNOWN_LINUX_GNUEABI_OPENSSL_LIB_DIR=/Users/devnul/Projects/Rust_Examples/currencies_request/open_ssl_armv6/lib; \
	# export ARM_UNKNOWN_LINUX_GNUEABI_OPENSSL_INCLUDE_DIR=/Users/devnul/Projects/Rust_Examples/currencies_request/open_ssl_armv6/include; \

	# export OPENSSL_DIR=/Users/devnul/Projects/Rust_Examples/currencies_request/open_ssl_armv6; \
	export OPENSSL_NO_VENDOR=1; \

	# export MACHINE=armv7; \
	# export ARCH=arm; \
	# export CC=arm-linux-gnueabi-gcc; \
	# export TARGET=arm-unknown-linux-gnueabi; \

	# export OPENSSL_DIR=/Users/devnul/Projects/Rust_Examples/currencies_request/open_ssl_armv6; \
	# $(BUILD_COMMAND) build --release --bin telegram_bot --target arm-unknown-linux-gnueabi

	#OPENSSL_DIR=/Users/devnul/Projects/Rust_Examples/currencies_request/open_ssl_armv6
	# MACHINE=armv7 \
	# ARCH=arm \
	# CC=gcc \
	# ARM_UNKNOWN_LINUX_GNUEABI_OPENSSL_LIB_DIR=/Users/devnul/Projects/Rust_Examples/currencies_request/open_ssl_armv6/lib \
	# ARM_UNKNOWN_LINUX_GNUEABI_OPENSSL_INCLUDE_DIR=/Users/devnul/Projects/Rust_Examples/currencies_request/open_ssl_armv6/include \
	# $(BUILD_COMMAND) build --release --bin telegram_bot --target arm-unknown-linux-gnueabi

RUN_ON_PI_DOCKER: 
	# https://github.com/alexey-goloburdin/telegram-finance-bot
	#docker run -d --name telegram_bot -v /home/pi/telegram_bot_db:/bot devnul/telegram_bot
	docker pull devnul/telegram_bot:latest
	docker run -d --env-file /home/pi/telegram_bot.env --name telegram_bot --rm -v /home/pi/telegram_bot_db:/bot/database devnul/telegram_bot:latest
	docker run -it --env-file /home/pi/telegram_bot.env --name telegram_bot --rm -v /home/pi/telegram_bot_db:/bot/database devnul/telegram_bot:latest sh
	# docker exec -ti telegram_bot bash

BUILD_ON_PI_DOCKER: BUILD_FOR_PI
	docker build --no-cache -t devnul/telegram_bot .
	docker push devnul/telegram_bot

BUILD_AND_INSTALL_ON_PI: BUILD_FOR_PI
	# MUSL excutable
	# https://www.ibm.com/developerworks/ru/library/l-lpic1-v3-102-3/index.html
	rsync -av target/arm-unknown-linux-musleabi/release/telegram_bot pi@192.168.1.2:/home/pi/bin/
	rsync -av sql/create_database.sql pi@192.168.1.2:/home/pi/bin/sql/
	rsync -av support/bot_start.sh pi@192.168.1.2:/home/pi/bin/
	# Запуск musl бинарника: 
	# 	export LD_LIBRARY_PATH=/openssl_musl_library_path
	#	/lib/ld-musl-armhf.so.1 telegram_bot

	# GNU executable
	#rsync -av target/arm-unknown-linux-gnueabi/release/telegram_bot pi@192.168.1.2:/home/pi/bin/
	#rsync -av sql/create_database.sql pi@192.168.1.2:/home/pi/bin/sql/
	#rsync -av bot_start.sh pi@192.168.1.2:/home/pi/bin/

	# export OPENSSL_LIB_DIR=/Users/devnul/Projects/Rust_Examples/currencies_request/open_ssl_armv6; \
    # export OPENSSL_INCLUDE_DIR=/Users/devnul/Projects/Rust_Examples/currencies_request/open_ssl_armv6/include; \

	# rustup target add arm-unknown-linux-gnueabi
	# ARM_UNKNOWN_LINUX_GNUEABI_OPENSSL_LIB_DIR=/Users/devnul/Projects/Rust_Examples/currencies_request/open_ssl_armv6 \
    # ARM_UNKNOWN_LINUX_GNUEABI_OPENSSL_INCLUDE_DIR=/Users/devnul/Projects/Rust_Examples/currencies_request/open_ssl_armv6/include \
	# $(BUILD_COMMAND) build --release --bin telegram_bot --target arm-unknown-linux-gnueabi;
	# rsync -av target/arm-unknown-linux-gnueabi/release/telegram_bot pi@192.168.1.2:/home/pi/bin/

	# rustup target add arm-unknown-linux-gnueabi
	# rustup default stable-x86_64-apple-darwin
	#TARGET_CC=clang $(BUILD_COMMAND) build --target arm-unknown-linux-gnueabi
	#TARGET_LD=clang $(BUILD_COMMAND) build --target arm-unknown-linux-gnueabi
	#export TARGET_CC=clang 
	#export TARGET_LD=clang-ld

	# $ brew install arm-linux-gnueabi-binutils
	# $ brew tap osx-cross/arm
	# $ brew install arm-gcc-bin
	# $ brew install llvm
	# ./.$(BUILD_COMMAND)/config
	# [build]
	# target = "armv7-unknown-linux-musleabihf"
	# [target.armv7-unknown-linux-c]
	# linker = "arm-linux-gnueabihf-ld"
	# ARM_UNKNOWN_LINUX_GNUEABIHF_OPENSSL_LIB_DIR=/Users/devnul/Projects/Rust_Examples/currencies_request/open_ssl_armv6 \
    # ARM_UNKNOWN_LINUX_GNUEABIHF_OPENSSL_INCLUDE_DIR=/Users/devnul/Projects/Rust_Examples/currencies_request/open_ssl_armv6/include \
	# $(BUILD_COMMAND) build --release --bin telegram_bot --target arm-unknown-linux-gnueabi; \
	# scp target/arm-unknown-linux-gnueabi/release/telegram_bot ~/bin/telegram_bot

	# cd /tmp
	# wget https://www.openssl.org/source/openssl-1.1.1g.tar.gz
	# tar xzf openssl-1.1.1g.tar.gz
	# export MACHINE=armv6l
	# export ARCH=arm
	# export CC=arm-linux-gnueabi-gcc
	# cd openssl-1.1.1g && ./config shared && make -j8 && cd -
	# export OPENSSL_LIB_DIR=/tmp/openssl-1.1.1g/
	# export OPENSSL_INCLUDE_DIR=/tmp/openssl-1.1.1g/include
	# $(BUILD_COMMAND) build --release --bin telegram_bot --target arm-unknown-linux-gnueabi;
	# scp target/arm-unknown-linux-gnueabi/release/telegram_bot ~/bin/telegram_bot

# BUILD_AND_INSTALL_ON_PI_DOCKER:
	# https://hackernoon.com/seamlessly-cross-compiling-rust-for-raspberry-pis-ede5e2bd3fe2
	# https://hackernoon.com/compiling-rust-for-the-raspberry-pi-49fdcd7df658
	# https://chacin.dev/blog/cross-compiling-rust-for-the-raspberry-pi/
	# docker build -t custom-x-compile .
	# docker run custom-x-compile > ./rpxc
	# chmod +x rpxc
	# ./rpxc $(BUILD_COMMAND) --version
	#./rpxc $(BUILD_COMMAND) build --release --bin telegram_bot --target arm-unknown-linux-gnueabihf
	# rm rpxc

TREE:
	# https://github.com/sfackler/cargo-tree
	# https://github.com/seanmonstar/reqwest/issues/571
	# cargo install cargo-tree
	cargo tree -i -p native-tls
	# cargo tree -i -p openssl-sys

UNUSED_DEPS:
	# cargo install cargo-udeps --locked
	cargo udeps

MIRI_UB_TEST:
	# https://github.com/rust-lang/miri
	# https://rust-lang.github.io/rustup-components-history/x86_64-apple-darwin.html
	# cargo update
	# cargo default nightly
	# rustup +nightly component add miri
	cargo miri run