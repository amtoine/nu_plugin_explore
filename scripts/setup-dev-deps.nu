#!/usr/bin/env nu
use std log

let cargo_nu = cargo install --list | lines | parse --regex '^nu (v[0-9.]*) \((?<path>.*)\)'

let nushell_root = if ($cargo_nu | is-empty) {
    if "NUSHELL_PATH" not-in $env {
        error make --unspanned {
            msg: "it appears `nu` is not installed with `cargo`, please set `NUSHELL_PATH` manually"
        }
    }

    log info "using `$env.NUSHELL_PATH` instead of install directory"

    $env.NUSHELL_PATH
} else {
    $cargo_nu | get path.0
}

cargo add nu-plugin --path ($nushell_root | path join "crates/nu-plugin")
cargo add nu-protocol --path ($nushell_root | path join "crates/nu-protocol") --features plugin
