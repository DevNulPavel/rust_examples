export NDK_HOME=/Users/devnul/AndroidNDK_19

BUILD:
	cargo build

BUILD_RELEASE:
	cargo build --release

# https://mozilla.github.io/firefox-browser-architecture/experiments/2017-09-21-rust-on-android.html
BUILD_ARM:
	mkdir -p NDK
	${NDK_HOME}/build/tools/make_standalone_toolchain.py --api 26 --arch arm64 --install-dir NDK/arm64
	rustup target add aarch64-linux-android
	cargo build --target aarch64-linux-android

TEST: BUILD
	./target/debug/vulkan_shader_converter --input "./testShaders" --output "./resultShaders"

TEST_TIME: BUILD_RELEASE
	time ./target/release/vulkan_shader_converter --input "./testShaders" --output "./resultShaders"
