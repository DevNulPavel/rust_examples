.PHONY:
.SILENT:

TEST_APP:
	cargo clippy && \
	cargo build --release && \
	target/release/file_upload_proxy -vvvv