# Описание командной утилиты по работе с базой данных:
# https://github.com/launchbadge/sqlx/tree/master/sqlx-cli

INITIALIZE_DATABASE:
	export DATABASE_URL=sqlite://oauth_db.sqlite && \
	sqlx database create

CREATE_DATABASE_MIGRATION:
	export DATABASE_URL=sqlite://oauth_db.sqlite && \
	sqlx migrate add initialize

PERFORM_DATABASE_MIGRATION:
	export DATABASE_URL=sqlite://oauth_db.sqlite && \
	sqlx migrate run

ENCRYPT_TEST_ENV:
	gpg -a -r 0x0BD10E4E6E578FB6 -o test_environment.env.asc -e test_environment.env

DECRYPT_TEST_ENV:
	rm -rf test_environment.env
	gpg -a -r 0x0BD10E4E6E578FB6 -o test_environment.env -d test_environment.env.asc

ENCRYPT_GOOGLE_CREDENTIALS:
	gpg -a -r 0x0BD10E4E6E578FB6 -o test_google_auth_credentials.json.asc -e test_google_auth_credentials.json

DECRYPT_GOOGLE_CREDENTIALS:
	rm -rf test_google_auth_credentials.json
	gpg -a -r 0x0BD10E4E6E578FB6 -o test_google_auth_credentials.json -d test_google_auth_credentials.json.asc

RUN_SERVER: DECRYPT_GOOGLE_CREDENTIALS
	$(shell gpg -a -r 0x0BD10E4E6E578FB6 -d test_environment.env.asc) && \
	source .env && \
	export RUST_LOG=actix_web=debug,test58_oauth=trace && \
	cargo run