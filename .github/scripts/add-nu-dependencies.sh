nushell="/tmp/nushell"

git clone https://github.com/nushell/nushell "$nushell"
cargo add nu-plugin --path "$nushell/crates/nu-plugin"
cargo add nu-protocol --path "$nushell/crates/nu-protocol" --features plugin
