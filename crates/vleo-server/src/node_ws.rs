//! What the Node workspace asks of the engine (docs/PLAN_F.md, stage F3).
//!
//! A node's six parts, each done, missing or failing, as the gate and the
//! sheet say (`vleo_sheet::parts`). The workspace draws them down its left
//! side; it decides none of them.

use super::json::Json;
use super::Ctx;

/// `GET /v1/parts/<id>` — a node's six parts, in the order the system model
/// gives them, each with its state and every reason for it.
pub(super) fn parts_json(ctx: &Ctx, id: &str) -> String {
    let mut j = Json::new();
    j.raw("{");
    let tree = match ctx.load() {
        Ok(t) => t,
        Err(e) => {
            j.bool_field("ok", false);
            j.str_field("node", id);
            j.str_field("message", &format!("the design does not load: {e}"));
            j.raw("}");
            return j.0;
        }
    };
    let Some(sh) = tree.sheets.get(id) else {
        j.bool_field("ok", false);
        j.str_field("node", id);
        j.str_field("message", &format!("no node '{id}' in the design"));
        j.raw("}");
        return j.0;
    };
    j.bool_field("ok", true);
    j.str_field("node", id);
    j.str_field("behaviour", sh.behaviour());
    j.key("parts").open_arr();
    for p in vleo_sheet::parts::parts(sh, &tree) {
        j.open_obj();
        j.str_field("name", p.name);
        j.str_field("state", p.state.name());
        j.key("why").open_arr();
        for (i, w) in p.why.iter().enumerate() {
            if i > 0 {
                j.raw(",");
            }
            j.push_string(w);
        }
        j.close_arr();
        j.close_obj();
    }
    j.close_arr();
    j.raw("}");
    j.0
}
