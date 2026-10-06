//! A group's sealed release, taken into the design.
//!
//! A group seals its release in the group application (`web/group.html`). The
//! developer writes it out as its folder (`node tools/group_db.mjs --unpack
//! <file.vleo>`) and runs this. Nothing about a node is taken on trust:
//!
//! - the seal is checked first — every file's SHA-256, recomputed here, must
//!   give the fingerprint the release was sealed with, so a file changed after
//!   the people signed is refused before anything is read from it;
//! - each computed node becomes the node form its author would have filled —
//!   its pseudocode as the method, its isolation results, brought back to SI,
//!   as its test cases, its author and their declaration of any assistant's
//!   help — and goes through the same plan, check and apply as any form
//!   (`vleo_sheet::template`). So a change the repository made since is a
//!   conflict, named; a method or results an assistant supplied are refused,
//!   and a method is stamped with its author's name, refused under an
//!   assistant's; and an apply is regenerated and gated as one edit or put
//!   back whole.
//!
//! What the release holds beyond the method and its cases — its words, its
//! pictures, its evidence — stays in the release, which is what the group
//! signed; this command reports them, it does not move them into the sheets.

use super::*;
use vleo_files::intake;
use vleo_sheet::template::{self, Derisk, Form};

pub(super) fn cmd_group_intake(root: &Path, args: &[&str]) -> Result<(), String> {
    let usage = "usage: group-intake <unpacked release folder> [--node <id>] [--apply [--partial]] [--draft]";
    let dir = PathBuf::from(*args.iter().find(|a| !a.starts_with("--")).ok_or(usage)?);
    let apply = args.contains(&"--apply");
    let partial = args.contains(&"--partial");
    let draft = args.contains(&"--draft");
    let only = args
        .iter()
        .position(|a| *a == "--node")
        .and_then(|i| args.get(i + 1))
        .copied();

    // 1 · what the release says of itself, and whether its files are the ones sealed
    if apply && draft {
        return Err("an unsealed release is looked at, never applied: --draft and --apply do not go together".into());
    }
    let release = Release::open(&dir, draft)?;

    // 2 · each node, as the form its author would have filled
    let tree = load(root)?;
    let nodes = read_csv(&dir.join("nodes.csv"))?;
    let derisk = release.release.latest_version();
    let (mut planned, mut applied, mut refused, mut takeable) = (0, 0, 0, 0);
    let mut changes: Vec<String> = Vec::new();
    for n in &nodes {
        let id = n.get("id").cloned().unwrap_or_default();
        if only.is_some_and(|o| o != id) {
            continue;
        }
        let Some(sh) = tree.sheets.get(&id) else {
            println!("\n\x1b[1m{id}\x1b[0m — not in the design: a new node arrives by its own form (`xtask form --new`)");
            continue;
        };
        if n.get("kind").map(String::as_str) != Some("computed") {
            continue;
        }
        let form = node_form(root, &release, sh, &derisk)?;
        println!(
            "\n\x1b[1m{id}\x1b[0m — by {}; assistant: {}",
            if form.name.is_empty() {
                "(nobody named)"
            } else {
                &form.name
            },
            form.ai
        );
        let p = template::plan_form(root, form)?;
        forms::print_plan(&p);
        planned += 1;
        if p.applicable() > 0 {
            takeable += 1;
            changes.push(id.clone());
        }
        if p.blocked() > 0 {
            refused += 1;
        }
        if apply && p.applicable() > 0 && (p.blocked() == 0 || partial) {
            match template::apply(root, &p) {
                vleo_sheet::form::Saved::Ok { regenerated, .. } => {
                    applied += 1;
                    println!("  applied — node.toml written, {regenerated} artefact(s) generated, the tree gated");
                }
                vleo_sheet::form::Saved::Stale { .. } => {
                    return Err(format!(
                        "{id}: node.toml changed while this ran — run it again"
                    ))
                }
                vleo_sheet::form::Saved::Refused(e) => {
                    return Err(format!("{id}: refused and put back — {e}"))
                }
            }
        }
    }
    // 3 · whom it reaches: every row of another group downstream of a node
    // this release changes, so the developer knows who to tell before merging.
    if !changes.is_empty() {
        println!(
            "\n\x1b[1mwhom it reaches\x1b[0m — {} node(s) this release changes:",
            changes.len()
        );
        let rows: Vec<&str> = changes.iter().map(String::as_str).collect();
        crate::catalogue::print_impact(&vleo_sheet::catalogue::impact(&tree, &rows));
    }
    println!(
        "\n{planned} computed node(s) read; {refused} with something that cannot be taken{}",
        if apply {
            format!("; {applied} applied")
        } else {
            String::new()
        }
    );
    if !apply && !draft && takeable > 0 {
        println!(
            "apply with: cargo run -p xtask -- group-intake {} --apply   (then `xtask build-node <node>` for each)",
            dir.display()
        );
    }
    Ok(())
}

