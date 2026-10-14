//! *Run my code again*, in the Node workspace (docs/PLAN_F.md, stage F3 and
//! decision 9).
//!
//! The node engineer's own code is run on this computer, on each case
//! recorded from it, and compared with what was recorded
//! (`vleo_sheet::rerun`). A check only: the design's answer is the method's,
//! and nothing here feeds it.
//!
//! It runs code from a file of the design, so it runs only when a person asks:
//! a POST, never a GET a page might make on its own, and it works in a
//! scratch folder of its own, never beside the design.

use super::json::Json;
use super::Ctx;
use vleo_sheet::rerun::Rerun;

/// `POST /v1/rerun/<id>` — the node engineer's code, run again on their cases.
pub(super) fn rerun_json(ctx: &Ctx, id: &str) -> String {
    let mut j = Json::new();
    j.raw("{");
    let refuse = |j: &mut Json, why: &str| {
        j.bool_field("ok", false);
        j.str_field("node", id);
        j.str_field("message", why);
        j.raw("}");
    };
    let tree = match ctx.load() {
        Ok(t) => t,
        Err(e) => {
            refuse(&mut j, &format!("the design does not load: {e}"));
            return j.0;
        }
    };
    let Some(sh) = tree.sheets.get(id) else {
        refuse(&mut j, &format!("no node '{id}' in the design"));
        return j.0;
    };
    let work = std::env::temp_dir().join(format!("vleo-rerun-{}-{id}", std::process::id()));
    let said = vleo_sheet::rerun::rerun(sh, &work);
    let _ = std::fs::remove_dir_all(&work);
    let said = match said {
        Ok(s) => s,
        Err(e) => {
            refuse(&mut j, &format!("the code could not be run: {e}"));
            return j.0;
        }
    };
    j.bool_field("ok", true);
    j.str_field("node", id);
    j.str_field("language", &sh.author.language);
    j.num_field("cases", sh.cases.len() as f64);
    let (verdict, lines) = match said {
        Rerun::Agrees(_) => ("agrees", Vec::new()),
        Rerun::Differs(lines) => ("differs", lines),
        Rerun::NotRun(why) => ("not-run", vec![why]),
    };
    j.str_field("verdict", verdict);
    j.key("said").open_arr();
    for (i, l) in lines.iter().enumerate() {
        if i > 0 {
            j.raw(",");
        }
        j.push_string(l);
    }
    j.close_arr();
    j.raw("}");
    j.0
}
