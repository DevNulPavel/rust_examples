# source ios_osx_bash.env &&

BUILD_FOR_IOS_AND_OSX:
	cargo build --release

# Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/clang
TEST_CONFIG_FILE: MAKE_CONFIG
	cargo run -- \
	-lstdc++ \
	-g \
	-O2 \
	test.cpp \
	-o test_result

TEST_PARAM: REMOVE_CONFIG
	cargo run -- \
	/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/clang \
	-lstdc++ \
	-g \
	-O2 \
	test.cpp \
	-o test_result

REMOVE_CONFIG:
	rm -rf target/release/compiler_wrapper_path.cfg;
	rm -rf target/debug/compiler_wrapper_path.cfg;

MAKE_CONFIG: REMOVE_CONFIG
	mkdir -p target/debug/;
	mkdir -p target/release/;
	rm -rf target/release/compiler_wrapper_path.cfg;
	rm -rf target/debug/compiler_wrapper_path.cfg;
	echo /Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/clang > target/debug/compiler_wrapper_path.cfg;
	echo /Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/clang > target/release/compiler_wrapper_path.cfg;