.SILENT:
.PHONY:

RUN_APP:
	export RUST_LOG=test61_mongodb=trace,warn && \
	cargo clippy && \
	cargo build --release && \
	target/release/test61_mongodb

RUN_TOKIO_CONSOLE:
	# cargo install tokio-console
	tokio-console