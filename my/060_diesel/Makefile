.SILENT:

DATABASE_INSTALL_DEPENDENCIES:
	brew install postgresql libpq
	cargo install diesel_cli --no-default-features --features postgres

DATABASE_START_SERVER:
	docker-compose up -d

DATABASE_CREATE:
	diesel setup

DATABASE_SETUP:
	diesel migration generate create_posts

DATABASE_MIGRATE:
	diesel migration run

RUN_SERVER:
	cargo run

# Выводить все логи при тесте
TEST:
	cargo test -- --nocapture