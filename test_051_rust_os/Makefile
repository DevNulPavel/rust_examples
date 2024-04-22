# Используются фичи из nightly билда, надо переключиться туда
# https://os.phil-opp.com/minimal-rust-kernel/
ENABLE_NIGHTLY_TOOLCHAIN:
	rustup component add rust-src
	rustup toolchain add nightly
	rustup default nightly
	rustup component add llvm-tools-preview
	cargo install bootimage

BUILD_KERNEL:
	#cargo build
	cargo bootimage

RUN_KERNEL: BUILD_KERNEL
	# Альтернативный способ запуска:qemu-system-x86_64 -drive format=raw,file=target/x86_64-build_config/debug/bootimage-rust_os.bin
	# Запускает "bootimage runner" - читать .cargo/
	cargo run

UNIT_TEST:
	cargo test

_BUILD_ARM:
	# rustup target add thumbv7em-none-eabihf
	cargo build --target thumbv7em-none-eabihf

_BUILD_OSX:
	cargo rustc -- -C link-args="-e __start -static -nostartfiles"

_BUILD_WINDOWS:
	cargo rustc -- -C link-args="/ENTRY:_start /SUBSYSTEM:console"

_BUILD_LINUX:
	cargo rustc -- -C link-arg=-nostartfiles