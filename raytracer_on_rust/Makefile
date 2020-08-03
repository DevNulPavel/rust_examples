TEST:
	RUST_BACKTRACE=1 \
	cargo build && \
	./target/debug/raytracer_on_rust

BENCH_SINGLE_THREAD:
	cargo build --release --features "" && \
	time ./target/release/raytracer_on_rust

BENCH_MULTI_THREAD:
	cargo build --release --features "multi_threaded" && \
	time ./target/release/raytracer_on_rust

BENCH_MULTI_THREAD_UNSAFE:
	cargo build --release --features "multi_threaded allow_unsafe" && \
	time ./target/release/raytracer_on_rust

UNIT_TEST:
	cargo test

CLEAN:
	cargo clean