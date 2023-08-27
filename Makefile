CLIPPY_OPTIONS="-D warnings"

check:
	cargo fmt --all -- --check
	cargo check --workspace --lib
	cargo check --workspace --tests
	cargo clippy --workspace -- "${CLIPPY_OPTIONS}"
	cargo test --workspace

fmt:
	cargo fmt --all

doc:
	cargo doc --document-private-items --no-deps --open
