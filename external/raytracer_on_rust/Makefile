TEST:
	RUST_BACKTRACE=1 \
	cargo build && \
	./target/debug/raytracer_on_rust

RUN:
	RUST_BACKTRACE=1 \
	cargo build --release && \
	./target/release/raytracer_on_rust

BENCH_SINGLE_THREAD:
	cargo build --release --features "" && \
	time ./target/release/raytracer_on_rust

BENCH_MULTI_THREAD:
	cargo build --release --features "multi_threaded" && \
	time ./target/release/raytracer_on_rust

BENCH_MULTI_THREAD_UNSAFE:
	cargo build --release --features "multi_threaded allow_unsafe" && \
	time ./target/release/raytracer_on_rust

PROFILE_CPU:
	# --flamechart
	cargo flamegraph \
		--root \
		--features "" \
		--bin raytracer_on_rust

UNIT_TEST:
	RUST_BACKTRACE=1 \
	cargo test

CLEAN:
	cargo clean