.PHONY:
.SILENT:

# export RUST_BACKTRACE=full

START_CHROME_WEB_DRIVER:
	chromedriver

TEST_APP:
	export RUST_BACKTRACE=1 && \
	cargo clippy --all --all-targets && \
	cargo run