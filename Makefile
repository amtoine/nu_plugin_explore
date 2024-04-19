CLIPPY_OPTIONS="-D warnings"

.PHONY: fmt-check fmt check lock clippy test doc build register install clean
DEFAULT: fmt-check check lock clippy test

fmt-check:
	cargo fmt --all --verbose -- --check --verbose
fmt:
	cargo fmt --all --verbose

check:
	cargo check --workspace --lib --tests

lock: check
	./.github/workflows/scripts/check-cargo-lock.sh

clippy:
	cargo clippy --workspace -- "${CLIPPY_OPTIONS}"

test:
	cargo test --workspace

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
