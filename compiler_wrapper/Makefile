# source ios_osx_bash.env &&

BUILD_FOR_IOS_AND_OSX:
	cargo build --release

TEST: MAKE_CONFIG
	cargo run -- \
	-lstdc++ \
	-g \
	-O2 \
	test.cpp \
	-o test_result

MAKE_CONFIG:
	rm -rf target/release/compiler_path.cfg;
	rm -rf target/debug/compiler_path.cfg;
	echo /Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/clang > target/debug/compiler_path.cfg;
	echo /Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/clang > target/release/compiler_path.cfg;