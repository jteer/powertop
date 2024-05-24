clippy:
	cargo clippy --all-targets --all-features --workspace -- -D warnings

test: 
	cargo test

fmt:
	cargo fmt --all --check

build: clippy test fmt