.PHONY:
.SILENT:

####################################################################################

RUN:
	export RUST_BACKTRACE=1 && \
	cargo clippy --all --all-targets && \
	cargo build && \
	./target/debug/test90_lalrpop

REBUILD_LALRPOP:
	cargo check

MANUAL_GEN:
	# cargo install lalrpop
	lalrpop file.lalrpop