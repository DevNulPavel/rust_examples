.PHONY:

BUILD_CODE:
	wasm-pack build --no-typescript --target bundler

TEST_CODE:
	wasm-pack test --headless --safari

CREATE_WEB_PAGE:
	npm init wasm-app www

INSTALL_WWW_DEPENCIES:
	cd www && \
	npm install

RUN_SERVER:
	cd www && \
	npm run start