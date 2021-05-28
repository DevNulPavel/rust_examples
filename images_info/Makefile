.PHONY:
.SILENT:

TEST_APP:
	#export RUST_BACKTRACE=1
	cargo build --release && \
	time target/release/images_info \
		-v \
		--input-directory "/Users/devnul/projects/island2/build/Emscripten_Standalone/res_to_server" \
		--output-file "/Users/devnul/projects/island2/build/Emscripten_Standalone/res_to_server/imagesInfo.json" \
		--ignore-directories "/Users/devnul/projects/island2/build/Emscripten_Standalone/res_to_server/res/images/buildings/atlases"

# Могут быть проблемы со сборкой для Linux
BUILD_UNIVERSAL_APP_NATIVE:
	rustup target add \
		aarch64-apple-darwin \
		x86_64-apple-darwin \
		x86_64-unknown-linux-gnu && \
	cargo build --release --target aarch64-apple-darwin && \
	cargo build --release --target x86_64-apple-darwin && \
	lipo \
		-create \
		target/aarch64-apple-darwin/release/images_info \
		target/x86_64-apple-darwin/release/images_info \
		-output \
		target/images_info_osx && \
	cargo build --release --target x86_64-unknown-linux-gnu

# Нужен docker для запуска
BUILD_UNIVERSAL_APP_CROSS:
	cargo install cross && \
	cross build --release --target aarch64-apple-darwin && \
	cross build --release --target x86_64-apple-darwin && \
	lipo \
		-create \
		target/aarch64-apple-darwin/release/images_info \
		target/x86_64-apple-darwin/release/images_info \
		-output \
		target/images_info_osx && \
	cross build --release --target x86_64-unknown-linux-gnu