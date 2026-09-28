//! `vleo-daemon` — the local engine, serving its own interface. The whole of
//! it is the library (`lib.rs`), so the Python package can start the same
//! server; this is the command line's door onto it.

fn main() {
    vleo_daemon::main();
}
