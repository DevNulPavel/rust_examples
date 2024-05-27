BOOTSTRAP_NODE:
	RUST_LOG=bootstrap_node=info;
	cargo run --example bootstrap_node -- --expected_conns 1 --ip "127.0.0.1"

CLIENT_NODE:
	RUST_LOG=client_node=info;
	cargo run --example client_node -- \
		--hard-coded-contacts '["127.0.0.1:5000"]'
	# '[{"peer_addr": "127.0.0.1:5000", "peer_cert_der":[48,130,..]}]'

CHAT:
	cargo run --example chat