BUILD:
	cargo build

BUILD_RELEASE:
	cargo build --release

ENCRYPT_TEST_ENV:
	# -a: ASCII
	# -r: Key fingerpring
	# -e: Encrypt file
	# -o: Output file
	gpg -a -r 0x0BD10E4E6E578FB6 -o test_environment.env.asc -e test_environment.env

DECRYPT_TEST_ENV:
	# -a: ASCII
	# -r: Key fingerpring
	# -d: Encrypt file
	# -o: Output file
	gpg -a -r 0x0BD10E4E6E578FB6 -o test_environment.env -d test_environment.env.asc

BENCH:
	$(shell gpg -a -r 0x0BD10E4E6E578FB6 -d test_environment.env.asc) && \
	cargo run --release --features=flame_it

TEST_ENV_PARAMETERS:
	$(shell gpg -a -r 0x0BD10E4E6E578FB6 -d test_environment.env.asc) && \
	cargo test -- env_parameters::tests

EXAMPLE_APP_PARAMETERS:
	cargo run --bin example_cli_params -- --help

TEST_APP: BUILD
	bash test_uploading.sh
