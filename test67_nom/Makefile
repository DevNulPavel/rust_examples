.PHONY:
.SILENT:

TEST_APP:
	export RUST_BACKTRACE=full && \
	export RUST_LOG=error && \
	cargo clippy && \
	cargo build && \
	target/debug/test67_nom

TEST_SCHEME:
	cargo test -- url_parse::scheme::tests

TEST_AUTHORITY:
	cargo test -- url_parse::authority::tests

TEST_HOST:
	cargo test -- url_parse::host::tests

BENCH_ALL:
	cargo bench