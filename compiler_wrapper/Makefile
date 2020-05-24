BUILD_FOR_IOS_AND_OSX:
	source ios_osx_bash.env && \
	cargo build --release

TEST:
	cargo run -- \
	/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/clang \
	-lstdc++ \
	-g \
	-O2 \
	test.cpp \
	-o test_result