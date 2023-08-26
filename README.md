# nu_plugin_explore
A fast *interactive explorer* tool for *structured data* inspired by [`nu-explore`]

# table of content
- [*introduction*](#introduction)
  - [*the idea behind an explorer*](#the-idea-behind-an-explorer)
  - [*why not `nu-explore`?*](#why-not-nu-explore)
- [*installation*](#installation)
  - [*building from source*](#building-from-source)
  - [*installing manually*](#installing-manually)
  - [*using `nupm install` (recommended)*](#using-nupm-install-recommended)
- [*usage*](#usage)
- [*configuration*](#configuration)
- [*see the documentation locally*](#see-the-documentation-locally)
- [*contributing*](#contributing)
- [*TODO*](#todo)
  - [*features*](#features)
  - [*internal*](#internal)

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
- the bindings are not configurable
- the code was really hard to wrap my head around
- i wanted to have fun learning about [Nushell] plugins and TUI applications in Rust

so here we are... LET'S GO :muscle:

# installation
there are three ways to do it:
## building from source
- build the plugin
```nushell
cargo build --release
```
- register the plugin in [Nushell]
```nushell
register target/release/nu_plugin_explore
```
- do not forget to restart [Nushell]

## installing manually
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

## using `nupm install` (recommended)
> **Warning**  
> this is a very alpha software

- download [nushell/nupm](https://github.com/nushell/nupm)
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

# contributing
in order to help, you can have a look at
- the [todo](#todo) list down below, there might be unticked tasks to tackle
- the issues and bugs in the [issue tracker](https://github.com/amtoine/nu_plugin_explore/issues)
- the `FIXME` and `TODO` comments in the source base

# TODO
## features
- [x] support non-character bindings
- [ ] when going into a file or URL, open it
- [x] give different colors to names and type
- [x] show true tables as such
- [ ] get the config from `$env.config` => can parse configuration from CLI
- [ ] add check for the config to make sure it's valid
- [ ] support for editing cells in INSERT mode
- [x] detect if a string is of a particular type, path, URL, ...

## internal
- [x] add tests...
  - [x] to `navigation.rs` to make sure the navigation in the data is ok
  - [x] to `app.rs` to make sure the application state machine works
  - [x] to `parsing.rs` to make sure the parsing of the config works
  - [x] to `tui.rs` to make sure the rendering works as intended
- [ ] get rid of the `.clone`s
- [ ] handle errors properly (`.unwrap`s and `panic!`s)
- [ ] restrict the visibility of objects when possible
- [ ] write better error messages when some test fails

[Nushell]: https://nushell.sh
[`nu-explore`]: https://crates.io/crates/nu-explore

[`nu-plugin`]: https://crates.io/crates/nu-plugin
[`nu-protocol`]: https://crates.io/crates/nu-protocol
[crates.io]: https://crates.io
