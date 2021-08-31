.PHONY:
.SILENT:

# https://aws.amazon.com/ru/blogs/opensource/rust-runtime-for-aws-lambda/
# https://github.com/awslabs/aws-lambda-rust-runtime

PREPARE_ENVIRONMENT_V1:
	rustup target add x86_64-unknown-linux-musl && \
	brew install filosottile/musl-cross/musl-cross && \
	ln -s /usr/local/bin/x86_64-linux-musl-gcc /usr/local/bin/musl-gcc && \
	mkdir .cargo && \
	echo $'[target.x86_64-unknown-linux-musl]\nlinker = "x86_64-linux-musl-gcc"' > .cargo/config

BUILD_APP:
	cargo build --release --target x86_64-unknown-linux-musl && \
	zip -j ./target/bootstrap.zip ./target/x86_64-unknown-linux-musl/release/bootstrap

TEST_APP:
	export RUST_BACKTRACE=full && \
	export RUST_LOG=error && \
	cargo clippy && \
	cargo build && \
	target/debug/test67_nom
