.PHONY:
.SILENT:

TEST_APP:
	export RUST_LOG=trace && \
	cargo clippy && \
	cargo build --release && \
	target/release/file_upload_proxy \
		--port 8888 \
		-vvvv