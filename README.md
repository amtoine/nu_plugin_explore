# nu_plugin_explore
A fast *interactive explorer* tool for *structured data* inspired by [`nu-explore`]

# table of content
- [*introduction*](#introduction)
  - [*the idea behind an explorer*](#the-idea-behind-an-explorer)
  - [*why not `nu-explore`?*](#why-not-nu-explore)
- [*installation*](#installation)
  - [*setup the Nushell dependencies*](#setup-the-Nushell-dependencies)
  - [*install the plugin*](#install-the-plugin)
    - [*building from source*](#building-from-source)
    - [*installing manually*](#installing-manually)
    - [*using `nupm install`*](#using-nupm-install)
- [*usage*](#usage)
- [*configuration*](#configuration)
- [*see the documentation locally*](#see-the-documentation-locally)
- [*TODO*](#todo)

# introduction
## the idea behind an *explorer*
i think having an *interactive explorer* for *structured data* is a requirement for a shell like
[Nushell]!  
the ability to
- traverse the data with a few quick key bindings
- peek the data at any level
- edit the data on the fly (COMING SOON)
- all while being configurable

will come very handy in a day-to-day basis for me at least :)

## why not `nu-explore`?
- it's a bit too complex for what it does to me
- the bindings are 
- the code was really hard to wrap my head around
- i wanted to have fun learning about [Nushell] plugins and TUI applications in Rust

so here we are... LET'S GO :muscle:

# installation
> **Note**  
> the plugin has been written with the latest revision of [Nushell]

## setup the Nushell dependencies
as one can see in the [`Cargo.toml`](Cargo.toml) file, the [Nushell] dependencies are empty
and need to be setup.  
the reason is that the latest revision of [`nu-plugin`] and [`nu-protocol`] are not on [crates.io]

to setup these dependencies, please download the source code of [Nushell], e.g.
```nushell
git clone https://github.com/nushell/nushell /path/to/nushell
```

> **Note**  
> this is a little variable to make the following script simpler
> ```nushell
> let nushell: path = /path/to/nushell
> ```

finally run the following to setup the dependencies
```nushell
cargo add nu-plugin --path ($nushell | path join "crates" "nu-plugin")
cargo add nu-protocol --path ($nushell | path join "crates" "nu-protocol") --features plugin
```

## install the plugin
now that the dependencies are all setup, we can install the plugin: there are threee ways to do it

### building from source
- build the plugin
```nushell
cargo build --release
```
- register the plugin in [Nushell]
```nushell
register target/release/nu_plugin_explore
```
- do not forget to restart [Nushell]

### installing manually
- define the install root, e.g. `$env.CARGO_HOME` or `/some/where/plugins/`
```nushell
let install_root: path = ...
```
- build and install the plugin
```nushell
cargo install --path . --root $install_root
```
- register the plugin in [Nushell]
```nushell
nu --commands $"register ($install_root | path join "bin" $name)"
```
- do not forget to restart [Nushell]

### using `nupm install`
> **Warning**  
> this is a very alpha software and even requires to use the code from an unmerged PR :eyes:

> **Note**  
> this method does not even require to change the [`Cargo.toml`](Cargo.toml) as advertised in
> [*setup the Nushell dependencies*](#setup-the-nushell-dependencies)

- pull down the `nupm` module from [nushell/nupm#12](https://github.com/nushell/nupm/pull/12)
- load the `nupm` module
```nushell
use /path/to/nupm/
```
- run the install process
```nushell
nupm install --path .
```

# usage
- get some help
```nushell
help explore
```
- run the command
```nushell
open Cargo.toml | explore
```

# configuration
the default configuration can be found in [`examples/configuration`](examples/configuration) and can
be tested as follows
```nushell
$nu | explore (
    open examples/configuration/config.nuon
        | insert colors (open examples/configuration/themes/dark.nuon)
        | insert keybindings (open examples/configuration/keybindings.nuon)
)
```

# see the documentation locally
```nushell
cargo doc --document-private-items --no-deps --open
```

# TODO
- [ ] get rid of the `.clone`s
- [ ] handle errors properly (`.unwrap`s and `panic!`s)
- [ ] get the config from `$env.config` => can parse configuration from CLI
- [x] support non-character bindings
- [ ] add check for the config to make sure it's valid
- [ ] when going into a file or URL, open it
- [ ] give different colors to names and type
- [ ] add tests...
  - [ ] to `navigation.rs` to make sure the navigation in the data is ok
  - [ ] to `app.rs` to make sure the application state machine works
  - [ ] to `parsing.rs` to make sure the parsing of the config works
  - [ ] to `tui.rs` to make sure the rendering works as intended
- [ ] restrict the visibility of objects when possible
- [ ] show true tables as such

[Nushell]: https://nushell.sh
[`nu-explore`]: https://crates.io/crates/nu-explore

[`nu-plugin`]: https://crates.io/crates/nu-plugin
[`nu-protocol`]: https://crates.io/crates/nu-protocol
[crates.io]: https://crates.io
