CLIPPY_OPTIONS="-D warnings"

all: check test install

dev-deps:
	cargo add nu-plugin --path ${NUSHELL_PATH}/crates/nu-plugin/
	cargo add nu-protocol --path ${NUSHELL_PATH}/crates/nu-protocol/ --features plugin

check:
	cargo fmt --all -- --check
	cargo check --workspace --lib
	cargo check --workspace --tests
	cargo clippy --workspace -- "${CLIPPY_OPTIONS}"

test: check
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
