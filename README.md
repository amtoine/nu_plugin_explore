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
    - [*demo*](#demo)
- [*configuration*](#configuration)
    - [*default configuration*](#default-configuration)
    - [*an example*](#an-example)
    - [*some convenience*](#-some-convenience)
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
## requirements
> **Note**  
> this is the development version of `nu_plugin_explore`, thus it does not require Nushell to be
> installed with a stable version.

let's setup the Nushell dependencies locally, because `nu-plugin` and `nu-protocol` are not release
in version `0.xx.1`, only the stable `0.xx.0` :open_mouth:

- clone the [Nushell repository][nushell/nushell] somewhere
- setup the dependencies
```shell
make dev-deps
```

there are three ways to do it:
## building from source
- build the plugin
```shell
make build
```
- register the plugin in [Nushell]
```nushell
make register
```
- do not forget to restart [Nushell]

> **Note**  
> alternatively, you can use directly `make install`

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
help nu_plugin_explore
```
- run the command
```nushell
open Cargo.toml | nu_plugin_explore
```

## demo
![simple demo](examples/demo.gif)

# configuration
## default configuration
you can find it in [`default.nuon`](./examples/config/default.nuon).

you can copy-paste it in your `config.nu` and set `$env.explore_config` to it:
```nushell
$env.config.plugins.explore = {
    # content of the default config
}
```
alternately, you can copy-paste the default config file to `$nu.default-config-dir` and add the following line to your `config.nu`
```nushell
$env.config.plugins.explore = (open ($nu.default-config-dir | path join "nu_plugin_explore.nu"))
```

## an example
if you do not like the Vim bindings by default you can replace the navigation part with
```nushell
$env.config.plugins.explore.keybindings.navigation = {
    left: 'left',
    down: 'down',
    up: 'up',
    right: 'right',
}
```
and voila :yum:

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
  - [x] string cells
  - [ ] other simple cells
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
[nushell/nushell]: https://github.com/nushell/nushell
[`nu-explore`]: https://crates.io/crates/nu-explore

[`nu-plugin`]: https://crates.io/crates/nu-plugin
[`nu-protocol`]: https://crates.io/crates/nu-protocol
[crates.io]: https://crates.io
