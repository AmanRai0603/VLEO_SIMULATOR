//! The node engineer's own code, run again on the cases recorded from it.
//!
//! A node's cases came from its node engineer's code: MATLAB, Octave or
//! Python, kept beside the method as the record it came from. Running that
//! code again on each case and comparing with what was recorded says whether
//! the record still holds (docs/PLAN_F.md, decision 9). It is a check only:
//! the design's answer is always the method's, and nothing here feeds it.
//!
//! One implementation, for `xtask rerun` and for the Node workspace's *run my
//! code again*.

use std::fs;
use std::path::Path;
use std::process::Command;

use crate::model::Sheet;

/// What running the node engineer's code again said about one node.
#[derive(Clone, Debug, PartialEq)]
pub enum Rerun {
    /// Every case gave what the sheet records.
    Agrees(usize),
    /// Some did not; each line says which and how.
    Differs(Vec<String>),
    /// It could not be run here, and why — a language with no free runner, or
    /// a runner that is not installed. Kept and read, not rerun.
    NotRun(String),
}

fn which(cmd: &str) -> bool {
    Command::new(cmd)
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Run the node engineer's code on each of their cases and compare with what they
/// recorded. Python is called with each input as a keyword argument named by
/// its binding; MATLAB and Octave code with the inputs in the node's declared
/// order. A raised error or exception is a refusal.
pub fn rerun(sh: &Sheet, work: &Path) -> Result<Rerun, String> {
    let lang = sh.author.language.trim();
    let entry = sh.author.entry.trim();
    if sh.cases.is_empty() {
        return Ok(Rerun::NotRun(
            "no case was recorded from the node engineer's code".into(),
        ));
    }
    if sh.author.code.trim().is_empty() {
        return Ok(Rerun::NotRun(
            "the node engineer's code is not kept with this node".into(),
        ));
    }
    if entry.is_empty() {
        return Ok(Rerun::NotRun("the entry function is not named".into()));
    }
    fs::create_dir_all(work).map_err(|e| e.to_string())?;
    let order: Vec<&str> = sh.inputs.iter().map(|i| i.binding.as_str()).collect();
    let value = |c: &crate::method::Case, k: &str| {
        c.inputs
            .iter()
            .find(|(n, _)| n == k)
            .map(|(_, v)| *v)
            .unwrap_or(0.0)
    };
    let output = match lang {
        "Python" => {
            if !which("python3") {
                return Ok(Rerun::NotRun("python3 is not installed here".into()));
            }
            fs::write(work.join("author_code.py"), &sh.author.code).map_err(|e| e.to_string())?;
            let mut h = String::from(
                "import sys\nsys.path.insert(0, '.')\nimport author_code as m\nf = getattr(m, ",
            );
            h.push_str(&format!("{entry:?})\n"));
            for c in &sh.cases {
                let kw = order
                    .iter()
                    .map(|k| format!("{k}={:?}", value(c, k)))
                    .collect::<Vec<_>>()
                    .join(", ");
                h.push_str(&format!(
                    "try:\n    print(repr(float(f({kw}))))\nexcept Exception:\n    print('refused')\n"
                ));
            }
            fs::write(work.join("harness.py"), h).map_err(|e| e.to_string())?;
            Command::new("python3")
                .arg("harness.py")
                .current_dir(work)
                .output()
                .map_err(|e| e.to_string())?
        }
        "MATLAB" | "Octave" => {
            if !which("octave") {
                return Ok(Rerun::NotRun(format!(
                    "{lang} code is run with Octave, and Octave is not installed here"
                )));
            }
            fs::write(work.join(format!("{entry}.m")), &sh.author.code)
                .map_err(|e| e.to_string())?;
            let mut h = String::new();
            for c in &sh.cases {
                let args = order
                    .iter()
                    .map(|k| format!("{:?}", value(c, k)))
                    .collect::<Vec<_>>()
                    .join(", ");
                h.push_str(&format!(
                    "try\n  printf('%.17g\\n', {entry}({args}));\ncatch\n  printf('refused\\n');\nend\n"
                ));
            }
            fs::write(work.join("harness.m"), h).map_err(|e| e.to_string())?;
            Command::new("octave")
                .args(["--no-gui", "--quiet", "--eval", "harness"])
                .current_dir(work)
                .output()
                .map_err(|e| e.to_string())?
        }
        other => {
            return Ok(Rerun::NotRun(format!(
                "{} code is kept and read, not rerun — Python is run again, and MATLAB or \
                 Octave through Octave",
                if other.is_empty() { "unnamed" } else { other }
            )))
        }
    };
    let text = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = text
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();
    if lines.len() != sh.cases.len() {
        return Ok(Rerun::Differs(vec![format!(
            "the code printed {} line(s) for {} case(s): {}",
            lines.len(),
            sh.cases.len(),
            String::from_utf8_lossy(&output.stderr)
                .lines()
                .last()
                .unwrap_or("")
        )]));
    }
    let mut bad = Vec::new();
    for (c, got) in sh.cases.iter().zip(lines) {
        match (c.expect, got) {
            (None, "refused") => {}
            (None, g) => bad.push(format!(
                "«{}»: recorded as refused, the code gives {g}",
                c.label
            )),
            (Some(e), "refused") => bad.push(format!(
                "«{}»: recorded as {e}, the code refuses it",
                c.label
            )),
            (Some(e), g) => {
                let v: f64 = g.parse().unwrap_or(f64::NAN);
                let rel = if e == 0.0 {
                    v.abs()
                } else {
                    ((v - e) / e).abs()
                };
                // NaN — output that was not a number — is a disagreement too.
                if rel.is_nan() || rel > c.tolerance {
                    bad.push(format!(
                        "«{}»: recorded as {e}, the code now gives {v} ({rel:.2e} apart)",
                        c.label
                    ));
                }
            }
        }
    }
    Ok(if bad.is_empty() {
        Rerun::Agrees(sh.cases.len())
    } else {
        Rerun::Differs(bad)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::method::Case;
    use crate::model::{AuthorCode, Input};

    fn case(label: &str, h: f64, expect: Option<f64>) -> Case {
        Case {
            label: label.into(),
            inputs: vec![("h".into(), h)],
            expect,
            tolerance: 1e-9,
            also: Vec::new(),
            origin: "code".into(),
        }
    }

    /// A node whose node engineer wrote it in Python: twice the altitude,
    /// refusing below the ground.
    fn node(cases: Vec<Case>, language: &str) -> Sheet {
        Sheet {
            id: "test_double".into(),
            inputs: vec![Input {
                binding: "h".into(),
                var: "orbit_altitude".into(),
                ty: "Length".into(),
            }],
            author: AuthorCode {
                name: "Test Node Engineer".into(),
                language: language.into(),
                entry: "double".into(),
                code: "def double(h):\n    if h < 0:\n        raise ValueError('below the ground')\n    return 2 * h\n".into(),
                ..AuthorCode::default()
            },
            cases,
            ..Sheet::default()
        }
    }

    fn work(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("vleo-rerun-{name}-{}", std::process::id()))
    }

    #[test]
    fn the_node_engineer_s_code_is_held_to_the_cases_recorded_from_it() {
        if Command::new("python3").arg("--version").output().is_err() {
            eprintln!("python3 is not installed here: nothing to rerun");
            return;
        }
        let agree = node(
            vec![
                case("400 km", 400e3, Some(800e3)),
                case("below the ground", -1.0, None),
            ],
            "Python",
        );
        assert_eq!(rerun(&agree, &work("agree")).unwrap(), Rerun::Agrees(2));

        // A case recorded with another answer, and one recorded as a refusal
        // that the code answers: each named, with what the code gives.
        let differ = node(
            vec![
                case("400 km", 400e3, Some(801e3)),
                case("200 km", 200e3, None),
            ],
            "Python",
        );
        let Rerun::Differs(lines) = rerun(&differ, &work("differ")).unwrap() else {
            panic!("a case recorded wrongly agreed");
        };
        assert!(
            lines
                .iter()
                .any(|l| l.contains("«400 km»") && l.contains("801000")),
            "{lines:?}"
        );
        assert!(
            lines
                .iter()
                .any(|l| l.contains("«200 km»") && l.contains("recorded as refused")),
            "{lines:?}"
        );

        // Code in a language with no runner here is kept and read, and says so.
        let Rerun::NotRun(why) = rerun(&node(agree.cases.clone(), "C"), &work("c")).unwrap() else {
            panic!("C code was run");
        };
        assert!(why.contains("kept and read"), "{why}");
        for n in ["agree", "differ", "c"] {
            let _ = fs::remove_dir_all(work(n));
        }
    }
}
