
# После -- идут параметры для самой программы
WATCH:
	systemfd --no-pid -s http::8080 -- cargo watch -x run

RUN:
	RUST_BACKTRACE=1 \
	cargo run