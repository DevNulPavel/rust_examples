BUILD_RUST_LIB:
	cargo build --release && \
	rm -rf $(shell pwd)/python_script/rust_python_lib.so && \
	ln -s $(shell pwd)/target/release/librust_python_lib.dylib $(shell pwd)/python_script/rust_python_lib.so

TEST: BUILD_RUST_LIB
	cd python_script && \
	python3 main.py; \
	cd ../

INSTALL_MATURIN:
	pip3 install 
	
CREATE_VIRTUAL_ENV:
	python3 -m venv ./python_env

ACTIVATE_VIRTUAL_ENV:
	# Нужно вызвать руками
	source ./python_env/bin/activate

CLEAN:
	cargo clean
	rm -rf python_env