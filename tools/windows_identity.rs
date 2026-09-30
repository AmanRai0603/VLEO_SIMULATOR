// Give a Windows program its identity: who made it, what it is, which version.
//
// Included by the build scripts of the programs and the Python module that
// ship to Windows. A program with no version information — no company, no
// product, no description — is one of the plainest signs an antivirus
// heuristic weighs against an unknown file, and a person looking at the file's
// Properties sees nothing to tell them what it is. This is not a signature and
// claims no trust; it only says what the file is.
//
// Nothing happens off Windows. The resource is written into OUT_DIR from the
// crate's own version, so it never drifts from the release, and compiled with
// the Windows SDK's resource compiler through `embed-resource`.

#[allow(dead_code)]
enum Kind {
    Program,
    Library,
}

#[allow(dead_code)]
fn windows_identity(file: &str, description: &str, kind: Kind) {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let version = std::env::var("CARGO_PKG_VERSION").expect("CARGO_PKG_VERSION");
    let mut nums: Vec<u16> = version
        .split(['.', '-'])
        .take(3)
        .map(|n| n.parse().unwrap_or(0))
        .collect();
    nums.resize(4, 0);
    let dotted = format!("{},{},{},{}", nums[0], nums[1], nums[2], nums[3]);
    let (file_type, extension) = match kind {
        Kind::Program => ("0x1", "exe"),
        Kind::Library => ("0x2", "dll"),
    };
    let rc = format!(
        r#"1 VERSIONINFO
FILEVERSION {dotted}
PRODUCTVERSION {dotted}
FILEFLAGSMASK 0x3f
FILEFLAGS 0x0
FILEOS 0x40004
FILETYPE {file_type}
FILESUBTYPE 0x0
BEGIN
  BLOCK "StringFileInfo"
  BEGIN
    BLOCK "040904b0"
    BEGIN
      VALUE "CompanyName", "Orbitt Space"
      VALUE "FileDescription", "{description}"
      VALUE "FileVersion", "{version}"
      VALUE "InternalName", "{file}"
      VALUE "LegalCopyright", "Copyright (C) Orbitt Space"
      VALUE "OriginalFilename", "{file}.{extension}"
      VALUE "ProductName", "VLEO design tool"
      VALUE "ProductVersion", "{version}"
    END
  END
  BLOCK "VarFileInfo"
  BEGIN
    VALUE "Translation", 0x409, 1200
  END
END
"#
    );
    let out = std::path::Path::new(&std::env::var("OUT_DIR").expect("OUT_DIR")).join("identity.rc");
    std::fs::write(&out, rc).expect("identity.rc");
    let linked = match kind {
        Kind::Program => embed_resource::compile(&out, embed_resource::NONE),
        Kind::Library => embed_resource::compile_for_cdylib(&out, embed_resource::NONE),
    };
    linked
        .manifest_optional()
        .expect("the Windows resource compiler could not build the version information");
}
