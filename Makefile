.PHONY: check build run fmt lint test clean

ARGS ?=

check:
	cargo check --all-targets

build:
	cargo build --release

run:
	MALLOC_ARENA_MAX=2 cargo run --release -- $(ARGS)

fmt:
	cargo fmt --all

lint:
	cargo clippy --all-targets -- -D warnings -D clippy::unwrap_used -D clippy::expect_used

test:
	cargo test

clean:
	cargo clean
