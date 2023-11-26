# Сборка с поддержкой кросс-компиляции:
# https://rust-lang.github.io/rustup/cross-compilation.html

# Установка зависимостей
INSTALL_TARGET:
	rustup target install x86_64-unknown-linux-gnu && \
	rustup show

# Сборка для целевого таргета
BUILD:
	cargo build \
		--target=x86_64-unknown-linux-gnu