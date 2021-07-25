.PHONY:
.SILENT:

TEST_APP:
	export RUST_BACKTRACE=full && \
	export RUST_LOG=error && \
	cargo clippy && \
	cargo build && \
	target/debug/test67_nom
