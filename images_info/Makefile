.PHONY:
.SILENT:

TEST_APP:
	# export RUST_BACKTRACE=1
	cargo build --release && \
	time target/release/images_info \
		-v \
		--input-directory "/Users/devnul/projects/island2/build/Emscripten_Standalone/res_to_server" \
		--output-file "/Users/devnul/projects/island2/build/Emscripten_Standalone/res_to_server/imagesInfo.json" \
		--ignore-directories "/Users/devnul/projects/island2/build/Emscripten_Standalone/res_to_server/res/images/buildings/atlases"