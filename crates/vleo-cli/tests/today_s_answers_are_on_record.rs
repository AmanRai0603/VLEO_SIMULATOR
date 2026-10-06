//! Today's answers, on record before the engine changes underneath them.
//!
//! 1.0 moves the graph out of the compiled code and into the design file
//! (`docs/PLAN_1_0.md`, phase D). Its parity gate asks one question: does the
//! new engine give, for every row in every case, the answer this one gives —
//! and refuse where this one refuses, for the same reason? That question has no
//! answer unless this one's answers were written down first, by this engine,
//! before anything it measures moved. This file is that record, and this test
//! holds the engine to it exactly.
//!
//! What is recorded, in `baseline/today.csv`:
//! - every row of the whole design, run on each shared case, with the reference
//!   data and without it: its value as the shortest text that reads back to the
//!   same f64, its status and its credibility; and every refusal, by kind and
//!   message;
//! - every input of the case at both ends of its declared range, the rest at
//!   their defaults, by what it changes from the declared design: each row
//!   that is not a declared value and answers differently there, and each
//!   refusal that can move with an input (whether a row is written or
//!   confirmed cannot, and is recorded once per case above);
//! - every fixture each row carries, as the engine computes it today.
//!
//! A difference is never re-recorded to get green. It is either an engine
//! change that moved an answer, which is a defect, or a deliberate change to
//! the design, whose answers are recorded again on purpose:
//!
//!     VLEO_BASELINE=write cargo test -p vleo-cli --test today_s_answers_are_on_record

mod baseline;

use std::fmt::Write as _;

use baseline::{record_path, today};
use vleo_modules::COMPILED;

#[test]
fn today_s_answers_are_on_record() {
    let now = today(&COMPILED);
    let path = record_path();
    if std::env::var("VLEO_BASELINE").as_deref() == Ok("write") {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, &now).unwrap();
        return;
    }
    let was = std::fs::read_to_string(&path).unwrap_or_default();
    if was == now {
        return;
    }
    let mut said = String::new();
    let (a, b): (Vec<&str>, Vec<&str>) = (was.lines().collect(), now.lines().collect());
    for (n, (x, y)) in a.iter().zip(&b).enumerate() {
        if x != y {
            let _ = writeln!(
                said,
                "  line {}:\n    on record  {x}\n    today      {y}",
                n + 1
            );
            if said.lines().count() > 30 {
                break;
            }
        }
    }
    if a.len() != b.len() {
        let _ = writeln!(said, "  {} line(s) on record, {} today", a.len(), b.len());
    }
    panic!(
        "the engine no longer gives the answers on record in baseline/today.csv:\n{said}\
         An engine change that moves an answer is a defect. A deliberate change to the \
         design is recorded again on purpose: VLEO_BASELINE=write cargo test -p vleo-cli \
         --test today_s_answers_are_on_record"
    );
}
