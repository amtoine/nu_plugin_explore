use nu_plugin::{serve_plugin, MsgPackSerializer};
use nu_plugin_explore::Explore;

fn main() {
    serve_plugin(&mut Explore{}, MsgPackSerializer {})
}
