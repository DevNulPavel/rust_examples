all: TEST_APP_RELEASE

clean: CLEAN

########################################################################################
# Установка зависимостей

# Список возможных таргетов: rustup target list

# https://mozilla.github.io/firefox-browser-architecture/experiments/2017-09-06-rust-on-ios.html
INSTALL_LINUX_DEPS:
	rustup target add x86_64-unknown-linux-gnu 

INSTALL_WINDOWS_DEPS:
	rustup target add aarch64-pc-windows-msvc i686-pc-windows-msvc x86_64-pc-windows-msvc

INSTALL_IOS_DEPS:
	rustup target add aarch64-apple-ios armv7-apple-ios armv7s-apple-ios x86_64-apple-ios i386-apple-ios
	cargo install cargo-lipo

INSTALL_ANDROID_DEPS:
	rustup target add armv7-linux-androideabi aarch64-linux-android x86_64-linux-android i686-linux-android

INSTALL_WASM_DEPS:
	rustup target add wasm32-unknown-emscripten wasm32-unknown-unknown wasm32-wasi
	cd ${EMSDK_PATH}
	./emsdk install latest
	./emsdk activate latest
	source ./emsdk_env.fish; source ./emsdk_env.sh
	cd -

########################################################################################
# | - значит, что запускать не параллельно, а последовательно

# OSX
_BUILD_OSX_RELEASE:
	cargo build --lib --release

_BUILD_OSX_DEBUG:
	cargo build --lib

# Linux
_BUILD_LINUX_RELEASE:
	cargo build --target x86_64-unknown-linux-gnu --lib --release

_BUILD_LINUX_DEBUG:
	cargo build --target x86_64-unknown-linux-gnu --lib

# Windows
_BUILD_WINDOWS_RELEASE:
	cargo build --target aarch64-pc-windows-msvc --lib --release
	cargo build --target i686-pc-windows-msvc --lib --release
	cargo build --target x86_64-pc-windows-msvc --lib --release

_BUILD_WINDOWS_DEBUG:
	cargo build --target aarch64-pc-windows-msvc --lib
	cargo build --target i686-pc-windows-msvc --lib
	cargo build --target x86_64-pc-windows-msvc --lib

# iOS
_BUILD_IOS_RELEASE:
	cargo lipo --targets aarch64-apple-ios,armv7-apple-ios,armv7s-apple-ios,x86_64-apple-ios --release

_BUILD_IOS_DEBUG:
	cargo lipo --targets aarch64-apple-ios,armv7-apple-ios,armv7s-apple-ios,x86_64-apple-ios

# ANDROID
_BUILD_ANDROID_RELEASE:
	cargo build --target armv7-linux-androideabi --lib --release
	cargo build --target aarch64-linux-android --lib --release
	cargo build --target x86_64-linux-android --lib --release
	cargo build --target i686-linux-android --lib --release
	 
_BUILD_ANDROID_DEBUG:
	cargo build --target armv7-linux-androideabi --lib
	cargo build --target aarch64-linux-android --lib
	cargo build --target x86_64-linux-android --lib
	cargo build --target i686-linux-android --lib

# WASM
_BUILD_WASM_RELEASE:
	source ${EMSDK_PATH}/emsdk_env.sh && \
	cargo build --target wasm32-unknown-emscripten --lib --release

_BUILD_WASM_DEBUG:
	source ${EMSDK_PATH}/emsdk_env.sh && \
	cargo build --target wasm32-unknown-emscripten --lib

########################################################################################

_MAKE_FOLDERS:
	mkdir -p "result/include";
	mkdir -p "result/OSX/release" "result/OSX/debug";
	mkdir -p "result/LINUX/release" "result/LINUX/debug";
	mkdir -p "result/WINDOWS/release/arm" "result/WINDOWS/release/win32" "result/WINDOWS/release/x64" \
			 "result/WINDOWS/debug/arm"   "result/WINDOWS/debug/win32"   "result/WINDOWS/debug/x64";
	mkdir -p "result/ANDROID/release/armv7" "result/ANDROID/release/aarch64" "result/ANDROID/release/x32" "result/ANDROID/release/x64" \
			 "result/ANDROID/debug/armv7"   "result/ANDROID/debug/aarch64"   "result/ANDROID/debug/x32"   "result/ANDROID/debug/x64";
	mkdir -p "result/IOS/release" "result/IOS/debug";
	mkdir -p "result/WASM/release" "result/WASM/debug";

