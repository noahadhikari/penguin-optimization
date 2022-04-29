prog :=pengwin

.PHONY: all format build install debub debug-install

all: format build

format:
	cargo +nightly fmt

build:
	cargo build --release

install:
	install target/release/$(prog) /usr/local/bin/$(prog)

debug: format
	cargo build

debug-install:
	install target/debug/$(prog) /usr/local/bin/$(prog)-debug