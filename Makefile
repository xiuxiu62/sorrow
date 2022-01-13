.PHONY: build run image test check clean

build:
	cargo kbuild

build-release:
	cargo kbuild-release

image:
	cargo kimage

run:
	cargo krun

run-release:
	cargo krun-release

test:
	cargo ktest

check:
	cargo kcheck

clean:
	cargo clean