/// What a written-out release says of itself, once its seal is checked.
pub(super) struct Release {
    pub group: String,
    pub version: String,
    /// When it was sealed; empty for a draft.
    pub sealed: String,
    /// The release as the library reads it (`vleo_files::intake`).
    pub release: intake::Release,
}

impl Release {
    /// Read RELEASE.toml and check every file against the sealed fingerprint.
    /// An unsealed release opens only as a draft, which is looked at and never
    /// taken; a sealed one whose files changed after signing does not open.
    pub(super) fn open(dir: &Path, draft: bool) -> Result<Release, String> {
        let rel: toml::Value = fs::read_to_string(dir.join("RELEASE.toml"))
            .map_err(|_| {
                format!(
                    "{} has no RELEASE.toml: it is not an unpacked release. Write the release out first: \
                     node tools/group_db.mjs --unpack <file.vleo> --out {}",
                    dir.display(),
                    dir.display()
                )
            })?
            .parse()
            .map_err(|e| format!("RELEASE.toml: {e}"))?;
        let meta: BTreeMap<String, String> = rel
            .as_table()
            .map(|t| {
                t.iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                    .collect()
            })
            .unwrap_or_default();
        let release = intake::Release::new(&meta, folder(dir)?);
        let (group, version, sealed) = (
            release.group.clone(),
            release.version.clone(),
            release.sealed.clone(),
        );
        println!(
            "\x1b[1m{group} {version}\x1b[0m — {}",
            if sealed.is_empty() {
                "NOT sealed".to_string()
            } else {
                format!(
                    "sealed {} by {}",
                    &sealed[..sealed.len().min(10)],
                    release.sealed_by
                )
            }
        );
        if sealed.is_empty() {
            if !draft {
                return Err(
                    "this release is not sealed. Look at it with --draft; it is taken only once sealed"
                        .into(),
                );
            }
        } else {
            let have = release.check_seal()?;
            println!(
                "  every file is the one sealed — fingerprint {}…",
                &have[..16]
            );
        }
        Ok(Release {
            group,
            version,
            sealed,
            release,
        })
    }
}

/// One node of the release as a node form (`vleo_files::intake::node_form`),
/// against its sheet as the folders hold it now.
fn node_form(
    root: &Path,
    release: &Release,
    sh: &vleo_sheet::model::Sheet,
    derisk: &Derisk,
) -> Result<Form, String> {
    let base = fs::read_to_string(sh.dir.join("node.toml"))
        .map(|t| vleo_sheet::form::file_hash(&t))
        .unwrap_or_default();
    let (form, not_taken) = intake::node_form(root, &release.release, sh, base, derisk)?;
    if let Some(why) = not_taken {
        println!("  transcribed, but {why}: taken as a relation an assistant supplied");
    }
    Ok(form)
}

#[cfg(test)]
use intake::declared_help;

/// The isolation results as the form's test cases: every value back in SI,
/// as the sheet holds it (`vleo_files::intake::cases`).
pub(super) fn cases(path: &Path) -> Result<Vec<BTreeMap<String, String>>, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    intake::cases(&text, &path.display().to_string())
}

