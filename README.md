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
help explore
```
- run the command
```nushell
open Cargo.toml | explore
```

# configuration
there is currently no way to use the `$env` configuration from inside a plugin...
`nu_plugin_explore` thus uses a CLI argument to do that, i.e. you can pass a config record as the
first positional argument to the `explore` command!

however, doing this by hand each time is not the right way to go with that, so let's find another way.

here is the default config, put it in your `config.nu` :yum:
```nushell
$env.explore_config = {
    show_cell_path: true,  # whether or not to show the current cell path above the status bar
    show_table_header: true,  # whether or not to show the table header in "table" layout
    layout: "table",  # the layout of the data, either "table" or "compact"

    # "reset" is used instead of "black" in a dark terminal because, when the terminal is actually
    # black, "black" is not really black which is ugly, whereas "reset" is really black.
    colors: {
        normal: {  # the colors for a normal row
            name: {
                background: reset,
                foreground: green,
            },
            data: {
                background: reset,
                foreground: white,
            },
            shape: {
                background: reset,
                foreground: blue,
            },
        },
        selected: {  # the colors for the row under the cursor
            background: white,
            foreground: black,
        },
        selected_modifier: "bold",  # a modifier to apply onto the row under the cursor
        selected_symbol: "",  # the symbol to show to the left of the row under the cursor
        status_bar: {
            normal: {  # the colors for the status bar in NORMAL mode
                background: black,
                foreground: white,
            },
            insert: {  # the colors for the status bar in INSERT mode
                background: black,
                foreground: lightyellow,
            },
            peek: {  # the colors for the status bar in PEEKING mode
                background: black,
                foreground: lightgreen,
            }
            bottom: {  # the colors for the status bar in BOTTOM mode
                background: black,
                foreground: lightmagenta,
            }
        }
        editor: {  # the colors when editing a cell
            frame: {
                background: black,
                foreground: lightcyan,
            },
            buffer: {
                background: reset,
                foreground: white,
            },
        },
    }
    keybindings: {
        quit: 'q',  # quit `explore`
        insert: 'i',  # go to INSERT mode to modify the data
        normal: "escape",  # go back to NORMAL mode to navigate through the data
        navigation: {  # only in NORMAL mode
            left: 'h',  # go back one level in the data
            down: 'j',  # go one row down in the current level
            up: 'k',  # go one row up in the current level
            right: 'l',  # go one level deeper in the data or hit the bottom
        },
        peek: 'p',  # go to PEEKING mode to peek a value
        peeking: {  # only in PEEKING mode
            all: 'a',  # peek the whole data, from the top level
            cell_path: 'c',  # peek the cell path under the cursor
            under: 'p',  # peek only what's under the cursor
            view: 'v',  # peek the current view, i.e. what is visible
        },
    }
}
```

then in order to avoid having to pass this record everytime we call `explore`, let's define an alias
> **Note**  
> this will not have an impact on the CLI interface of `explore` because it does not have any other
> option or argument than `config: record` as the first positional argument.

```nushell
alias explore = explore ($env.explore_config? | default {})
```

now, you can just call `explore` and have your config loaded automatically!
and you can change `$env.explore_config` as much as you like :partying_face:

> **Note**  
> if you omit one of the config field of the configuration for `explore`, it's not an issue at all,
> it will just take the default value instead!

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
