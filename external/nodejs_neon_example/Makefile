INSTALL:
	npm install -g neon-cli

BUILD_DEBUG:
	neon build

BUILD_RELEASE:
	neon build --release

CLEAN:
	neon clean

RUN_DEBUG: BUILD_DEBUG
	node lib/index.js

RUN_RELEASE: BUILD_RELEASE
	node lib/index.js