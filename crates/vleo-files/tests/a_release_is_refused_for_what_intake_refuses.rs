//! The library refuses a release for what today's intake refuses it for.
//!
//! docs/PLAN_1_0.md, phase C: "the library refuses everything today's intake
//! refuses, in a test for each". Two real releases are the oracle, both as the
//! solar group sealed them:
//!
//! - **Solar 1.0**, which `xtask group-intake` refused whole: six computed
//!   nodes had no case their method must refuse. The group's own record says
//!   which six, in the `learned` of its 1.1 row of versions.csv.
//! - **Solar 1.1**, which intake took and the group accepted
//!   (acceptances/l3_solar-1.1.toml): nothing in it is refused.
//!
//! Then each refusal is made on its own, one change to Solar 1.1 at a time,
//! and each is refused by name, at its node.

use std::path::Path;

use vleo_files::checks::{self, Level};
use vleo_files::model::File;
use vleo_files::sqlite;
use vleo_files::upgrade::Upgrade;

const HOW: Upgrade = Upgrade {
    app: "vleo 1.0.0 (test)",
    at: "2026-10-06T09:00:00Z",
};

fn solar(version: &str) -> File {
    let p = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("tests/fixtures/l3_solar-{version}.vleo"));
    sqlite::open(&p, &HOW).unwrap().file
}

fn errors(f: &File) -> Vec<(String, String)> {
    checks::check_release(f)
        .errors()
        .map(|e| (e.place.clone(), e.what.clone()))
        .collect()
}

#[test]
fn solar_1_1_which_intake_took_holds() {
    let r = checks::check_release(&solar("1.1"));
    let errs: Vec<_> = r.errors().collect();
    assert!(errs.is_empty(), "{errs:#?}");
    assert!(r.holds());
    // Shown, never refusing: edges of a range no case reaches.
    assert!(r.0.iter().all(|f| f.level == Level::Warning));
}

#[test]
fn solar_1_0_which_intake_refused_is_refused_for_the_same_six_nodes() {
    // The six the group's record names: versions.csv, 1.1, `learned` — "six
    // nodes had no case the method must refuse" — and 0.2's list of them.
    let named = [
        "sw_activity_band",
        "sw_ap_daily_band_drop",
        "sw_ap_design",
        "sw_exceedance_rate",
        "sw_horizon_persistence",
        "sw_regime",
    ];
    let f = solar("1.0");
    let errs = errors(&f);
    let places: std::collections::BTreeSet<&str> = errs.iter().map(|(p, _)| p.as_str()).collect();
    assert_eq!(places, named.into_iter().collect(), "{errs:#?}");
    assert!(
        errs.iter()
            .all(|(_, w)| w.contains("no case the node must refuse")),
        "{errs:#?}"
    );
    assert!(!checks::check_release(&f).holds());
}

/// A change to one of the release's CSV tables.
fn edit(f: &mut File, scope: &str, path: &str, how: impl Fn(&str) -> String) {
    let t = f
        .tables
        .iter_mut()
        .find(|t| t.scope == scope && t.path == path)
        .unwrap_or_else(|| panic!("no {path} in {scope}"));
    t.csv = how(&t.csv);
}

fn drop_table(f: &mut File, scope: &str, path: &str) {
    f.tables.retain(|t| !(t.scope == scope && t.path == path));
}

const DECL: &str = "author,ai,date,source,checked_by\n";

fn declare(f: &mut File, row: &str) {
    edit(f, "sw_regime", "declaration.csv", |_| {
        format!("{DECL}{row}\n")
    });
}

fn results_line(f: &mut File, from: &str, to: &str) {
    edit(f, "sw_regime", "results/isolation.csv", |t| {
        assert!(t.contains(from), "the results do not hold {from:?}");
        t.replacen(from, to, 1)
    });
}

type Change = fn(&mut File);

