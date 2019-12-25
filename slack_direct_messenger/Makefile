BUILD:
	cargo build

BUILD_RELEASE:
	cargo build --release

TEST: BUILD
	./scripts/test_upload_script.sh

BACKUP_KEYS:
	mkdir -p backup/ && \
	rm -f backup/*  && \
	zip -er backup/scripts.zip ./scripts/