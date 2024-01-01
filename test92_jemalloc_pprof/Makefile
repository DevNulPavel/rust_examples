.PHONY:
.SILENT:

####################################################################################

# Используется pprof на GO, а не какой-то другой
INSTALL_GO_PPROF:
	go install github.com/google/pprof@latest

INSTALL_FLAMEGRAPH:
	sudo yum install flamegraph flamegraph-stackcollapse flamegraph-stackcollapse-perf -y 

RUN:
	export RUST_BACKTRACE=1 && \
	cargo clippy \
		--all \
		--all-targets && \
	cargo build && \
	rm -rf ./heap.pb.gz && \
	./target/debug/test92_jemalloc_pprof

PPROF_FLAMEGRAPH:
	~/go/bin/pprof \
		-raw \
		./target/debug/test92_jemalloc_pprof \
		./heap.pb.gz | \
	stackcollapse-go.pl | \
	flamegraph.pl > flamegraph.svg
	
# Прочие варианты

PPROF_TRACES:
	~/go/bin/pprof \
		-traces \
		./target/debug/test92_jemalloc_pprof \
		./heap.pb.gz

PPROF_TEXT:
	~/go/bin/pprof \
		-text \
		-call_tree \
		./target/debug/test92_jemalloc_pprof \
		./heap.pb.gz

PPROF_TOP:
	~/go/bin/pprof \
		-top \
		./target/debug/test92_jemalloc_pprof \
		./heap.pb.gz

PPROF_TREE:
	~/go/bin/pprof \
		-tree \
		./target/debug/test92_jemalloc_pprof \
		./heap.pb.gz

PPROF_WEB:
	~/go/bin/pprof \
		-http=:8080 \
		./target/debug/test92_jemalloc_pprof \
		heap.pb.gz

PPROF_SVG:
	~/go/bin/pprof \
		-svg \
		./target/debug/test92_jemalloc_pprof \
		heap.pb.gz