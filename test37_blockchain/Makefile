ID:
	# GET
	curl "http://127.0.0.1:8888/id"

CHAIN:
	# GET
	curl "http://127.0.0.1:8888/chain"

TRANSACTION:
	# POST
	curl \
		-H "Content-Type: application/json" \
		--data '{ "from": 1, "to": 2, "amount": 100 }' \
		"http://127.0.0.1:8888/transaction"

MINE:
	# POST
	curl --data "" "http://127.0.0.1:8888/mine"

NEW_NODE:
	# POST
	curl \
		-H "Content-Type: application/json" \
		--data '{ "address": "http://127.0.0.1:8881" }' \
		"http://127.0.0.1:8888/transaction"
		
RECONSILE:
	# POST
	curl --data "" "http://127.0.0.1:8888/reconcile"
