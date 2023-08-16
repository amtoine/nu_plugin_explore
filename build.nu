use std log

let repo_root = $env.CURRENT_FILE | path dirname
let install_root = $env.NUPM_HOME | path join "plugins"
let name = open Cargo.toml | get package.name

cargo install --path $repo_root --root $install_root
nu --commands $"register ($install_root | path join "bin" $name)"

log info "do not forget to restart Nushell for the plugin to be fully available!"
