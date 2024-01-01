.PHONY:
.SILENT:

####################################################################################

# Используется pprof на GO, а не какой-то другой
INSTALL_GO_PPROF:
	go install github.com/google/pprof@latest

RUN:
	export RUST_BACKTRACE=1 && \
	cargo clippy \
		--all \
		--all-targets && \
	cargo build && \
	rm -rf heap.pb.gz && \
	./target/debug/test92_jemalloc_pprof

PPROF_TOP:
	~/go/bin/pprof \
		-top \
		./target/debug/test92_jemalloc_pprof \
		heap.pb.gz

PPROF_WEB:
	~/go/bin/pprof \
		-http=:8080 \
		./target/debug/test92_jemalloc_pprof \
		heap.pb.gz

PPROF_SVG:
	~/go/bin/pprof \
		--svg \
		./target/debug/test92_jemalloc_pprof \
		heap.pb.gz