all: TEST_APP_RELEASE

clean: CLEAN

########################################################################################
# Установка зависимостей

INSTALL_PYTHON_DEPS:
	pip3 install cffi

# https://mozilla.github.io/firefox-browser-architecture/experiments/2017-09-06-rust-on-ios.html
INSTALL_IOS_DEPS:
	rustup target add aarch64-apple-ios armv7-apple-ios armv7s-apple-ios x86_64-apple-ios i386-apple-ios
	cargo install cargo-lipo

INSTALL_ANDROID_DEPS:
	rustup target add armv7-linux-androideabi aarch64-linux-android

INSTALL_WASM_DEPS:
	rustup target add wasm32-unknown-emscripten wasm32-unknown-unknown wasm32-wasi
	pushd /Users/devnul/Projects/emsdk/
	./emsdk install latest
	./emsdk activate latest
	source ./emsdk_env.fish
	popd

########################################################################################
# | - значит, что запускать не параллельно, а последовательно

_BUILD_RUST_LIB_RELEASE:
	cargo build --release

_BUILD_RUST_LIB_DEBUG:
	cargo build

_BUILD_RUST_IOS_LIB_RELEASE:
	cargo lipo --targets aarch64-apple-ios,armv7-apple-ios,armv7s-apple-ios,x86_64-apple-ios --release

_BUILD_RUST_IOS_LIB_DEBUG:
	cargo lipo --targets aarch64-apple-ios,armv7-apple-ios,armv7s-apple-ios,x86_64-apple-ios

_BUILD_RUST_WASM_LIB_RELEASE:
	cargo build --target wasm32-unknown-emscripten --release

_BUILD_RUST_WASM_LIB_DEBUG:
	cargo build --target wasm32-unknown-emscripten

_BUILD_RUST_ANDROID_LIB_RELEASE:
	cargo build --target armv7-linux-androideabi --release
	cargo build --target aarch64-linux-android --release

_BUILD_RUST_ANDROID_LIB_DEBUG:
	cargo build --target armv7-linux-androideabi
	cargo build --target aarch64-linux-android

# Создает, но не очень подходящий хедер
# _MAKE_RUST_INCLUDE_HEADER:
# 	cbindgen --config "CBindgen.toml" --crate "test19_cbindgen" --output "shared_include/library.h"

_MAKE_RUST_INCLUDE_HEADER:

_BUILD_CPP_APP_RELEASE:
	cd cpp_app && \
		mkdir -p build_release && \
		cd build_release && \
			cmake -G "Unix Makefiles" -DCMAKE_BUILD_TYPE="Release" ../ && \
			make -j5; \
		cd ../ && \
	cd ../

_BUILD_CPP_APP_DEBUG:
	cd cpp_app && \
		mkdir -p build_debug && \
		cd build_debug && \
			cmake -G "Unix Makefiles" -DCMAKE_BUILD_TYPE="Debug" ../ && \
			make -j5; \
		cd ../ && \
	cd ../

########################################################################################

BUILD_RELEASE_APP: | _BUILD_RUST_LIB_RELEASE _MAKE_RUST_INCLUDE_HEADER _BUILD_CPP_APP_RELEASE

BUILD_DEBUG_APP: | _BUILD_RUST_LIB_DEBUG _MAKE_RUST_INCLUDE_HEADER _BUILD_CPP_APP_DEBUG

BUILD_RELEASE_IOS_LIB: | _BUILD_RUST_IOS_LIB_RELEASE _MAKE_RUST_INCLUDE_HEADER

BUILD_DEBUG_IOS_LIB: | _BUILD_RUST_IOS_LIB_DEBUG _MAKE_RUST_INCLUDE_HEADER

# TODO: Other libs

TEST_APP_RELEASE: | BUILD_RELEASE_APP
	./cpp_app/build_release/cpp_rust_app
	#./python_app/main.py -release

TEST_APP_DEBUG: | BUILD_DEBUG_APP
	./cpp_app/build_debug/cpp_rust_app
	#./python_app/main.py -debug

CLEAN:
	rm -rf shared_include;
	cargo clean;
	pushd cpp_app && \
	rm -rf build_*; \
	popd;
