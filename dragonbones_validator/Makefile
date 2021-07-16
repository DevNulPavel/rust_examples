.PHONY:
.SILENT:

TEST_APP:
	export RUST_BACKTRACE=0 && \
	export RUST_LOG=error && \
	cargo clippy && \
	cargo build --release && \
	time target/release/dragonbones_validator \
		--json-files-directory "/Users/devnul/projects/island2/build/Emscripten_Standalone/res/dragonbones" \
		--alternative-texture-files-directory "/Users/devnul/projects/island2/build/Emscripten_Standalone/res_to_server/res/dragonbones" \
		--x2-texture-size \
		-vvv

# Могут быть проблемы со сборкой для Linux
BUILD_UNIVERSAL_APP_NATIVE:
	rustup target add \
		aarch64-apple-darwin \
		x86_64-apple-darwin \
		x86_64-unknown-linux-gnu && \
	rm -rf target/dragonbones_validator_osx && \
	rm -rf target/dragonbones_validator_linux && \
	cargo build --release --target aarch64-apple-darwin && \
	cargo build --release --target x86_64-apple-darwin && \
	lipo \
		-create \
		target/aarch64-apple-darwin/release/dragonbones_validator \
		target/x86_64-apple-darwin/release/dragonbones_validator \
		-output \
		target/dragonbones_validator_osx
		
	# cargo build --release --target x86_64-unknown-linux-gnu && \
	# cp target/x86_64-unknown-linux-gnu/release/dragonbones_validator target/dragonbones_validator_linux

# Нужен docker для запуска
BUILD_UNIVERSAL_APP_CROSS:
	# cargo install cross
	# cargo install --git https://github.com/rust-embedded/cross.git
	
	rm -rf target/dragonbones_validator_osx && \
	rm -rf target/dragonbones_validator_linux && \
	cross build --release --target aarch64-apple-darwin && \
	cross build --release --target x86_64-apple-darwin && \
	lipo \
		-create \
		target/aarch64-apple-darwin/release/dragonbones_validator \
		target/x86_64-apple-darwin/release/dragonbones_validator \
		-output \
		target/dragonbones_validator_osx && \
	cross build --release --target x86_64-unknown-linux-gnu && \
	cp target/x86_64-unknown-linux-gnu/release/dragonbones_validator target/dragonbones_validator_linux