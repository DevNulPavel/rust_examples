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
	cargo build --release --lib

_BUILD_RUST_LIB_DEBUG:
	cargo build --lib

_BUILD_RUST_IOS_LIB_RELEASE:
	cargo lipo --targets aarch64-apple-ios,armv7-apple-ios,armv7s-apple-ios,x86_64-apple-ios --release

_BUILD_RUST_IOS_LIB_DEBUG:
	cargo lipo --targets aarch64-apple-ios,armv7-apple-ios,armv7s-apple-ios,x86_64-apple-ios

_BUILD_RUST_WASM_LIB_RELEASE:
	cargo build --target wasm32-unknown-emscripten --release

_BUILD_RUST_WASM_LIB_DEBUG:
	cargo build --target wasm32-unknown-emscripten

# Создает, но не очень подходящий хедер
# _MAKE_RUST_INCLUDE_HEADER:
# 	cbindgen --config "CBindgen.toml" --crate "test19_cbindgen" --output "shared_include/library.h"

_MAKE_RUST_INCLUDE_HEADER:

_BUILD_CPP_APP_RELEASE:
	pushd cpp_app && \
		mkdir -p build_release && \
		pushd build_release && \
			cmake -G "Unix Makefiles" -DCMAKE_BUILD_TYPE="Release" ../ && \
			make -j5; \
		popd && \
	popd

_BUILD_CPP_APP_DEBUG:
	pushd cpp_app && \
		mkdir -p build_debug && \
		pushd build_debug && \
			cmake -G "Unix Makefiles" -DCMAKE_BUILD_TYPE="Debug" ../ && \
			make -j5; \
		popd && \
	popd

########################################################################################

BUILD_RELEASE_APP: | _BUILD_RUST_LIB_RELEASE _MAKE_RUST_INCLUDE_HEADER _BUILD_CPP_APP_RELEASE

BUILD_DEBUG_APP: | _BUILD_RUST_LIB_DEBUG _MAKE_RUST_INCLUDE_HEADER _BUILD_CPP_APP_DEBUG

BUILD_RELEASE_IOS_LIB: | _BUILD_RUST_IOS_LIB_RELEASE _MAKE_RUST_INCLUDE_HEADER

BUILD_DEBUG_IOS_LIB: | _BUILD_RUST_IOS_LIB_DEBUG _MAKE_RUST_INCLUDE_HEADER

TEST_APP_RELEASE: | BUILD_RELEASE_APP
	./cpp_app/build_release/cpp_rust_app
	./python_app/main.py -release

TEST_APP_DEBUG: | BUILD_DEBUG_APP
	./cpp_app/build_debug/cpp_rust_app
	./python_app/main.py -debug

CLEAN:
	rm -rf shared_include;
	cargo clean;
	pushd cpp_app && \
	rm -rf build_*; \
	popd;
