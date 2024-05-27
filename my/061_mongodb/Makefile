.SILENT:
.PHONY:

ENCRYPT_TEST_ENV:
	gpg -a -r 0x0BD10E4E6E578FB6 -o env/cloud.env.asc -e env/cloud.env
	gpg -a -r 0x0BD10E4E6E578FB6 -o env/local.env.asc -e env/local.env

DECRYPT_TEST_ENV:
	rm -rf env/cloud.env
	rm -rf env/local.env
	gpg -a -r 0x0BD10E4E6E578FB6 -o env/cloud.env -d env/cloud.env.asc
	gpg -a -r 0x0BD10E4E6E578FB6 -o env/local.env -d env/local.env.asc

RUN_MONGODB_LOCAL_DOCKER:
	# docker-compose stop
	# docker-compose rm -f
	# docker-compose pull
	docker-compose up

# Параметризация происходит с помощью переменных окружения
RUN_APP:
	export RUST_LOG=test61_mongodb=trace,warn && \
	cargo clippy && \
	cargo build --release && \
	source ./env/cloud.env && \
	target/release/test61_mongodb

RUN_TOKIO_CONSOLE:
	# cargo install tokio-console
	tokio-console