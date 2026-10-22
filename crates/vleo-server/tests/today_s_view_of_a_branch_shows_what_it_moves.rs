//! Today's view of a branch, and its impact (docs/briefs/F5.md, F5-d).
//!
//! A drive of invented people, as the W5 test has it: Test Pat (91),
//! programme manager; Test Sam (92), system engineer; Test Ola (93), who owns
//! solar; Test Nia (94), assigned two of solar's nodes. No group has sealed a
//! release, so today's design is the design's own. In Nia's copy,
//! `sw_f107_design_long` is brought to 1.0's checks from what solar's 1.1
//! release holds of it, and its method given a line its own cases do not
//! look at, as the broken-release test gives it: below a centre of 95 sfu it
//! answers from another table. She signs it. `sw_activity_band` she has not
//! signed. Today's view of solar takes the signed node and leaves the other
//! out, saying why; and every value outside solar that moves is listed with
//! its group.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use vleo_bus::Case;
use vleo_files::chain::{self, Registry, FILE};
use vleo_files::keys::SigningKey;
use vleo_files::meta::Kind;
use vleo_files::model::*;
use vleo_files::sqlite;
use vleo_server::today::{branch_view, build, impact};

const APP: &str = "vleo 1.0.0 (test)";
const AT: &str = "2026-10-15T12:00:00Z";
const LONG: &str = "sw_f107_design_long";
const BAND: &str = "sw_activity_band";

fn s(v: &str) -> String {
    v.to_string()
}

