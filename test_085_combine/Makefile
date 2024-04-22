.PHONY:
.SILENT:

# export RUST_BACKTRACE=full

TEST_APP:
	export RUST_BACKTRACE=1 && \
	cargo clippy --all --all-targets && \
	cargo run