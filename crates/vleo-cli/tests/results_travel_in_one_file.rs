//! Saved results go out in one file and come back as they were.
//!
//! `vleo results export` writes a results folder into one `.vleor` file and
//! `vleo results import` puts it into another. What comes back must be the
//! same results byte for byte — a result is a record, and a copy that differs
//! from it is a different record under the same name — and importing twice
//! must keep each question once. The file's `value` rows must carry the same
//! numbers the results do, because that is what a Python or SQL reader reads.

use std::path::{Path, PathBuf};
use std::process::Command;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn vleo(scratch: &Path, results: &Path, args: &[&str]) -> (bool, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_vleo"))
        .args(args)
        .current_dir(root())
        .env("VLEO_DATA", scratch.join("data"))
        .env("VLEO_CASE", scratch.join("case.csv"))
        .env("VLEO_LOG", scratch.join("log"))
        .env("VLEO_RESULTS", results)
        .output()
        .expect("vleo did not start");
    (
        out.status.success(),
        format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        ),
    )
}

/// Every file under `dir`, by its path below it, with its bytes — the index
/// left out, because it is a cache rebuilt from the folders whenever it is
/// stale and never the only copy of anything.
fn tree(dir: &Path) -> Vec<(String, Vec<u8>)> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d).unwrap() {
            let p = e.unwrap().path();
            if p.is_dir() {
                stack.push(p);
            } else if p.file_name().unwrap() != ".index.tsv" {
                let rel = p.strip_prefix(dir).unwrap().display().to_string();
                out.push((rel, std::fs::read(&p).unwrap()));
            }
        }
    }
    out.sort();
    out
}

/// How many rows the design has, as the design file counts them: one node
/// file each in `design/`, which the design file is written from.
fn design_rows() -> usize {
    std::fs::read_dir(root().join("design/groups"))
        .unwrap()
        .flatten()
        .filter_map(|g| std::fs::read_dir(g.path().join("nodes")).ok())
        .map(|d| {
            d.flatten()
                .filter(|n| n.path().extension().is_some_and(|x| x == "vnode"))
                .count()
        })
        .sum()
}

#[test]
fn results_exported_and_imported_are_the_same_results() {
    let scratch = std::env::temp_dir().join(format!("vleo-vleor-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&scratch);
    std::fs::create_dir_all(&scratch).unwrap();
    let (a, b) = (scratch.join("a"), scratch.join("b"));
    let file = scratch.join("study.vleor");

    let (ok, said) = vleo(&scratch, &a, &["data", "sync", "bundles"]);
    assert!(ok, "the reference data did not sync: {said}");
    for node in ["sw_ap_design", "sw_f107_design_short"] {
        let (ok, said) = vleo(&scratch, &a, &["run", node, "--defaults", "--keep"]);
        assert!(ok, "{node} did not run: {said}");
    }

    // A sweep, so the file carries one: its points go out and come back too.
    let (ok, said) = vleo(
        &scratch,
        &a,
        &[
            "sweep",
            "sw_ap_design",
            "--over",
            "sw_storm_design_level",
            "--from",
            "1",
            "--to",
            "3",
            "--points",
            "3",
            "--defaults",
            "--keep",
        ],
    );
    assert!(ok, "the sweep did not run: {said}");

    let (ok, said) = vleo(&scratch, &a, &["results", "export", file.to_str().unwrap()]);
    assert!(ok && said.contains("wrote 3 result(s)"), "{said}");

    // The rows a SQL reader gets are the results' own numbers.
    let kept = vleo_design::results::read(&file).expect("the file does not read");
    assert_eq!(kept.len(), 3);
    assert_eq!(kept.iter().filter(|k| !k.sweep_csv.is_empty()).count(), 1);
    let ap = kept
        .iter()
        .find(|k| k.target == "sw_ap_design" && k.sweep_csv.is_empty())
        .unwrap();
    let answer = ap
        .values
        .iter()
        .find(|v| v.section == "output" && v.id == "sw_ap_design")
        .expect("the answer is not among the value rows");
    assert_eq!(answer.si, Some(132.0), "{answer:?}");
    assert!(ap.values.iter().any(|v| v.section == "input"));

    let (ok, listed) = vleo(&scratch, &b, &["result", file.to_str().unwrap()]);
    assert!(
        ok && listed.contains("3 saved result(s)") && listed.contains(" 132"),
        "{listed}"
    );

    let (ok, said) = vleo(&scratch, &b, &["results", "import", file.to_str().unwrap()]);
    assert!(ok && said.contains("3 result(s) put"), "{said}");
    assert_eq!(
        tree(&a),
        tree(&b),
        "the imported results are not the exported ones"
    );

    let (ok, said) = vleo(&scratch, &b, &["results", "import", file.to_str().unwrap()]);
    assert!(
        ok && said.contains("0 result(s) put") && said.contains("3 were already"),
        "{said}"
    );
    assert_eq!(tree(&a), tree(&b), "a second import changed the folder");

    // A design file is not a results file, and is refused by name.
    let design = scratch.join("design.vleo");
    vleo_design::write(&root(), &design, &vleo_design::Stamp::default()).unwrap();

    // Python reads both files with nothing but the standard library: the same
    // numbers, the same rows, and a design file refused as results.
    let script = r#"
import importlib.util, sys
spec = importlib.util.spec_from_file_location("files", sys.argv[1])
files = importlib.util.module_from_spec(spec); spec.loader.exec_module(files)
rs = files.results(sys.argv[2])
ap = [r for r in rs if r["target"] == "sw_ap_design" and not r["sweep_csv"]][0]
print("results", len(rs), [v["si"] for v in ap["values"] if v["section"] == "output" and v["id"] == "sw_ap_design"])
d = files.design(sys.argv[3])
print("rows", len(d.rows()), d.meta["rows"])
print("sheet", "id" in d.text("crates/vleo-mod-solar/nodes/sw_ap_design/node.toml"))
try:
    files.results(sys.argv[3])
except ValueError as e:
    print("refused", e)
"#;
    let py = Command::new("python3")
        .arg("-c")
        .arg(script)
        .arg(root().join("crates/vleo-py/python/vleo/files.py"))
        .arg(&file)
        .arg(&design)
        .output()
        .expect("python3 is needed to check the Python reader");
    let said = format!(
        "{}{}",
        String::from_utf8_lossy(&py.stdout),
        String::from_utf8_lossy(&py.stderr)
    );
    assert!(py.status.success(), "{said}");
    assert!(said.contains("results 3 [132.0]"), "{said}");
    let rows = design_rows();
    assert!(said.contains(&format!("rows {rows} {rows}")), "{said}");
    assert!(said.contains("sheet True"), "{said}");
    assert!(said.contains("a design file, not a results file"), "{said}");
    let (ok, said) = vleo(
        &scratch,
        &b,
        &["results", "import", design.to_str().unwrap()],
    );
    assert!(
        !ok && said.contains("a design file, not a results file"),
        "{said}"
    );

    let _ = std::fs::remove_dir_all(&scratch);
}
