.PHONY:
.SILENT:

CLEAR_TEST_DATA:
	rm -rf test_*_res

TEST_APP: CLEAR_TEST_DATA
	export RUST_BACKTRACE=0 && \
	export RUST_LOG=error && \
	cargo build --release && \
	time target/release/prepare_base_packs \
		--config-json "test_config.json" \
		--source-directories "/Users/devnul/projects/island2/project" \
							 "/Users/devnul/projects/island2/project/patched_res/base" \
							 "/Users/devnul/projects/island2/project/targets/Emscripten" \
		--packs-directory "/Users/devnul/projects/island2/project/download/pack_sources" \
		--packs-directory-prefixes "pack_base_" "pack_common_all_" \
		--target-client-res-directory "./test_client_res" \
		--target-server-res-directory "./test_server_res"
		# -vvv

COMPARE_RESULTS_EXAMPLE:
	diff  -x '.*' -r -q folder2 folder1 

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
	# cargo install cross
	# cargo install --git https://github.com/rust-embedded/cross.git
	
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