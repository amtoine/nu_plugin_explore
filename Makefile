CLIPPY_OPTIONS="-D warnings"

all: check test install

dev-deps:
	./scripts/setup-dev-deps

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

install:
	cargo install --path .
	nu --commands "register ${CARGO_HOME}/bin/nu_plugin_explore"

clean:
	cargo remove nu-plugin
	cargo remove nu-protocol

purge: clean
	cargo clean