/// Every refusal, made alone: what changes, where it is refused, and the
/// words that say why.
const REFUSALS: &[(&str, Change, &str, &str)] = &[
    // ── who made it, and whether an assistant helped (AGENTS.md, rule 6) ──
    (
        "no declaration",
        |f| drop_table(f, "sw_regime", "declaration.csv"),
        "sw_regime",
        "has no declaration",
    ),
    (
        "an assistant supplied the relation",
        |f| declare(f, "Aman Rai,relation,2026-10-03,,"),
        "sw_regime",
        "an assistant supplied the relation",
    ),
    (
        "the declaration is silent",
        |f| declare(f, "Aman Rai,,2026-10-03,,"),
        "sw_regime",
        "silence is not none",
    ),
    (
        "a help that is no kind there is",
        |f| declare(f, "Aman Rai,some,2026-10-03,,"),
        "sw_regime",
        "ai = \"some\"",
    ),
    (
        "a transcription from nowhere",
        |f| declare(f, "Aman Rai,transcribed,2026-10-03,,Aman Rai"),
        "sw_regime",
        "names no source",
    ),
    (
        "a transcription nobody checked",
        |f| declare(f, "Aman Rai,transcribed,2026-10-03,model.rs,"),
        "sw_regime",
        "names nobody who checked",
    ),
    (
        "a transcription an assistant checked",
        |f| declare(f, "Aman Rai,transcribed,2026-10-03,model.rs,Claude"),
        "sw_regime",
        "\"Claude\" checked the transcription",
    ),
    (
        "an assistant as the author",
        |f| declare(f, "ChatGPT,none,2026-10-03,,"),
        "sw_regime",
        "an assistant's name, as the author",
    ),
    (
        "no author",
        |f| declare(f, ",none,2026-10-03,,"),
        "sw_regime",
        "names no author",
    ),
    (
        "two declarations",
        |f| {
            edit(f, "sw_regime", "declaration.csv", |_| {
                format!("{DECL}A,none,d,,\nB,none,d,,\n")
            })
        },
        "sw_regime",
        "must be one row; it has 2",
    ),
    (
        "a signature under an assistant's name",
        |f| f.signatures[1].signer = "Copilot".into(),
        "l3_solar_ach_01",
        "is signed by \"Copilot\"",
    ),
    // ── the method ──
    (
        "no method",
        |f| {
            f.texts
                .retain(|t| !(t.scope == "sw_regime" && t.kind == "pseudocode"))
        },
        "sw_regime",
        "has no method",
    ),
    // ── the method, read and run by the method language ──
    (
        "a method that does not parse",
        |f| set_method(f, "sw_regime", "let out =\nreturn out"),
        "sw_regime",
        "its method, line",
    ),
    (
        "a method whose units disagree",
        |f| {
            port(f, "sw_regime", "ap").unit = "m".into();
            results_line(f, "ap [1],", "ap [m],");
        },
        "sw_regime",
        "its method, line",
    ),
    (
        "an answer the method does not reproduce",
        |f| results_line(f, "\n20,2,", "\n20,3,"),
        "sw_regime",
        "the method gives 2, your code gave 3",
    ),
    (
        "a refusal the method answers",
        |f| results_line(f, "\nNaN,,,yes,", "\n50,,,yes,"),
        "sw_regime",
        "your code refuses this case, but the method answers 3",
    ),
    (
        "a published member the method does not reproduce",
        |f| {
            edit(f, "sw_kp_scenarios", "results/isolation.csv", |t| {
                t.replacen(",7.9850088183,2.6", ",7.9850088183,9.6", 1)
            })
        },
        "sw_kp_scenarios",
        "the method publishes Kp_mean_nominal",
    ),
    (
        "an input in a unit the method language does not read",
        |f| {
            port(f, "sw_regime", "ap").unit = "furlongs".into();
            results_line(f, "ap [1],", "ap [furlongs],");
        },
        "sw_regime",
        "which the method language does not read",
    ),
    // ── the results: the reference the code must match (rule 4) ──
    (
        "no results",
        |f| drop_table(f, "sw_regime", "results/isolation.csv"),
        "sw_regime",
        "has no results/isolation.csv",
    ),
    (
        "no column for an input",
        |f| results_line(f, "ap [1],", "Ap [1],"),
        "sw_regime",
        "no column for the input ap",
    ),
    (
        "an input in another unit",
        |f| results_line(f, "ap [1],", "ap [nT],"),
        "sw_regime",
        "give ap in [nT] where the input is in [1]",
    ),
    (
        "no answer",
        |f| results_line(f, "answer [1],", "result [1],"),
        "sw_regime",
        "have no answer column",
    ),
    (
        "the answer in another unit",
        |f| results_line(f, "answer [1],", "answer [m],"),
        "sw_regime",
        "answer in [m] where the node answers in [1]",
    ),
    (
        "no tolerance column",
        |f| results_line(f, ",tolerance,", ",slack,"),
        "sw_regime",
        "have no tolerance column",
    ),
    (
        "a refusal neither yes nor no",
        |f| results_line(f, ",no,paper,", ",maybe,paper,"),
        "sw_regime",
        "refuses is \"maybe\"",
    ),
    (
        "an expected value from the code under test",
        |f| results_line(f, ",no,paper,", ",no,self-snapshot,"),
        "sw_regime",
        "never from the code under test",
    ),
    (
        "an input that is not a number",
        |f| results_line(f, "\n20,2,", "\ntwenty,2,"),
        "sw_regime",
        "ap is \"twenty\", not a number",
    ),
    (
        "an answer that is not a number",
        |f| results_line(f, "\n20,2,", "\n20,two,"),
        "sw_regime",
        "answer is not a number",
    ),
    (
        "a tolerance of nothing",
        |f| results_line(f, "\n20,2,0.000000000001,", "\n20,2,0,"),
        "sw_regime",
        "tolerance must be a number above zero",
    ),
    (
        "no row at the defaults",
        |f| results_line(f, "\n158.38,3,", "\n158.39,3,"),
        "sw_regime",
        "no row with every input at its default",
    ),
    (
        "fewer than three ordinary cases",
        |f| {
            edit(f, "sw_regime", "results/isolation.csv", |t| {
                let mut lines: Vec<&str> = t.lines().collect();
                // The header, the defaults, one more, and the refusals.
                lines.retain(|l| {
                    l.starts_with("ap ")
                        || l.starts_with("158.38,")
                        || l.starts_with("20,")
                        || l.contains(",yes,")
                });
                lines.join("\n") + "\n"
            })
        },
        "sw_regime",
        "2 ordinary case(s); a node needs at least 3",
    ),
    (
        "no refusal",
        |f| {
            edit(f, "sw_regime", "results/isolation.csv", |t| {
                t.lines()
                    .filter(|l| !l.contains(",yes,"))
                    .collect::<Vec<_>>()
                    .join("\n")
                    + "\n"
            })
        },
        "sw_regime",
        "no case the node must refuse",
    ),
    // ── inputs ──
    (
        "an input from nowhere",
        |f| f.wires.retain(|w| w.to_block != "sw_regime"),
        "sw_regime",
        "the input ap does not say where it comes from",
    ),
    (
        "an input from a node that is not there",
        |f| {
            let w = f
                .wires
                .iter_mut()
                .find(|w| w.to_block == "sw_regime")
                .unwrap();
            w.from_ref = "sw_nothing".into();
        },
        "sw_regime",
        "comes from \"sw_nothing\", which connects to nothing",
    ),
    (
        "an input from a port its node does not have",
        |f| {
            let w = f
                .wires
                .iter_mut()
                .find(|w| w.to_block == "sw_regime")
                .unwrap();
            w.from_ref = "sw_storm_return_level.nothing".into();
        },
        "sw_regime",
        "which connects to nothing",
    ),
    (
        "an input with no default",
        |f| port(f, "sw_regime", "ap").value.clear(),
        "sw_regime",
        "the input ap has no default value",
    ),
    (
        "a default below its min",
        |f| port(f, "sw_regime", "ap").value = "10".into(),
        "sw_regime",
        "the default is below its min",
    ),
    (
        "a default above its max",
        |f| port(f, "sw_regime", "ap").value = "300".into(),
        "sw_regime",
        "the default is above its max",
    ),
    // ── a declared value ──
    (
        "a declared node with no value",
        |f| out_port(f, "sw_band_confidence").value.clear(),
        "sw_band_confidence",
        "is declared, so it needs a value",
    ),
    (
        "a range upside down",
        |f| out_port(f, "sw_band_confidence").lower = "2".into(),
        "sw_band_confidence",
        "lower bound is above its upper",
    ),
    // ── requirements, which way each binds ──
    (
        "a requirement with no sense",
        |f| {
            edit(f, "file", "requirements.csv", |t| {
                t.replacen(",<=,", ",,", 1)
            })
        },
        "file",
        "a sense is never defaulted",
    ),
    (
        "a requirement with a sense there is not",
        |f| {
            edit(f, "file", "requirements.csv", |t| {
                t.replacen(",<=,", ",=,", 1)
            })
        },
        "file",
        "has the sense \"=\"",
    ),
    (
        "a requirement on a node that is not there",
        |f| {
            edit(f, "file", "requirements.csv", |t| {
                t.replacen(
                    "l3_solar_interface,l3_solar_interface,",
                    "l3_solar_interface,nothing_here,",
                    1,
                )
            })
        },
        "file",
        "the required node \"nothing_here\" does not exist",
    ),
    // ── which belief broke (docs/DERISKING.md) ──
    (
        "no record of versions",
        |f| drop_table(f, "file", "versions.csv"),
        "file",
        "has no versions.csv",
    ),
    (
        "no row for this version",
        |f| {
            f.meta.insert("version".into(), "1.2".into());
        },
        "file",
        "no row for version 1.2",
    ),
    (
        "a change that does not say what broke",
        |f| {
            edit(f, "file", "versions.csv", |t| {
                let (head, rest) = t.split_once('\n').unwrap();
                let rows: Vec<Vec<String>> = vleo_files::csv::parse(t)
                    .1
                    .into_iter()
                    .map(|mut r| {
                        if r[0] == "1.1" {
                            r[5] = String::new();
                        }
                        r
                    })
                    .collect();
                let _ = rest;
                vleo_files::csv::write(&head.split(',').collect::<Vec<_>>(), &rows)
            })
        },
        "file",
        "its record is missing: learned",
    ),
];

