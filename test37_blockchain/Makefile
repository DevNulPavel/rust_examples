START_SERVER_1:
	RUST_LOG=INFO \
	PORT=8880 \
	cargo run --release

START_SERVER_2:
	RUST_LOG=INFO \
	PORT=8881 \
	cargo run --release

ADD_SERVER_2_NODE:
	# POST
	curl \
		-H "Content-Type: application/json" \
		--data '{ "address": "http://127.0.0.1:8881" }' \
		"http://127.0.0.1:8880/node"

ID:
	# GET
	curl "http://127.0.0.1:8880/id"

CHAIN:
	# GET
	curl "http://127.0.0.1:8880/chain"

TRANSACTION_SERVER_1:
	# POST
	curl \
		-H "Content-Type: application/json" \
		--data '{"from": 1, "to": 2, "amount": 100 }' \
		"http://127.0.0.1:8880/transaction"

TRANSACTION_SERVER_2:
	# POST
	curl \
		-H "Content-Type: application/json" \
		--data '{"from": 1, "to": 2, "amount": 100 }' \
		"http://127.0.0.1:8880/transaction"

MINE:
	# POST
	curl --data "" "http://127.0.0.1:8880/mine";
	echo "";
	curl --data "" "http://127.0.0.1:8881/mine";
		
RECONSILE:
	# POST
	curl --data "" "http://127.0.0.1:8880/reconcile"
	curl --data "" "http://127.0.0.1:8881/reconcile"
