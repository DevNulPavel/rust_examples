# https://github.com/RustAudio/vst-rs
BUILD_RELEASE:
	cargo build --release

MAKE_OSX_VST: BUILD_RELEASE
	scripts/osx_vst_bundler.sh PolyFilterPlugin target/release/libtest33_vst_plugin_filter.dylib