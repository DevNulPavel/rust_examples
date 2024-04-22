# https://github.com/RustAudio/vst-rs
BUILD_RELEASE:
	cargo build --release

MAKE_OSX_VST: BUILD_RELEASE
	scripts/osx_vst_bundler.sh FFTPlugin target/release/libtest32_vst_plugin.dylib

TEST_BIN:
	export RUST_BACKTRACE=1; cargo run