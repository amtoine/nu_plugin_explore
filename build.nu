use std log

let repo_root = $env.CURRENT_FILE | path dirname
let install_root = $env.NUPM_HOME | path join "plugins"

let cargo_toml = $repo_root | path join "Cargo.toml"

let name = open $cargo_toml | get package.name

if $env.NUSHELL_SOURCE_PATH? == null {
    error make --unspanned {
        msg: $"(ansi red_bold)$env.NUSHELL_SOURCE_PATH is not set(ansi reset), please set the environment variable and run `nupm install` again."
    }
}

log debug "setting nu-plugin and nu-protocol"
open $cargo_toml
    | update dependencies.nu-plugin.path ($env.NUSHELL_SOURCE_PATH | path join "crates" "nu-plugin")
    | update dependencies.nu-protocol.path (
        $env.NUSHELL_SOURCE_PATH | path join "crates" "nu-protocol"
    )
    | save --force $cargo_toml

cargo install --path $repo_root --root $install_root
nu --commands $"register ($install_root | path join "bin" $name)"

log debug "unsetting nu-plugin and nu-protocol"
open $cargo_toml
    | update dependencies.nu-plugin.path ""
    | update dependencies.nu-protocol.path ""
    | save --force $cargo_toml

log info "do not forget to restart Nushell for the plugin to be fully available!"
