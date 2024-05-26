const NUSHELL_REMOTE = "https://github.com/nushell/nushell"
const PKGS = [ "nuon", "nu-protocol", "nu-plugin" ]

# setup Nushell dependencies
#
# > **Note**
# > options are shown in inverse order of precedence
export def main [
    --rev: string, # a Nushell revision
    --tag: string, # a Nushell tag
    --current,     # use the current Nushell version
] {
    let opts = if $current {
        if (version).commit_hash != null {
            print $"using current revision of Nushell: (version | get commit_hash)"
            [ --rev (version).commit_hash ]
        } else {
            print $"using current version of Nushell: (version | get version)"
            [ --tag (version).version ]
        }
    } else {
        if $tag != null {
            print $"using user version: ($tag)"
            [ --tag $tag ]
        } else if $rev != null {
            print $"using user revision: ($rev)"
            [ --rev $rev ]
        } else {
            error make --unspanned { msg: "please give either `--rev` or `--tag`" }
        }
    }

    for pkg in $PKGS {
        cargo add $pkg --git $NUSHELL_REMOTE ...$opts
    }
}
