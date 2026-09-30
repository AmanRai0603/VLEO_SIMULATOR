//! `vleo-app` — the VLEO Design Tool as a desktop program.
//!
//! Double-clicked, it starts the tool and opens it in the browser. On Windows
//! it is a windowed program, so no console window opens beside it; there is
//! nothing to close, because it ends by itself once no page has been open for
//! a few minutes, or at once from the page's Quit. See `vleo_server::app`.
#![cfg_attr(windows, windows_subsystem = "windows")]

fn main() {
    vleo_server::app();
}
