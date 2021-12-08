.PHONY:
.SILENT:

RUN_SERVER:
	cd server && \
	cargo clippy && \
	cargo build --release && \
	cd ../ && \
	target/release/server