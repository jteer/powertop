clippy:
	cargo clippy --all-targets --all-features --workspace -- -D warnings

test: 
	cargo test

fmt:
	cargo fmt --all --check

build: clippy test fmt

run_debug:
	POWERTOP_CONFIG=`pwd`/.config POWERTOP_DATA=`pwd`/.data POWERTOP_LOG_LEVEL=debug cargo run