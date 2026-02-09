.PHONY: build install clean release test

BINARY_NAME=whoopterm
BUILD_DIR=target/release

build:
	cargo build --release

install:
	cargo install --path .

test:
	cargo test

clean:
	cargo clean

# Build for multiple platforms
release: release-macos release-linux

release-macos:
	cargo build --release --target x86_64-apple-darwin
	cargo build --release --target aarch64-apple-darwin

release-linux:
	cargo build --release --target x86_64-unknown-linux-gnu
	cargo build --release --target aarch64-unknown-linux-gnu

release-windows:
	cargo build --release --target x86_64-pc-windows-msvc

dev:
	cargo run