fn key(seed: u8) -> SigningKey {
    SigningKey::from_seed([seed; 32])
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn register(f: &mut File, name: &str, role: &str, k: &SigningKey, by: &str) {
    f.people.push(Person {
        name: s(name),
        role: s(role),
        deputy_for: s(""),
    });
    f.keys.push(PersonKey {
        person: s(name),
        public_key: k.public().to_hex(),
        registered_at: s("2026-01-01"),
        registered_by: s(by),
        revoked_from: s(""),
        proof: vleo_files::people::proof(k, name),
    });
}

fn mount(f: &mut File, uid: &str, group: &str, owner: &str) {
    f.blocks.push(Block {
        uid: s(uid),
        id: s(uid),
        behaviour: s("children"),
        ..Block::default()
    });
    f.mounts.push(Mount {
        block_uid: s(uid),
        group_id: s(group),
        release: s(""),
    });
    f.assignments.push(Assignment {
        block_uid: s(uid),
        person: s(owner),
        contract_version: 1,
    });
}

fn group(id: &str, writer: &str) -> File {
    let mut f = File::new(Kind::Group, APP);
    f.meta.insert(s("group_id"), s(id));
    f.meta.insert(s("writer"), s(writer));
    f.meta.insert(s("based_on"), s(""));
    f
}

fn signed(mut f: File, who: &str, k: &SigningKey) -> File {
    let row = chain::sign(&f, FILE, who, k, "2026-02-01", "ok", "").unwrap();
    f.signatures.push(row);
    f
}

fn design(path: &str) -> File {
    sqlite::read(&root().join("design").join(path)).unwrap()
}

fn node_path(uid: &str) -> String {
    format!("groups/l3_solar/nodes/{uid}.vnode")
}

/// Solar's group file, opened through the chain as the drive holds it.
fn solar_registry() -> Registry {
    let mut programme = group("programme", "Test Pat");
    register(
        &mut programme,
        "Test Pat",
        "programme manager",
        &key(91),
        "Test Pat",
    );
    register(
        &mut programme,
        "Test Sam",
        "system engineer",
        &key(92),
        "Test Pat",
    );
    register(
        &mut programme,
        "Test Ola",
        "subsystem engineer",
        &key(93),
        "Test Pat",
    );
    mount(&mut programme, "root", "systems", "Test Sam");
    let mut systems = group("systems", "Test Sam");
    mount(
        &mut systems,
        "sys_space_environment",
        "l3_solar",
        "Test Ola",
    );
    let mut solar = design("groups/l3_solar/l3_solar.vgroup");
    solar.meta.insert(s("writer"), s("Test Ola"));
    register(
        &mut solar,
        "Test Nia",
        "node engineer",
        &key(94),
        "Test Ola",
    );
    for uid in [LONG, BAND] {
        solar.assignments.push(Assignment {
            block_uid: s(uid),
            person: s("Test Nia"),
            contract_version: 1,
        });
    }
    let anchor = key(91).public().fingerprint();
    let pr = chain::open_programme(signed(programme, "Test Pat", &key(91)), &anchor).unwrap();
    let sy = chain::open_systems(&signed(systems, "Test Sam", &key(92)), &pr).unwrap();
    chain::open_group_in(&signed(solar, "Test Ola", &key(93)), &sy, &pr).unwrap()
}

/// The design's node `uid`, brought to 1.0's checks by its node engineer
/// from what solar's 1.1 release holds of it: its method in the method
/// language, its declaration and cases, its inputs' defaults, and its units
/// as the method language writes them.
fn brought_up(uid: &str) -> File {
    let mut n = design(&node_path(uid));
    n.meta.insert(s("writer"), s("Test Nia"));
    let how = vleo_files::upgrade::Upgrade {
        app: APP,
        at: "2026-10-01",
    };
    let rel = sqlite::open(
        &root().join("crates/vleo-files/tests/fixtures/l3_solar-1.1.vleo"),
        &how,
    )
    .unwrap()
    .file;
    n.texts.extend(
        rel.texts
            .iter()
            .filter(|t| t.scope == uid && t.kind == "pseudocode")
            .cloned(),
    );
    n.tables
        .extend(rel.tables.iter().filter(|t| t.scope == uid).cloned());
    for p in n.ports.iter_mut() {
        let theirs = rel
            .ports
            .iter()
            .find(|q| {
                q.block_uid == uid
                    && q.direction == p.direction
                    && (q.name == p.name || p.direction == "out")
            })
            .unwrap();
        p.unit = theirs.unit.clone();
        if p.direction == "in" {
            p.value = theirs.value.clone();
            p.lower = theirs.lower.clone();
            p.upper = theirs.upper.clone();
        }
    }
    n
}

/// The long design F10.7, its method answering from another table below a
/// centre of 95 sfu, where none of its own cases is; signed by Nia.
fn long_changed(registry: &Registry) -> File {
    let mut n = brought_up(LONG);
    // The method is the design's `method` text, which the loader runs, and
    // the `pseudocode` the library's checks read: one method, said twice.
    for t in n
        .texts
        .iter_mut()
        .filter(|t| t.kind == "pseudocode" || t.kind == "method")
    {
        assert!(t.body.contains("return sustained"));
        t.body = t.body.replace(
            "return sustained",
            "if central < 95 then\n  return central + spread * 14\nend\nreturn sustained",
        );
    }
    chain::sign_node(&n, LONG, "Test Nia", &key(94), AT, registry)
        .expect("Nia signs it")
        .0
}

/// The case with the reference data, verified into a store of the test's
/// own.
fn with_data(scratch: &Path) -> Case {
    std::env::set_var("VLEO_DATA", scratch.join("data"));
    let mut store = vleo_data::Store::open(&scratch.join("data"));
    store
        .sync(&vleo_data::Source::Shipped(root().join("bundles")))
        .expect("the shipped bundles verify");
    Case {
        data: store.verified_names(),
        ..Default::default()
    }
}

/// Every node of solar the design holds, by id: what is inside the branch.
fn solar_ids() -> BTreeSet<String> {
    let mut out: BTreeSet<String> = std::fs::read_dir(root().join("design/groups/l3_solar/nodes"))
        .unwrap()
        .map(|e| {
            e.unwrap()
                .path()
                .file_stem()
                .unwrap()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    out.insert(s("l3_solar"));
    out
}

#[test]
fn today_s_view_of_a_branch_takes_its_signed_nodes_and_lists_what_they_move() {
    let scratch = std::env::temp_dir().join(format!("vleo-view-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&scratch);
    let drive = scratch.join("drive");
    std::fs::create_dir_all(drive.join("groups")).unwrap();
    let (served, _) = vleo_files::convert::serve(&root()).unwrap();
    let today = build(Arc::new(served), &root(), &drive).expect("today's design builds");
    assert!(today.groups.is_empty());

    let registry = solar_registry();
    let mut band = design(&node_path(BAND));
    band.meta.insert(s("writer"), s("Test Nia"));
    let nodes = vec![long_changed(&registry), band];
    let view = branch_view(today.files.clone(), &root(), "l3_solar", &registry, &nodes)
        .expect("the view builds");
    assert_eq!(view.changes, [LONG]);
    let (left, why) = &view.left_out[0];
    assert_eq!(left, BAND);
    assert!(why.starts_with("not signed as it is"), "{why}");

    let case = with_data(&scratch);
    let t_tree = vleo_sheet::load::load_all_from(&*today.files, &root()).unwrap();
    let t_graph = vleo_modules::opened::graph(&t_tree).unwrap();
    let v_tree = vleo_sheet::load::load_all_from(&*view.files, &root()).unwrap();
    let v_graph = vleo_modules::opened::graph(&v_tree).unwrap();
    let inside = solar_ids();
    let im = impact(t_graph, v_graph, &case, &|id| inside.contains(id)).unwrap();
    // How each design ran is said beside what moved: "n ran, m blocked",
    // each blocked node with why.
    assert!(
        im.today.ran > 0 && im.view.ran > 0,
        "{:?} {:?}",
        im.today,
        im.view
    );
    assert!(im.view.blocked.iter().all(|(_, why)| !why.is_empty()));
    // On a case with no reference data, what needs it blocks: each blocked
    // node is named, with why, never dropped.
    let bare = impact(t_graph, v_graph, &Case::default(), &|id| {
        inside.contains(id)
    })
    .unwrap();
    assert!(!bare.today.blocked.is_empty(), "{:?}", bare.today);
    assert!(
        bare.view
            .blocked
            .iter()
            .all(|(node, why)| !node.is_empty() && !why.is_empty()),
        "{:?}",
        bare.view
    );
    let moved = im.moved;
    // Nothing of solar's own is its impact: its interface's outputs neither.
    assert!(
        moved
            .iter()
            .all(|m| !inside.contains(m.id.split('.').next().unwrap())),
        "{moved:#?}"
    );
    // What it moves outside: the environment's F10.7 in the orbit group, and
    // the systems' space environment, each with its group, before and after.
    for (id, group) in [
        ("env_f107", "l3_x_envorbit"),
        ("env_exospheric_temperature", "l3_x_envorbit"),
        ("sys_space_environment_f10_7", "sys_space_environment"),
    ] {
        let m = moved
            .iter()
            .find(|m| m.id == id)
            .unwrap_or_else(|| panic!("{id} moves: {moved:#?}"));
        assert_eq!(m.group, group, "{m:?}");
        let (b, a) = (m.before.unwrap(), m.after.unwrap());
        assert!(b != a, "{m:?}");
    }
    // The same designs, judged with nothing inside the branch: solar's own
    // node moves too, and nothing in the view that is not signed does.
    let everything = impact(t_graph, v_graph, &case, &|_| false).unwrap().moved;
    assert!(everything.iter().any(|m| m.id == LONG));
    assert!(!everything.iter().any(|m| m.id == BAND));
    let _ = std::fs::remove_dir_all(&scratch);
}

#[test]
fn a_node_whose_method_its_node_engineer_dropped_is_left_out_never_run_with_the_design_s() {
    let scratch = std::env::temp_dir().join(format!("vleo-view-drop-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&scratch);
    let drive = scratch.join("drive");
    std::fs::create_dir_all(drive.join("groups")).unwrap();
    let (served, _) = vleo_files::convert::serve(&root()).unwrap();
    let today = build(Arc::new(served), &root(), &drive).expect("today's design builds");
    let registry = solar_registry();
    // Its method text gone from the file Nia signs, its pseudocode, which
    // the checks read, kept: the design's method is not what runs in its
    // place.
    let mut n = brought_up(LONG);
    n.texts.retain(|t| t.kind != "method");
    let n = chain::sign_node(&n, LONG, "Test Nia", &key(94), AT, &registry)
        .expect("Nia signs it")
        .0;
    let view = branch_view(today.files.clone(), &root(), "l3_solar", &registry, &[n])
        .expect("the view builds");
    assert!(view.changes.is_empty(), "{:?}", view.changes);
    let (left, why) = &view.left_out[0];
    assert_eq!(left, LONG);
    assert!(
        why.contains("it is not read as the design reads a node"),
        "{why}"
    );
    let _ = std::fs::remove_dir_all(&scratch);
}
