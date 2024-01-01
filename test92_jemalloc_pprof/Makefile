.PHONY:
.SILENT:

####################################################################################

RUN:
	export RUST_BACKTRACE=1 && \
	cargo clippy \
		--all \
		--all-targets && \
	cargo build && \
	./target/debug/test92_jemalloc_pprof

PPROF_WEB: