.PHONY:
# .SILENT:

ENCRYPT_TEST_ENV:
	gpg -a -r 0x0BD10E4E6E578FB6 -o env/test.env.asc -e env/test.env

DECRYPT_TEST_ENV:
	rm -rf env/test.env
	gpg -a -r 0x0BD10E4E6E578FB6 -o env/test.env -d env/test.env.asc
