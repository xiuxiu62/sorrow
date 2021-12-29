.PHONY: all build release debug run test clean

all:
	make build
	make release

build:
	cargo bootimage

release:
	cargo bootimage --release

debug:
	cargo run

run:
	cargo run --release

test:
	cargo xtest -- --target x86_64-vonnegut.json

clean:
	cargo clean


