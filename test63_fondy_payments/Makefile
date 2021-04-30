.SILENT:
.PHONY:

DATABASE_CREATE_EMPTY:
	sqlx database create

DATABASE_INITIALIZE_MIGRATIONS:
	sqlx migrate add init

DATABASE_MIGRATIONS_ADD:
	sqlx migrate add <NAME>