use std log


export def main [package_file: path = nupm.nuon --debug] {
    let repo_root = $package_file | path dirname
    let install_root = $env.NUPM_HOME | path join "plugins"

    let name = open ($repo_root | path join "Cargo.toml") | get package.name

    alias install = cargo install --path $repo_root --root $install_root
    if $debug { install --debug } else { install }
    ^$nu.current-exe --commands $"register ($install_root | path join "bin" $name)"

    log info "do not forget to restart Nushell for the plugin to be fully available!"
}
