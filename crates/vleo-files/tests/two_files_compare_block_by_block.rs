//! Any two files, compared block by block.
//!
//! The oracle is Solar 1.0 against Solar 1.1, read with Python's sqlite3 and
//! nothing of this library (docs/PLAN_1_0.md, phase C): between the two
//! seals, six nodes gained the cases that make them refuse (their
//! `results/isolation.csv`), `sw_kp_scenarios` also changed its method, and
//! the group's own `versions.csv` gained the record of why. Nothing else in
//! the content moved. Both files are upgraded first, as the application
//! opens them, so this is the comparison of two format-2 files.

use std::collections::BTreeSet;
use std::path::Path;

use vleo_files::compare::{self, Difference, Status};
use vleo_files::model::File;
use vleo_files::sqlite;
use vleo_files::upgrade::{self, Upgrade};

const HOW: Upgrade = Upgrade {
    app: "vleo 1.0.0 (test)",
    at: "2026-10-06T09:00:00Z",
};

fn solar(version: &str) -> File {
    let p = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("tests/fixtures/l3_solar-{version}.vleo"));
    let old = sqlite::read_format_1(&p).unwrap();
    upgrade::from_format_1(old.to_tables(), &HOW).unwrap().file
}

#[test]
fn solar_1_0_to_1_1_is_what_python_finds() {
    let (a, b) = (solar("1.0"), solar("1.1"));
    let c = compare::compare(&a, &b);
    let changed: BTreeSet<&str> = c.blocks.iter().map(|b| b.id()).collect();
    assert_eq!(
        changed,
        BTreeSet::from([
            "sw_activity_band",
            "sw_ap_daily_band_drop",
            "sw_ap_design",
            "sw_exceedance_rate",
            "sw_horizon_persistence",
            "sw_kp_scenarios",
            "sw_regime",
        ]),
        "{c:#?}"
    );
    assert!(c.blocks.iter().all(|b| b.status == Status::Changed));
    assert_eq!(c.same + c.blocks.len(), b.blocks.len());
    // How many lines each text gained and lost, as Python's difflib counts
    // them: each of the six gained one case, the one that must be refused;
    // sw_kp_scenarios rewrote its method and its cases.
    let lines = |id: &str, table: &str| -> (usize, usize) {
        let b = c.blocks.iter().find(|b| b.id() == id).unwrap();
        b.differences
            .iter()
            .find_map(|d| match d {
                Difference::Changed {
                    table: t,
                    before,
                    after,
                    ..
                } if t == table => {
                    let (gone, came) = compare::lines_changed(before, after);
                    Some((came.len(), gone.len()))
                }
                _ => None,
            })
            .unwrap_or_else(|| panic!("{id}: no {table} difference: {:#?}", b.differences))
    };
    for id in [
        "sw_activity_band",
        "sw_ap_daily_band_drop",
        "sw_ap_design",
        "sw_exceedance_rate",
        "sw_horizon_persistence",
        "sw_regime",
    ] {
        assert_eq!(lines(id, "tbl"), (1, 0), "{id}");
        let b = c.blocks.iter().find(|b| b.id() == id).unwrap();
        assert_eq!(b.differences.len(), 1, "{id}: {:#?}", b.differences);
        assert!(b.differences[0].says().starts_with(
            "tbl results/isolation.csv: csv, 1 line(s) added and 0 removed, first “NaN,"
        ));
    }
    assert_eq!(lines("sw_kp_scenarios", "tbl"), (6, 6));
    assert_eq!(lines("sw_kp_scenarios", "text"), (12, 2));
    // The group's own: versions.csv, and the meta that names the version.
    let file: Vec<String> = c.file.iter().map(Difference::says).collect();
    assert!(file.iter().any(|d| d.contains("versions.csv")), "{file:#?}");
    assert!(
        file.iter()
            .all(|d| d.starts_with("tbl versions.csv") || d.starts_with("meta ")),
        "{file:#?}"
    );
    assert!(c
        .summary()
        .starts_with("7 block(s) changed, 0 added, 0 removed"));
}

#[test]
fn a_file_against_itself_is_the_same() {
    let a = solar("1.1");
    let c = compare::compare(&a, &a);
    assert!(c.is_same(), "{c:#?}");
    assert_eq!(c.same, a.blocks.len());
    assert!(c.history.iter().all(|(_, x, y)| *x == 0 && *y == 0));
}

#[test]
fn a_renamed_block_is_one_block_and_a_new_one_is_added() {
    let a = solar("1.1");
    let mut b = a.clone();
    let uid = b.blocks[0].uid.clone();
    let was = b.blocks[0].id.clone();
    b.blocks[0].id = format!("{was}_renamed");
    let mut extra = b.blocks[1].clone();
    extra.uid = "brand_new_uid".into();
    extra.id = "brand_new".into();
    b.blocks.push(extra);
    let port = b.ports.iter_mut().find(|p| p.block_uid == uid);
    let port_name = port.as_ref().map(|p| p.name.clone());
    if let Some(p) = port {
        p.unit = "furlong".into();
    }
    let c = compare::compare(&a, &b);
    let renamed = c.blocks.iter().find(|x| x.uid == uid).unwrap();
    assert_eq!(renamed.status, Status::Changed);
    assert_eq!(renamed.before.as_deref(), Some(was.as_str()));
    assert_eq!(renamed.after, Some(format!("{was}_renamed")));
    assert!(renamed.differences.iter().any(|d| matches!(d,
        Difference::Changed { table, column, .. } if table == "block" && column == "id")));
    if let Some(name) = port_name {
        assert!(
            renamed
                .differences
                .iter()
                .any(|d| d.says().starts_with("port ")
                    && d.says().contains(&name)
                    && d.says().contains("unit")
                    && d.says().contains("“furlong”")),
            "{:#?}",
            renamed.differences
        );
    }
    let added = c.blocks.iter().find(|x| x.uid == "brand_new_uid").unwrap();
    assert_eq!(
        (added.status, added.before.as_deref()),
        (Status::Added, None)
    );
    // The other way round, the new block is removed.
    let back = compare::compare(&b, &a);
    let gone = back
        .blocks
        .iter()
        .find(|x| x.uid == "brand_new_uid")
        .unwrap();
    assert_eq!(gone.status, Status::Removed);
}

#[test]
fn every_table_of_the_schema_is_compared_or_counted() {
    let schema = include_str!("../src/schema.sql");
    let tables: BTreeSet<&str> = schema
        .lines()
        .filter_map(|l| l.strip_prefix("CREATE TABLE "))
        .map(|l| l.split_whitespace().next().unwrap())
        .collect();
    let file = File::default();
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let c = compare::compare(&file, &file);
    for (t, _, _) in &c.history {
        seen.insert(t.clone());
    }
    for t in compare::CONTENT_TABLES {
        seen.insert(t.to_string());
    }
    let seen: BTreeSet<&str> = seen.iter().map(String::as_str).collect();
    assert_eq!(seen, tables);
}
