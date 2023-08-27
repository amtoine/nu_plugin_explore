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

build:
	cargo build --release

register:
	nu --commands "register target/release/nu_plugin_explore"

install: build register
