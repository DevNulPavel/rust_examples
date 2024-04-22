.SILENT:

DATABASE_MIGRATION_CREATE:
	sqlx migrate add initialization

DATABASE_DOCKER_START:
	docker-compose up -d

DATABASE_RUN_MIGRATION:
	sqlx migrate run

DATABASE_OFFLINE_MODE_UPDATE:
	cargo sqlx prepare

RUN_SERVER:
	export RUST_LOG=trace && \
	source .env && \
	cargo run

# Выводить все логи при тесте
TEST:
	docker-compose -f docker-compose-test.yml up -d && \
	cargo test -- --nocapture;
	docker-compose -f docker-compose-test.yml stop && \
	docker-compose -f docker-compose-test.yml rm -f