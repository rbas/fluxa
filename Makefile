.PHONY: build
build:
	cargo build -release
.PHONY: build-x86-64-linux-gnu
build-x86-64-linux-gnu:
	cargo build --target=x86_64-unknown-linux-gnu --release
