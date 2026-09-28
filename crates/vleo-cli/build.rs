//! The command line's Windows identity (company, product, version). See
//! tools/windows_identity.rs.

include!("../../tools/windows_identity.rs");

fn main() {
    println!("cargo:rerun-if-changed=../../tools/windows_identity.rs");
    windows_identity("vleo", "VLEO design tool, command line", Kind::Program);
}
