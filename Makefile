CLIPPY_OPTIONS="-D warnings"

.PHONY: all check test fmt doc build register install clean
DEFAULT: check test

check:
	cargo fmt --all --verbose -- --check --verbose
	cargo check --workspace --lib --tests
	cargo clippy --workspace -- "${CLIPPY_OPTIONS}"

test: check
	cargo test --workspace

fmt:
	cargo fmt --all --verbose

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
	cargo clean
