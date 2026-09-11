//! An incomplete bundle is refused, not warned about.
//!
//! The rule these tests hold to is the one in the working model: publishing
//! without a hash, a provenance or a licence should be *impossible*, not
//! forbidden. A forbidden thing is a sentence in a document that somebody
//! skips on the day they are in a hurry, and the bundle that results looks
//! exactly like a good one. So each test here takes a manifest that is known
//! to load, breaks exactly one field, and checks that the break is what gets
//! refused — a checker nobody has watched fail is a checker nobody knows
//! works.

use std::fs;
use std::path::PathBuf;

/// A manifest that loads. Every test below is this, with one line changed.
const GOOD: &str = r#"
name = "test-bundle"
version = "2026.01.01"
provenance = "noaa_swpc"
licence_until = "2031-12-31"
stale_after_days = 120
files = ["payload.csv"]
content_hash = "0000000000000000"
"#;

/// Write a bundle whose manifest is `GOOD` with `field` replaced by `line`
/// (or removed entirely, when `line` is `None`), and return its directory.
fn bundle(tag: &str, field: &str, line: Option<&str>) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("vleo-data-refusal-{tag}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let mut out = String::new();
    for l in GOOD.lines() {
        if l.split('=').next().unwrap_or("").trim() == field {
            if let Some(replacement) = line {
                out.push_str(replacement);
                out.push('\n');
            }
        } else {
            out.push_str(l);
            out.push('\n');
        }
    }
    fs::write(dir.join("manifest.toml"), out).unwrap();
    fs::write(dir.join("payload.csv"), "day,f107\n0,150\n").unwrap();
    dir
}

/// The control. Without this the refusals below prove nothing: a loader that
/// refused everything would pass every other test on this page.
#[test]
fn a_complete_manifest_loads() {
    let dir = bundle("good", "___none___", None);
    let b = vleo_data::load_bundle(&dir).expect("a complete manifest loads");
    assert_eq!(b.manifest.provenance, "noaa_swpc");
    assert_eq!(b.manifest.stale_after_days, 120);
    // It loads, but it is not usable: the placeholder hash does not match the
    // payload, so nothing downstream will touch it until `bundle publish`
    // writes the real one.
    assert!(!b.verified, "a placeholder hash must not verify");
}

#[test]
fn no_provenance_is_refused() {
    let dir = bundle("prov", "provenance", None);
    let e = vleo_data::load_bundle(&dir).expect_err("a manifest with no provenance fails");
    assert!(e.contains("provenance"), "{e}");
}

#[test]
fn an_empty_provenance_is_refused_too() {
    // The field being present is not the point; a blank one is the same gap
    // with a line of TOML in front of it.
    let dir = bundle("prov-blank", "provenance", Some(r#"provenance = "   ""#));
    let e = vleo_data::load_bundle(&dir).expect_err("a blank provenance fails");
    assert!(e.contains("provenance"), "{e}");
}

#[test]
fn no_licence_is_refused() {
    let dir = bundle("lic", "licence_until", None);
    let e = vleo_data::load_bundle(&dir).expect_err("a manifest with no licence fails");
    assert!(e.contains("licence_until"), "{e}");
}

#[test]
fn a_licence_that_is_not_a_date_is_refused() {
    // The failure worth stopping: `"soon"` parses as TOML, reads as an intent,
    // and expires on no day at all.
    let dir = bundle(
        "lic-soon",
        "licence_until",
        Some(r#"licence_until = "soon""#),
    );
    let e = vleo_data::load_bundle(&dir).expect_err("a non-date licence fails");
    assert!(e.contains("YYYY-MM-DD"), "{e}");
}

#[test]
fn a_missing_staleness_is_refused_rather_than_defaulted() {
    // Silently defaulting to zero would mean every bundle is either always
    // fresh or always stale, decided by a number nobody chose.
    let dir = bundle("stale", "stale_after_days", None);
    let e = vleo_data::load_bundle(&dir).expect_err("a missing staleness fails");
    assert!(e.contains("stale_after_days"), "{e}");
}

#[test]
fn a_manifest_listing_no_files_is_refused() {
    let dir = bundle("nofiles", "files", Some("files = []"));
    let e = vleo_data::load_bundle(&dir).expect_err("a manifest with no files fails");
    assert!(e.contains("nothing to hash"), "{e}");
}

#[test]
fn no_name_and_no_version_are_refused() {
    for (tag, field) in [("name", "name"), ("version", "version")] {
        let dir = bundle(tag, field, None);
        let e = vleo_data::load_bundle(&dir).unwrap_err();
        assert!(e.contains(field), "{field}: {e}");
    }
}

/// The hash is what the other refusals protect. A byte changed under a
/// published manifest must not produce a degraded result; it must produce no
/// result.
#[test]
fn a_tampered_byte_does_not_verify() {
    let dir = bundle("tamper", "___none___", None);
    let real = vleo_data::hash_files(&dir, &["payload.csv".to_string()]).unwrap();
    let text = fs::read_to_string(dir.join("manifest.toml"))
        .unwrap()
        .replace("0000000000000000", &real);
    fs::write(dir.join("manifest.toml"), text).unwrap();
    assert!(
        vleo_data::load_bundle(&dir).unwrap().verified,
        "the honest bundle verifies"
    );

    fs::write(dir.join("payload.csv"), "day,f107\n0,151\n").unwrap();
    let b = vleo_data::load_bundle(&dir).unwrap();
    assert!(!b.verified, "one changed digit must break the hash");
    assert!(
        b.refusal.unwrap().contains(&real),
        "the refusal says both hashes"
    );
}
