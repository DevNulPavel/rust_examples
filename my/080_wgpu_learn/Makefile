.PHONY:
.SILENT:

# export RUST_BACKTRACE=full

TEST_APP:
	export RUST_LOG=test80_wgpu_learn=debug && \
	export RUST_BACKTRACE=1 && \
	cargo clippy --all-targets && \
	cargo run