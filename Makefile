.PHONY: all build release debug run test clean

all:
	make build
	make release

build:
	make clean
	cargo bootimage

release:
	make clean
	cargo bootimage --release

debug:
	cargo run

run:
	cargo run --release

test:
	cargo xtest -- --target x86_64-sorrow.json

clean:
	cargo clean


