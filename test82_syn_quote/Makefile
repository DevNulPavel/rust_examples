.PHONY:
.SILENT:

TEST_APP:
	export RUST_LOG=test81_syn_quote=debug && \
	export RUST_BACKTRACE=full && \
	cargo clippy && \
	cargo run --bin example