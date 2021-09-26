.PHONY:
.SILENT:

TEST_APP:
	export RUST_BACKTRACE=0 && \
	export RUST_LOG=error && \
	cargo build --release && \
	time target/release/lang_filter \
		--lang-files-folder "/Users/devnul/projects/island2/project/res/lang" \
		--filter-config-path "/Users/devnul/projects/rust_examples/lang_filter/test_config.json" \
		-vvvv

# Могут быть проблемы со сборкой для Linux
BUILD_UNIVERSAL_APP_NATIVE:
	rustup target add \
		aarch64-apple-darwin \
		x86_64-apple-darwin \
		x86_64-unknown-linux-gnu && \
	rm -rf target/lang_filter_osx && \
	rm -rf target/lang_filter_linux && \
	cargo build --release --target aarch64-apple-darwin && \
	cargo build --release --target x86_64-apple-darwin && \
	lipo \
		-create \
			target/aarch64-apple-darwin/release/lang_filter \
			target/x86_64-apple-darwin/release/lang_filter \
		-output \
			target/lang_filter_osx
		
	# cargo build --release --target x86_64-unknown-linux-gnu && \
	# cp target/x86_64-unknown-linux-gnu/release/lang_filter target/lang_filter_linux

# Нужен docker для запуска
BUILD_UNIVERSAL_APP_CROSS:
	cargo install cross && \
	rm -rf target/lang_filter_osx && \
	rm -rf target/lang_filter_linux && \
	cross build --release --target aarch64-apple-darwin && \
	cross build --release --target x86_64-apple-darwin && \
	lipo \
		-create \
			target/aarch64-apple-darwin/release/lang_filter \
			target/x86_64-apple-darwin/release/lang_filter \
		-output \
			target/lang_filter_osx

	#cross build --release --target x86_64-unknown-linux-gnu && \
	#cp target/x86_64-unknown-linux-gnu/release/lang_filter target/lang_filter_linux