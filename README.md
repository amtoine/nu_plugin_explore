# nu_plugin_explore
A fast structured data explorer for Nushell.

## setup the Nushell dependencies
run
```nushell
let nushell: path = ...
```
```nushell
open Cargo.toml
    | update dependencies.nu-plugin.path ($nushell | path join "crates" "nu-plugin")
    | update dependencies.nu-protocol.path ($nushell | path join "crates" "nu-protocol")
    | save --force Cargo.toml
```

## install the plugin
```nushell
cargo build --release
```
```nushell
register target/release/nu_plugin_explore
```

## TODO
- [ ] get rid of the `.clone`s
- [ ] handle errors properly (`.unwrap`s and `panic!`s)
- [ ] get the config from `$env.config` => can parse configuration from CLI
- [x] support non-character bindings
- [ ] add check for the config to make sure it's valid
- [ ] when going into a file or URL, open it
- [ ] give different colors to names and type
- [ ] add tests
- [ ] restrict the visibility of objects when possible
- [ ] show true tables as such
