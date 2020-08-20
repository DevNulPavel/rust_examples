
# После -- идут параметры для самой программы
WATCH:
	systemfd --no-pid -s http::8080 -- cargo watch -x run

RUN:
	RUST_BACKTRACE=1 \
	cargo run

DEPLOY:
	cargo build --release && \
	rm -rf deploy_result && \
	mkdir -p deploy_result && \
	cp target/release/odroid_web_server deploy_result/ && \
	cp -R rustls_certificates deploy_result/ && \
	cp -R static deploy_result/ && \
	cp -R html deploy_result/