ENCRYPT_TEST_ENV:
	gpg -a -r 0x0BD10E4E6E578FB6 -o test_environment.env.asc -e test_environment.env

DECRYPT_TEST_ENV:
	gpg -a -r 0x0BD10E4E6E578FB6 -o test_environment.env -d test_environment.env.asc

RUN_APP:
	$(shell gpg -a -r 0x0BD10E4E6E578FB6 -d test_environment.env.asc) && \
	cargo run --release