/// The unpacked release as the folder that was sealed, for the library's
/// seal (`vleo_files::seal`). RELEASE.toml is the unpacking's record of the
/// seal, written beside the folder's files and never one of them.
fn folder(dir: &Path) -> Result<vleo_files::format_1::Folder, String> {
    fn walk(base: &Path, d: &Path, out: &mut vleo_files::format_1::Folder) -> Result<(), String> {
        for e in fs::read_dir(d).map_err(|e| format!("{}: {e}", d.display()))? {
            let p = e.map_err(|e| e.to_string())?.path();
            if p.is_dir() {
                walk(base, &p, out)?;
            } else {
                let rel = p.strip_prefix(base).map_err(|e| e.to_string())?;
                let bytes = fs::read(&p).map_err(|e| format!("{}: {e}", p.display()))?;
                out.insert(rel.to_string_lossy().replace('\\', "/"), bytes);
            }
        }
        Ok(())
    }
    let mut folder = vleo_files::format_1::Folder::new();
    walk(dir, dir, &mut folder)?;
    folder.remove("RELEASE.toml");
    Ok(folder)
}

/// A CSV file as records by header.
pub(super) fn read_csv(path: &Path) -> Result<Vec<BTreeMap<String, String>>, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(intake::records(&text))
}

pub(super) use intake::parse_csv;

#[cfg(test)]
mod tests {
    use super::*;

    fn declaration(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    fn repo() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .to_path_buf()
    }

    #[test]
    fn a_transcription_is_taken_only_with_its_source_and_a_person_who_checked_it() {
        let root = repo();
        for a in ["none", "wording", "relation"] {
            assert_eq!(declared_help(&root, &declaration(&[("ai", a)])).ai, a);
        }
        // Silence, and a value nobody defined, are an assistant's relation.
        assert_eq!(declared_help(&root, &declaration(&[])).ai, "relation");
        assert_eq!(
            declared_help(&root, &declaration(&[("ai", "copied")])).ai,
            "relation"
        );
        let taken = declared_help(
            &root,
            &declaration(&[
                ("ai", "transcribed"),
                ("source", "crates/vleo-core/src/physics/solar.rs:120"),
                ("checked_by", "A. Person"),
            ]),
        );
        assert_eq!(taken.ai, "transcribed");
        let record = taken.transcribed.unwrap();
        assert!(
            record.contains("solar.rs:120") && record.contains("A. Person"),
            "{record}"
        );
        for (source, checker) in [
            ("", "A. Person"),
            ("x.rs:1", ""),
            ("x.rs:1", "  "),
            ("x.rs:1", "Claude"),
            ("x.rs:1", "claude/opus"),
            ("x.rs:1", "Copilot bot"),
        ] {
            let h = declared_help(
                &root,
                &declaration(&[
                    ("ai", "transcribed"),
                    ("source", source),
                    ("checked_by", checker),
                ]),
            );
            assert_eq!(h.ai, "relation", "{source:?} / {checker:?}");
            assert!(h.not_taken.is_some() && h.transcribed.is_none());
        }
    }

    #[test]
    fn the_spec_refuses_the_same_assistants_the_engine_does() {
        let root = repo();
        let spec: toml::Value = fs::read_to_string(root.join("groups/SPEC.toml"))
            .unwrap()
            .parse()
            .unwrap();
        let listed: Vec<String> = spec["file"]
            .as_array()
            .unwrap()
            .iter()
            .find(|f| f["path"].as_str() == Some("declaration.csv"))
            .and_then(|f| f.get("assistants"))
            .and_then(|a| a.as_array())
            .expect("declaration.csv lists the assistants' names")
            .iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect();
        assert_eq!(listed, vleo_sheet::form::agent_identities(&root));
    }

    #[test]
    fn a_transcribed_method_is_planned_where_an_assistants_relation_is_refused() {
        let root = repo();
        let tree = load(&root).unwrap();
        let sh = tree.sheets.get("sw_activity_band").unwrap();
        let original = template::content(sh);
        let mut filled = original.clone();
        let method = original
            .fields
            .get("method_text")
            .cloned()
            .unwrap_or_default();
        filled.fields.insert(
            "method_text".into(),
            format!("# copied from the running code\n{method}"),
        );
        let base = fs::read_to_string(sh.dir.join("node.toml"))
            .map(|t| vleo_sheet::form::file_hash(&t))
            .unwrap();
        let plan = |ai: &str| {
            let f = Form {
                node: sh.id.clone(),
                base: base.clone(),
                name: "A. Person".into(),
                date: "2026-10-03".into(),
                ai: ai.into(),
                original: original.clone(),
                filled: filled.clone(),
                ..Form::default()
            };
            let p = template::plan_form(&root, f).unwrap();
            p.items
                .into_iter()
                .find(|i| i.what == "method_text")
                .expect("the method is an item of the plan")
                .verdict
        };
        let refused_as_assistants = |v: &vleo_sheet::template::Verdict| matches!(v, vleo_sheet::template::Verdict::Refused(why) if why.contains("assistant"));
        assert!(refused_as_assistants(&plan("relation")));
        assert!(!refused_as_assistants(&plan("transcribed")));
    }

