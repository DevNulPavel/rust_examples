.PHONY:
.SILENT:

TEST_APP:
	# --other-source-directories "./target"
	export RUST_BACKTRACE=0 && \
	export RUST_LOG=error && \
	cargo build --release && \
	time target/release/prepare_base_packs \
		--config-json "test_config.json" \
		--packs-directory "/Users/devnul/projects/island2/project/download/pack_sources" \
		--packs-directory-prefixes "pack_base_" "pack_common_all_" \
		-vvv

# Могут быть проблемы со сборкой для Linux
BUILD_UNIVERSAL_APP_NATIVE:
	rustup target add \
		aarch64-apple-darwin \
		x86_64-apple-darwin \
		x86_64-unknown-linux-gnu && \
	rm -rf target/prepare_base_packs_osx && \
	rm -rf target/prepare_base_packs_linux && \
	cargo build --release --target aarch64-apple-darwin && \
	cargo build --release --target x86_64-apple-darwin && \
	lipo \
		-create \
		target/aarch64-apple-darwin/release/prepare_base_packs \
		target/x86_64-apple-darwin/release/prepare_base_packs \
		-output \
		target/prepare_base_packs_osx
		
	# cargo build --release --target x86_64-unknown-linux-gnu && \
	# cp target/x86_64-unknown-linux-gnu/release/prepare_base_packs target/prepare_base_packs_linux

# Нужен docker для запуска
BUILD_UNIVERSAL_APP_CROSS:
	cargo install cross && \
	rm -rf target/prepare_base_packs_osx && \
	rm -rf target/prepare_base_packs_linux && \
	cross build --release --target aarch64-apple-darwin && \
	cross build --release --target x86_64-apple-darwin && \
	lipo \
		-create \
		target/aarch64-apple-darwin/release/prepare_base_packs \
		target/x86_64-apple-darwin/release/prepare_base_packs \
		-output \
		target/prepare_base_packs_osx && \
	cross build --release --target x86_64-unknown-linux-gnu && \
	cp target/x86_64-unknown-linux-gnu/release/prepare_base_packs target/prepare_base_packs_linux