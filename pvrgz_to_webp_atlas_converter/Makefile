.PHONY:
.SILENT:

TEST_APP:
	export RUST_BACKTRACE=0 && \
	export RUST_LOG=error && \
	cargo build --release && \
	time target/release/pvrgz_to_webp_atlas_converter \
		--minimum-pvrgz-size 2048 \
		--target-webp-quality 75 \
		--atlases-images-directory "/Users/devnul/projects/island2/project/res/images/buildings/atlases" \
		--cache-path "./test_cache"
		# --alternative-atlases-json-directory ""
		# -vvv

# Могут быть проблемы со сборкой для Linux
BUILD_UNIVERSAL_APP_NATIVE:
	rustup target add \
		aarch64-apple-darwin \
		x86_64-apple-darwin \
		x86_64-unknown-linux-gnu && \
	rm -rf target/pvrgz_to_webp_atlas_converter_osx && \
	rm -rf target/pvrgz_to_webp_atlas_converter_linux && \
	cargo build --release --target aarch64-apple-darwin && \
	cargo build --release --target x86_64-apple-darwin && \
	lipo \
		-create \
		target/aarch64-apple-darwin/release/pvrgz_to_webp_atlas_converter \
		target/x86_64-apple-darwin/release/pvrgz_to_webp_atlas_converter \
		-output \
		target/pvrgz_to_webp_atlas_converter_osx
		
	# cargo build --release --target x86_64-unknown-linux-gnu && \
	# cp target/x86_64-unknown-linux-gnu/release/pvrgz_to_webp_atlas_converter target/pvrgz_to_webp_atlas_converter_linux

# Нужен docker для запуска
BUILD_UNIVERSAL_APP_CROSS:
	cargo install cross && \
	rm -rf target/pvrgz_to_webp_atlas_converter_osx && \
	rm -rf target/pvrgz_to_webp_atlas_converter_linux && \
	cross build --release --target aarch64-apple-darwin && \
	cross build --release --target x86_64-apple-darwin && \
	lipo \
		-create \
		target/aarch64-apple-darwin/release/pvrgz_to_webp_atlas_converter \
		target/x86_64-apple-darwin/release/pvrgz_to_webp_atlas_converter \
		-output \
		target/pvrgz_to_webp_atlas_converter_osx

	#cross build --release --target x86_64-unknown-linux-gnu && \
	#cp target/x86_64-unknown-linux-gnu/release/pvrgz_to_webp_atlas_converter target/pvrgz_to_webp_atlas_converter_linux