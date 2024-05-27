.PHONY:
# .SILENT:

ENCRYPT_TEST_ENV:
	gpg -a -r 0x0BD10E4E6E578FB6 -o env/test_google_service_account.json.asc -e env/test_google_service_account.json
	gpg -a -r 0x0BD10E4E6E578FB6 -o env/test.env.asc -e env/test.env

DECRYPT_TEST_ENV:
	rm -rf env/test.env
	rm -rf env/test_google_service_account.json
	gpg -a -r 0x0BD10E4E6E578FB6 -o env/test_google_service_account.json -d env/test_google_service_account.json.asc

TEST_APP:
	cargo clippy && \
	cargo run