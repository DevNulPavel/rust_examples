.PHONY:
.SILENT:

TEST_ORIGINAL_SCRIPT:
	python2 \
		/Users/devnul/projects/island2/project/targets/Emscripten/prepare_res/prepare_dynamic_packs.py \
		"./test_config/dynamic_packs.json" \
		"/Users/devnul/projects/island2/project" \
		"./test_output_script/packResources.json" \
		"./test_output_script/"

TEST_APP:
	# --resources-directory "/Users/devnul/projects/island2/build/Emscripten_Standalone/res_to_server
	export RUST_BACKTRACE=1 && \
	export RUST_LOG=error && \
	cargo build --release && \
	time target/release/prepare_dynamic_packs \
		--dynamic-packs-config-path "./test_config/dynamic_packs.json" \
		--resources-directory "/Users/devnul/projects/island2/project" \
		--output-resources-config-path "./test_output_app/packResources.json" \
		--output-dynamic-packs-dir "./test_output_app/" \
		--config-pack-file-dir "res/dynamicPacks/" \
		-v

# Могут быть проблемы со сборкой для Linux
BUILD_UNIVERSAL_APP_NATIVE:
	rustup target add \
		aarch64-apple-darwin \
		x86_64-apple-darwin \
		x86_64-unknown-linux-gnu && \
	rm -rf target/prepare_dynamic_packs_osx && \
	rm -rf target/prepare_dynamic_packs_linux && \
	cargo build --release --target aarch64-apple-darwin && \
	cargo build --release --target x86_64-apple-darwin && \
	lipo \
		-create \
		target/aarch64-apple-darwin/release/prepare_dynamic_packs \
		target/x86_64-apple-darwin/release/prepare_dynamic_packs \
		-output \
		target/prepare_dynamic_packs_osx

	#cargo build --release --target x86_64-unknown-linux-gnu && \
	#cp target/x86_64-unknown-linux-gnu/release/prepare_dynamic_packs target/prepare_dynamic_packs_linux

# Нужен docker для запуска
BUILD_UNIVERSAL_APP_CROSS:
	cargo install cross && \
	rm -rf target/prepare_dynamic_packs_osx && \
	rm -rf target/prepare_dynamic_packs_linux && \
	cross build --release --target aarch64-apple-darwin && \
	cross build --release --target x86_64-apple-darwin && \
	lipo \
		-create \
		target/aarch64-apple-darwin/release/prepare_dynamic_packs \
		target/x86_64-apple-darwin/release/prepare_dynamic_packs \
		-output \
		target/prepare_dynamic_packs_osx

	#cross build --release --target x86_64-unknown-linux-gnu && \
	#cp target/x86_64-unknown-linux-gnu/release/prepare_dynamic_packs target/prepare_dynamic_packs_linux