fn set_method(f: &mut File, block: &str, body: &str) {
    let t = f
        .texts
        .iter_mut()
        .find(|t| t.scope == block && t.kind == "pseudocode")
        .unwrap();
    t.body = body.to_string();
}

fn port<'a>(f: &'a mut File, block: &str, name: &str) -> &'a mut vleo_files::model::Port {
    f.ports
        .iter_mut()
        .find(|p| p.block_uid == block && p.direction == "in" && p.name == name)
        .unwrap()
}

fn out_port<'a>(f: &'a mut File, block: &str) -> &'a mut vleo_files::model::Port {
    f.ports
        .iter_mut()
        .find(|p| p.block_uid == block && p.direction == "out")
        .unwrap()
}

#[test]
fn each_refusal_is_made_alone_and_refused_by_name() {
    let base = solar("1.1");
    assert!(errors(&base).is_empty());
    let mut wrong = Vec::new();
    for (what, change, place, says) in REFUSALS {
        let mut f = base.clone();
        change(&mut f);
        let errs = errors(&f);
        if !errs.iter().any(|(p, w)| p == place && w.contains(says)) {
            wrong.push(format!(
                "{what}: expected at {place} \"{says}\", got {errs:#?}"
            ));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n\n"));
}

#[test]
fn the_assistants_are_the_ones_the_sheets_refuse() {
    for name in [
        "claude",
        "Claude",
        "agent",
        "assistant",
        "copilot",
        "ChatGPT",
        "gpt",
        "gemini",
        "claude code",
        "gpt/4",
    ] {
        assert!(checks::is_assistant(name), "{name}");
    }
    for name in ["Aman Rai", "Claudette Ng", "", "  "] {
        assert!(!checks::is_assistant(name), "{name:?}");
    }
}
