//! The desktop app's Windows identity and icon. See tools/windows_identity.rs.

include!("../../tools/windows_identity.rs");

fn main() {
    println!("cargo:rerun-if-changed=../../tools/windows_identity.rs");
    println!("cargo:rerun-if-changed=icon/vleo.ico");
    windows_identity_with_icon(
        "VLEO Design Tool",
        "VLEO Design Tool",
        Kind::Program,
        Some("icon/vleo.ico"),
    );
}
