# source ios_osx_bash.env &&

CLEAN:
	cargo clean
	rm -rf test_result.dSYM
	rm -rf test_result

BUILD_FOR_IOS_AND_OSX:
	cargo build --release

# Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/clang
TEST_CONFIG_FILE: | PREPARE MAKE_COMPILER_FILE
	XGEN_ENABLE_DISTCC=0 \
	XGEN_ENABLE_CCACHE=0 \
	cargo run -- \
	-lstdc++ \
	-g \
	-O2 \
	test.cpp \
	-o test_result

TEST_PARAM: | PREPARE REMOVE_COMPILER_FILE
	cargo run -- \
	/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/clang \
	-lstdc++ \
	-g \
	-O2 \
	test.cpp \
	-o test_result

PREPARE:
	mkdir -p target/debug/ && \
	mkdir -p target/release/ && \
	cp compiler_wrapper_config.json target/debug/ && \
	cp compiler_wrapper_config.json target/release/

REMOVE_COMPILER_FILE:
	rm -rf target/release/compiler_wrapper_path.cfg;
	rm -rf target/debug/compiler_wrapper_path.cfg;

MAKE_COMPILER_FILE: REMOVE_COMPILER_FILE
	mkdir -p target/debug/;
	mkdir -p target/release/;
	rm -rf target/release/compiler_wrapper_path.cfg;
	rm -rf target/debug/compiler_wrapper_path.cfg;
	echo /Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/clang > target/debug/compiler_wrapper_path.cfg;
	echo /Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/clang > target/release/compiler_wrapper_path.cfg;