_COPY_HEADER:
	cp "./include/rust_extention_lib.h" "result/include/"

_BUILD_RELEASE: _BUILD_OSX_RELEASE _BUILD_LINUX_RELEASE _BUILD_WINDOWS_RELEASE _BUILD_IOS_RELEASE _BUILD_ANDROID_RELEASE _BUILD_WASM_RELEASE _BUILD_ANDROID_RELEASE
	cp "./target/release/librust_extention_lib.a" "result/OSX/release/"
	cp "./target/x86_64-unknown-linux-gnu/release/librust_extention_lib.a" "result/LINUX/release/"
	cp "./target/aarch64-pc-windows-msvc/release/rust_extention_lib.lib" "result/WINDOWS/release/arm"
	cp "./target/i686-pc-windows-msvc/release/rust_extention_lib.lib" "result/WINDOWS/release/win32"
	cp "./target/x86_64-pc-windows-msvc/release/rust_extention_lib.lib" "result/WINDOWS/release/x64"
	cp "./target/universal/release/librust_extention_lib.a" "result/IOS/release/"
	cp "./target/armv7-linux-androideabi/release/librust_extention_lib.a" "result/ANDROID/release/armv7"
	cp "./target/aarch64-linux-android/release/librust_extention_lib.a" "result/ANDROID/release/aarch64"
	cp "./target/i686-linux-android/release/librust_extention_lib.a" "result/ANDROID/release/x32"
	cp "./target/x86_64-linux-android/release/librust_extention_lib.a" "result/ANDROID/release/x64"
	cp "./target/wasm32-unknown-emscripten/release/librust_extention_lib.a" "result/WASM/release"

# Собираем Debug только для OSX
_BUILD_DEBUG: _BUILD_OSX_DEBUG #  _BUILD_LINUX_DEBUG _BUILD_IOS_DEBUG _BUILD_ANDROID_DEBUG _BUILD_WASM_DEBUG _BUILD_ANDROID_DEBUG
	cp "./target/debug/librust_extention_lib.a" "result/OSX/debug/"
	# cp "./target/x86_64-unknown-linux-gnu/debug/librust_extention_lib.a" "result/LINUX/debug/"
	# cp "./target/aarch64-pc-windows-msvc/debug/rust_extention_lib.lib" "result/WINDOWS/debug/arm"
	# cp "./target/i686-pc-windows-msvc/debug/rust_extention_lib.lib" "result/WINDOWS/debug/win32"
	# cp "./target/x86_64-pc-windows-msvc/debug/rust_extention_lib.lib" "result/WINDOWS/debug/x64"	
	# cp "./target/universal/debug/librust_extention_lib.a" "result/IOS/debug/"
	# cp "./target/armv7-linux-androideabi/debug/librust_extention_lib.a" "result/ANDROID/debug/armv7"
	# cp "./target/aarch64-linux-android/debug/librust_extention_lib.a" "result/ANDROID/debug/aarch64"
	# cp "./target/i686-linux-android/debug/librust_extention_lib.a" "result/ANDROID/debug/x32"
	# cp "./target/x86_64-linux-android/debug/librust_extention_lib.a" "result/ANDROID/debug/x64"	
	# cp "./target/wasm32-unknown-emscripten/debug/librust_extention_lib.a" "result/WASM/debug"

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

BUILD: | CLEAN _MAKE_FOLDERS _COPY_HEADER _BUILD_RELEASE _BUILD_DEBUG

CLEAN:
	rm -r -f cpp_app/build_release;
	rm -r -f cpp_app/build_debug;
	rm -r -f result;
	cargo clean;

MAKE_RUST_PROFILE_BUILD:
	cargo build

TEST_RUST:
	cargo test

TEST_CPP_APP_RELEASE: | _BUILD_OSX_RELEASE _BUILD_CPP_APP_RELEASE
	./cpp_app/build_release/cpp_rust_app

TEST_CPP_APP_DEBUG: | _BUILD_OSX_DEBUG _BUILD_CPP_APP_DEBUG
	./cpp_app/build_debug/cpp_rust_app