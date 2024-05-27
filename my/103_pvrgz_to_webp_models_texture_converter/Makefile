.PHONY:
.SILENT:

TEST_APP:
	export RUST_BACKTRACE=0 && \
	export RUST_LOG=error && \
	cargo build --release && \
	time target/release/pvrgz_to_webp_models_texture_converter \
		--minimum-pvrgz-size 128 \
		--target-webp-quality 75 \
		--models-config-json "/Users/devnul/projects/island2/build/Emscripten_Standalone/res/models/models.json" \
		--pvrgz-directories "/Users/devnul/projects/island2/build/Emscripten_Standalone/res/models" "/Users/devnul/projects/island2/build/Emscripten_Standalone/res_to_server/res/models" \
		--cache-path "./test_cache" \
		--ignore-config-path "./test_ignore_images.json"
		# -vvv

# Могут быть проблемы со сборкой для Linux
BUILD_UNIVERSAL_APP_NATIVE:
	rustup target add \
		aarch64-apple-darwin \
		x86_64-apple-darwin \
		x86_64-unknown-linux-gnu && \
	rm -rf target/pvrgz_to_webp_models_texture_converter_osx && \
	rm -rf target/pvrgz_to_webp_models_texture_converter_linux && \
	cargo build --release --target aarch64-apple-darwin && \
	cargo build --release --target x86_64-apple-darwin && \
	lipo \
		-create \
		target/aarch64-apple-darwin/release/pvrgz_to_webp_models_texture_converter \
		target/x86_64-apple-darwin/release/pvrgz_to_webp_models_texture_converter \
		-output \
		target/pvrgz_to_webp_models_texture_converter_osx
		
	# cargo build --release --target x86_64-unknown-linux-gnu && \
	# cp target/x86_64-unknown-linux-gnu/release/pvrgz_to_webp_models_texture_converter target/pvrgz_to_webp_models_texture_converter_linux

# Нужен docker для запуска
BUILD_UNIVERSAL_APP_CROSS:
	cargo install cross && \
	rm -rf target/pvrgz_to_webp_models_texture_converter_osx && \
	rm -rf target/pvrgz_to_webp_models_texture_converter_linux && \
	cross build --release --target aarch64-apple-darwin && \
	cross build --release --target x86_64-apple-darwin && \
	lipo \
		-create \
		target/aarch64-apple-darwin/release/pvrgz_to_webp_models_texture_converter \
		target/x86_64-apple-darwin/release/pvrgz_to_webp_models_texture_converter \
		-output \
		target/pvrgz_to_webp_models_texture_converter_osx && \
	cross build --release --target x86_64-unknown-linux-gnu && \
	cp target/x86_64-unknown-linux-gnu/release/pvrgz_to_webp_models_texture_converter target/pvrgz_to_webp_models_texture_converter_linux