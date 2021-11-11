.PHONY:
.SILENT:

TEST_APP:
	export RUST_BACKTRACE=0 && \
	cargo clippy && \
	cargo build --release && \
	time target/release/dir_split \
		--resuld-dirs-count 4 \
		--result-dirs-path "./test_results" \
		--compression-type "brotli" \
		--compression-level 11 \
		--compression-cache-path "test_cache" \
		--source-dirs-root "/Users/devnul/projects/island2/build/Emscripten_Standalone" \
		--source-dirs \
			"res/data/localConfigs,res/data/symbols,res/dragonbones,res/fonts,res/images,res/match3,res/models,res/shaders,res/sounds,res/spine,res/ui,res/lang" \
		-vvvv

INSTALL_PROFILE_DEPS:
	cargo install flamegraph

PROFILE_APP:
	sudo cargo flamegraph -- \
		--resuld-dirs-count 4 \
		--result-dirs-path "./test_results" \
		--compression-type "brotli" \
		--compression-level 11 \
		--compression-cache-path "test_cache" \
		--source-dirs-root "/Users/devnul/projects/island2/build/Emscripten_Standalone" \
		--source-dirs \
			"res/data/localConfigs,res/data/symbols,res/dragonbones,res/fonts,res/images,res/match3,res/models,res/shaders,res/sounds,res/spine,res/ui,res/lang" \
		-v

# Могут быть проблемы со сборкой для Linux
BUILD_UNIVERSAL_APP_NATIVE:
	rustup target add \
		aarch64-apple-darwin \
		x86_64-apple-darwin \
		x86_64-unknown-linux-gnu && \
	rm -rf target/dir_split_osx && \
	rm -rf target/dir_split_linux && \
	cargo build --release --target aarch64-apple-darwin && \
	cargo build --release --target x86_64-apple-darwin && \
	lipo \
		-create \
			target/aarch64-apple-darwin/release/dir_split \
			target/x86_64-apple-darwin/release/dir_split \
		-output \
			target/dir_split_osx
		
	# cargo build --release --target x86_64-unknown-linux-gnu && \
	# cp target/x86_64-unknown-linux-gnu/release/dir_split target/dir_split_linux

# Нужен docker для запуска
BUILD_UNIVERSAL_APP_CROSS:
	cargo install cross && \
	rm -rf target/dir_split_osx && \
	rm -rf target/dir_split_linux && \
	cross build --release --target aarch64-apple-darwin && \
	cross build --release --target x86_64-apple-darwin && \
	lipo \
		-create \
			target/aarch64-apple-darwin/release/dir_split \
			target/x86_64-apple-darwin/release/dir_split \
		-output \
			target/dir_split_osx

	#cross build --release --target x86_64-unknown-linux-gnu && \
	#cp target/x86_64-unknown-linux-gnu/release/dir_split target/dir_split_linux