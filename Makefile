clippy:
	cargo clippy --all-targets --all-features --workspace -- -D warnings

test: 
	cargo test

fmt:
	cargo fmt --all --check

cargo-build:
	cargo build

build: clippy test fmt cargo-build

run_debug:
	POWERTOP_CONFIG=`pwd`/.config POWERTOP_DATA=`pwd`/.data POWERTOP_LOG_LEVEL=debug cargo run

get_todos:
	rustc ./scripts/collect_todos.rs --out-dir ./scripts && ./scripts/collect_todos