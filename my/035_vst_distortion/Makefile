# https://github.com/RustAudio/vst-rs
BUILD_RELEASE:
	cargo build --release

MAKE_OSX_VST: BUILD_RELEASE
	scripts/osx_vst_bundler.sh DistPlugin target/release/libtest35_vst_distortion.dylib