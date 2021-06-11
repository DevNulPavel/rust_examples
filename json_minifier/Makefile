.PHONY:
.SILENT:

TEST_APP:
	export RUST_BACKTRACE=0 && \
	export RUST_LOG=error && \
	cargo build --release && \
	time target/release/json_minifier \
		--input-folder "/Users/devnul/projects/island2/project/res" \
		-vvvv

# Могут быть проблемы со сборкой для Linux
BUILD_UNIVERSAL_APP_NATIVE:
	rustup target add \
		aarch64-apple-darwin \
		x86_64-apple-darwin \
		x86_64-unknown-linux-gnu && \
	rm -rf target/json_minifier_osx && \
	rm -rf target/json_minifier_linux && \
	cargo build --release --target aarch64-apple-darwin && \
	cargo build --release --target x86_64-apple-darwin && \
	lipo \
		-create \
		target/aarch64-apple-darwin/release/json_minifier \
		target/x86_64-apple-darwin/release/json_minifier \
		-output \
		target/json_minifier_osx
		
	# cargo build --release --target x86_64-unknown-linux-gnu && \
	# cp target/x86_64-unknown-linux-gnu/release/json_minifier target/json_minifier_linux

# Нужен docker для запуска
BUILD_UNIVERSAL_APP_CROSS:
	cargo install cross && \
	rm -rf target/json_minifier_osx && \
	rm -rf target/json_minifier_linux && \
	cross build --release --target aarch64-apple-darwin && \
	cross build --release --target x86_64-apple-darwin && \
	lipo \
		-create \
		target/aarch64-apple-darwin/release/json_minifier \
		target/x86_64-apple-darwin/release/json_minifier \
		-output \
		target/json_minifier_osx

	#cross build --release --target x86_64-unknown-linux-gnu && \
	#cp target/x86_64-unknown-linux-gnu/release/json_minifier target/json_minifier_linux