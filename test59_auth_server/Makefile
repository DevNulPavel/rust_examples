.SILENT:

DATABASE_MIGRATION_CREATE:
	sqlx migrate add initialization

DATABASE_DOCKER_START:
	docker-compose up -d

DATABASE_RUN_MIGRATION:
	sqlx migrate run

RUN_SERVER:
	export RUST_LOG=trace && \
	cargo run