DAEMON:
	cargo run --bin stats_daemon

CLIENT:
	./scripts/test_upload_script.sh

BACKUP_KEYS:
	mkdir -p backup/ && \
	rm -f backup/*  && \
	zip -er backup/scripts.zip ./scripts/