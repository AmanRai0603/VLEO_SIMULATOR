//! The copy's record (docs/PLAN_F.md, decision 3): where a working copy came
//! from, and every file as it was copied, written and read back, refused when
//! it names a path outside the database, and asked what changed since.

use std::collections::BTreeMap;
use vleo_files::copied::{self, Copied};

fn s(v: &str) -> String {
    v.to_string()
}

fn record() -> Copied {
    let mut files = BTreeMap::new();
    files.insert(s("START HERE.md"), copied::digest(b"the anchor"));
    files.insert(
        s("groups/l3_solar/l3_solar.vgroup"),
        copied::digest(b"solar"),
    );
    Copied {
        drive: s("/mnt/drive/vleo \"database\""),
        copied_at: s("2026-10-15T08:00:00Z"),
        files,
    }
}

#[test]
fn the_record_reads_back_as_it_was_written() {
    let r = record();
    assert_eq!(Copied::from_toml(&r.to_toml()).unwrap(), r);
}

#[test]
fn a_record_names_only_paths_inside_the_database() {
    for bad in [
        "../outside",
        "/etc/passwd",
        "groups//x",
        "groups/./x",
        "C:x",
        "groups\\x",
        "",
    ] {
        assert!(!copied::path_ok(bad), "{bad:?}");
        let mut r = record();
        r.files.insert(s(bad), copied::digest(b"x"));
        let e = Copied::from_toml(&r.to_toml()).unwrap_err();
        assert!(
            e.message().contains("not a path inside the database"),
            "{bad:?}: {}",
            e.message()
        );
    }
    let text = record().to_toml().replace(&copied::digest(b"solar"), "abc");
    let e = Copied::from_toml(&text).unwrap_err();
    assert!(e.message().contains("not a SHA-256"), "{}", e.message());
    let text = record()
        .to_toml()
        .replace("2026-10-15T08:00:00Z", "this morning");
    assert!(Copied::from_toml(&text).is_err());
}

#[test]
fn what_changed_since_the_copy_is_every_file_changed_added_or_gone() {
    let r = record();
    let mut now = r.files.clone();
    assert!(r.changed(&now).is_empty());
    now.insert(
        s("groups/l3_solar/l3_solar.vgroup"),
        copied::digest(b"solar, signed"),
    );
    now.insert(s("groups/l3_prop/l3_prop.vgroup"), copied::digest(b"new"));
    now.remove("START HERE.md");
    assert_eq!(
        r.changed(&now),
        vec![
            s("START HERE.md"),
            s("groups/l3_prop/l3_prop.vgroup"),
            s("groups/l3_solar/l3_solar.vgroup"),
        ]
    );
}
