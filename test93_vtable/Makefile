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
	rm -rf ./output/heap.pb.gz && \
	./target/debug/test92_jemalloc_pprof


# Там есть и flamegraph, и много всего прочего
# Есть еще веб-версии визуализации: https://flamegraph.com/
PPROF_WEB:
	~/go/bin/pprof \
		-http=:8080 \
		-no_browser \
		./target/debug/test92_jemalloc_pprof \
		./output/heap.pb.gz

PPROF_FLAMEGRAPH:
	~/go/bin/pprof \
		-raw \
		./target/debug/test92_jemalloc_pprof \
		./output/heap.pb.gz | \
	stackcollapse-go.pl | \
	flamegraph.pl \
		--hash > ./output/flamegraph.svg
	
PPROF_FLAMECHART:
	~/go/bin/pprof \
		-raw \
		./target/debug/test92_jemalloc_pprof \
		./output/heap.pb.gz | \
	stackcollapse-go.pl | \
	flamegraph.pl \
		--flamechart \
		--hash > ./output/flamechart.svg

# Прочие варианты

PPROF_TRACES:
	~/go/bin/pprof \
		-traces \
		./target/debug/test92_jemalloc_pprof \
		./output/heap.pb.gz

PPROF_TEXT:
	~/go/bin/pprof \
		-text \
		-call_tree \
		./target/debug/test92_jemalloc_pprof \
		./output/heap.pb.gz

PPROF_TOP:
	~/go/bin/pprof \
		-top \
		./target/debug/test92_jemalloc_pprof \
		./output/heap.pb.gz

PPROF_TREE:
	~/go/bin/pprof \
		-tree \
		./target/debug/test92_jemalloc_pprof \
		./output/heap.pb.gz

PPROF_SVG:
	~/go/bin/pprof \
		-svg \
		-output ./output/pprof.svg \
		./target/debug/test92_jemalloc_pprof \
		./output/heap.pb.gz