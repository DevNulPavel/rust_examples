.PHONY:
.SILENT:

ENCRYPT_TEST_ENV:
	gpg -a -r 0x0BD10E4E6E578FB6 -o env/test_google_service_account.json.asc -e env/test_google_service_account.json
	gpg -a -r 0x0BD10E4E6E578FB6 -o env/test.env.asc -e env/test.env

DECRYPT_TEST_ENV:
	rm -rf env/test.env
	rm -rf env/test_google_service_account.json
	gpg -a -r 0x0BD10E4E6E578FB6 -o env/test_google_service_account.json -d env/test_google_service_account.json.asc

TEST_APP:
	export RUST_LOG=trace && \
	cargo clippy && \
	cargo build --release && \
	target/release/file_upload_proxy \
		--google-credentials-file "env/test_google_service_account.json" \
		--port 8888 \
		-vvvv