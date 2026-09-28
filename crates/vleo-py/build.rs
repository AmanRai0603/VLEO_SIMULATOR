//! The Python module's build: its Windows identity, and on macOS the link
//! arguments every Python extension needs — Python's own symbols are found
//! when the interpreter loads the module, not when it is linked.

include!("../../tools/windows_identity.rs");

fn main() {
    println!("cargo:rerun-if-changed=../../tools/windows_identity.rs");
    windows_identity("_vleo", "VLEO design tool, Python module", Kind::Library);
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        println!("cargo:rustc-cdylib-link-arg=-undefined");
        println!("cargo:rustc-cdylib-link-arg=dynamic_lookup");
    }
}
