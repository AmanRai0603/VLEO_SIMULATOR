//! `vleo-daemon` — the local engine, serving its own interface. The whole of
//! it is `vleo-server`, which the Python package starts too; this is the
//! command line's door onto it.

fn main() {
    vleo_server::main();
}