    #[test]
    fn csv_reads_quotes_commas_and_line_breaks() {
        let (h, r) = parse_csv("a,b\n\"x, \"\"y\"\"\",\"two\nlines\"\n1,2\n");
        assert_eq!(h, ["a", "b"]);
        assert_eq!(r[0], ["x, \"y\"", "two\nlines"]);
        assert_eq!(r[1], ["1", "2"]);
    }

    #[test]
    fn the_seal_fingerprint_is_the_one_the_page_makes() {
        // Two files and the folder's record of itself; the expected value was
        // worked out with Python's hashlib, not with this code.
        let d = std::env::temp_dir().join(format!("vleo-gi-fp-{}", std::process::id()));
        fs::create_dir_all(d.join("nodes/a")).unwrap();
        fs::write(d.join("group.csv"), "id,name\nx,X\n").unwrap();
        fs::write(d.join("nodes/a/pseudocode.txt"), "return 1\n").unwrap();
        fs::write(d.join("reviews.csv"), "name,scope\nsomebody,group\n").unwrap();
        fs::write(d.join("RELEASE.toml"), "sealed = \"x\"\n").unwrap();
        assert_eq!(
            vleo_files::seal::fingerprint_of(&folder(&d).unwrap(), "group"),
            "e7eda2b52f58711281855d3e2815334551728a2db3b862a724b7ba6f4dc93575"
        );
        // One byte changed after the seal, and it is another fingerprint.
        fs::write(d.join("nodes/a/pseudocode.txt"), "return 2\n").unwrap();
        assert_ne!(
            vleo_files::seal::fingerprint_of(&folder(&d).unwrap(), "group"),
            "e7eda2b52f58711281855d3e2815334551728a2db3b862a724b7ba6f4dc93575"
        );
        fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn isolation_results_become_cases_in_si() {
        let d = std::env::temp_dir().join(format!("vleo-gi-{}", std::process::id()));
        fs::create_dir_all(&d).unwrap();
        let f = d.join("isolation.csv");
        fs::write(
            &f,
            "lead [d],answer [d],tolerance,refuses,origin,says\n2,1,1e-9,no,hand,two days\nNaN,,,yes,hand,\n",
        )
        .unwrap();
        let c = cases(&f).unwrap();
        assert_eq!(c[0]["inputs"], "{ lead = 172800.0 }");
        assert_eq!(c[0]["expect"], "86400.0");
        assert_eq!(c[0]["label"], "two days");
        assert_eq!(c[1]["refuse"], "yes");
        assert_eq!(c[1]["inputs"], "{ lead = nan }");
        assert!(!c[1].contains_key("expect"));
        // Where each answer came from goes with it.
        assert_eq!(c[0]["origin"], "hand");
        fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn each_published_member_is_answered_in_its_own_column() {
        let d = std::env::temp_dir().join(format!("vleo-gi-also-{}", std::process::id()));
        fs::create_dir_all(&d).unwrap();
        let f = d.join("isolation.csv");
        fs::write(
            &f,
            "lead [d],answer [d],answer.Half [d],tolerance,refuses,origin\n2,1,0.5,1e-9,no,code\n-1,,,,yes,hand\n",
        )
        .unwrap();
        let c = cases(&f).unwrap();
        // A member's column is an answer, never an input.
        assert_eq!(c[0]["inputs"], "{ lead = 172800.0 }");
        assert_eq!(c[0]["also"], "{ Half = 43200.0 }");
        assert!(!c[1].contains_key("also"));
        fs::remove_dir_all(&d).ok();
    }
}
