cargo install --path ($env.CURRENT_FILE | path dirname) --root ($env.NUPM_HOME | path join "plugins")
nu --commands $"register ($env.NUPM_HOME | path join "plugins" "bin" "nu_plugin_explore")"
