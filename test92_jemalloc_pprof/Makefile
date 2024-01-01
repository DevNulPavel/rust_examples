.PHONY:
.SILENT:

####################################################################################

PPROF:
	

RUN_TEST:
	export RUST_BACKTRACE=1 && \
	cargo clippy \
		--release \
		--all \
		--all-targets && \
	cargo build \
		--release && \
	./target/release/test92_jemalloc_pprof
