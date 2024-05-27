.PHONY: TEST
.SILENT:

TEST:
	export RUST_BACKTRACE=1 && \
	cargo clippy --all --all-targets && \
	cargo run --release