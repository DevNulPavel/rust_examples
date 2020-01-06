all: TEST_RELEASE

clean: CLEAN

########################################################################################

# | - значит, что запускать не параллельно, а последовательно

BUILD_SNAPPY_LIB_RELEASE:
	pushd libs/snappy && \
		mkdir -p build_release && \
		pushd build_release && \
			cmake -G "Unix Makefiles" -DCMAKE_BUILD_TYPE="Release" -DSNAPPY_BUILD_TESTS=OFF -DSNAPPY_INSTALL=OFF ../ && \
			make -j5; \
		popd && \
	popd

BUILD_SNAPPY_LIB_DEBUG:
	pushd libs/snappy && \
		mkdir -p build_debug && \
		pushd build_debug && \
			cmake -G "Unix Makefiles" -DCMAKE_BUILD_TYPE="Debug" -DSNAPPY_BUILD_TESTS=OFF -DSNAPPY_INSTALL=OFF ../ && \
			make -j5; \
		popd && \
	popd

BUILD_RUST_APP_RELEASE:
	# export LD_LIBRARY_PATH=$PREFIX/lib:$LD_LIBRARY_PATH
	cargo build --release

BUILD_RUST_APP_DEBUG:
	cargo build

BUILD_RELEASE: | BUILD_SNAPPY_LIB_RELEASE BUILD_RUST_APP_RELEASE

BUILD_DEBUG: | BUILD_SNAPPY_LIB_DEBUG BUILD_RUST_APP_DEBUG

TEST_RELEASE: | BUILD_RELEASE
	./target/release/test20_bindgen

TEST_DEBUG: | BUILD_DEBUG
	./target/debug/test20_bindgen

CLEAN:
	cargo clean;
	pushd libs/snappy && \
	rm -rf build_*; \
	popd;
