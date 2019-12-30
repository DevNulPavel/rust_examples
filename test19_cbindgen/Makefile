all: TEST_RELEASE

clean: CLEAN

########################################################################################

# | - значит, что запускать не параллельно, а последовательно

BUILD_RUST_LIB_RELEASE:
	cargo build --release --lib

BUILD_RUST_LIB_DEBUG:
	cargo build --lib

MAKE_RUST_INCLUDE_HEADER:
	cbindgen --config "CBindgen.toml" --crate "test19_cbindgen" --output "shared_include/library.h"

BUILD_CPP_APP_RELEASE:
	pushd cpp_app && \
		mkdir -p build_release && \
		pushd build_release && \
			cmake -G "Unix Makefiles" -DCMAKE_BUILD_TYPE="Release" ../ && \
			make -j5; \
		popd && \
	popd

BUILD_CPP_APP_DEBUG:
	pushd cpp_app && \
		mkdir -p build_debug && \
		pushd build_debug && \
			cmake -G "Unix Makefiles" -DCMAKE_BUILD_TYPE="Debug" ../ && \
			make -j5; \
		popd && \
	popd

BUILD_RELEASE: | BUILD_RUST_LIB_RELEASE MAKE_RUST_INCLUDE_HEADER BUILD_CPP_APP_RELEASE

BUILD_DEBUG: | BUILD_RUST_LIB_DEBUG MAKE_RUST_INCLUDE_HEADER BUILD_CPP_APP_DEBUG

TEST_RELEASE: | BUILD_RELEASE
	./cpp_app/build_release/cpp_rust_app

TEST_DEBUG: | BUILD_DEBUG
	./cpp_app/build_debug/cpp_rust_app

CLEAN:
	rm -rf shared_include;
	cargo clean;
	pushd cpp_app && \
	rm -rf build_*; \
	popd;
