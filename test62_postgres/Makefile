# Описание командной утилиты по работе с базой данных:
# https://github.com/launchbadge/sqlx/tree/master/sqlx-cli

.SILENT:
.PHONY:

START_POSTGRES:
	docker-compose up -d

########################################################################################

DATABASE_INITIALIZE:
	source .env && \
	sqlx database create

DATABASE_MIGRATION_CREATE:
	source .env && \
	sqlx migrate add initialize

DATABASE_MIGRATION_PERFORM:
	source .env && \
	sqlx migrate run

########################################################################################

SQLX_OFFLINE_MODE:
	# https://www.lpalmieri.com/posts/2020-11-01-zero-to-production-5-how-to-deploy-a-rust-application/
	cargo sqlx prepare
