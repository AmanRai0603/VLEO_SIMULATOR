//! A node's page as the copy's revision of it would make it, over today's
//! design (docs/briefs/F3.md, F3-g2b).
//!
//! `GET /v1/node/<id>/live` reads the node's file in the copy as a release's
//! node (`vleo_files::intake::copy_form`), plans it against today's design
//! as today's design plans a release (`vleo_sheet::template::
//! plan_form_against`), and lays the sheet the plan writes over today's
//! design for this request only (`today::Overlay`). The page is drawn by the
//! page's own fragment (`vleo_sheet::page::fragment`). Nothing of today's
//! design is written, and nothing is run: the engine's graph is today's.
//!
//! It is drawn only as today's design would take it, and refused where
//! today's design would refuse a release holding it (`today::take`): a node
//! file whose own checks find an error (`vleo_files::checks::check_node`:
//! its declaration, its cases' origins, its method against its cases), and a
//! plan that withholds anything, since one withheld change refuses the whole
//! release. Either is said, with what refuses it, and the page is today's.
//!
//! The answer says what it is: the copy's revision over today's design, not
//! a release and not today's design. Beside the page, what it changes, what
//! the plan withholds and why, and what the copy's file says that a release
//! does not bring into today's design (its question and its explanation's
//! texts). A revision that changes nothing on the page says so, and the page
//! is today's.

use super::*;
use vleo_files::edit::{self, Part};
use vleo_sheet::template::Verdict;

/// `GET /v1/node/<id>/live`.
pub(super) fn live(id: &str, ctx: &Ctx) -> String {
    drawn(id, ctx).unwrap_or_else(|why| me::refusal(&why))
}

fn drawn(id: &str, ctx: &Ctx) -> Result<String, String> {
    let id = node_file::node_id(id)?;
    let copy = node_file::the_copy(ctx)?;
    let path = node_file::find(&copy, id)?;
    let (bytes, f) = node_file::read_once(&copy, &path)?;
    let group = f.meta.get("group_id").map_or("", String::as_str);
    if !path.starts_with(&format!("groups/{group}/nodes/")) {
        return Err(format!(
            "{path} says it is {group:?}'s, and is not in that group's folder: it is not drawn"
        ));
    }
    let tree = ctx
        .load()
        .map_err(|e| format!("today's design does not load, so nothing is drawn over it: {e}"))?;
    let sh = tree.sheets.get(id).ok_or_else(|| {
        format!(
            "today's design holds no node {id}, so your revision has nothing to be drawn over: \
             a node joins it when its group seals a release that holds it"
        )
    })?;
    let revision = f.meta.get("revision").map_or("", String::as_str);
    let mut j = Json::new();
    j.raw("{");
    j.bool_field("ok", true);
    j.str_field("node", id);
    j.str_field("path", &path);
    j.str_field("opened", &vleo_files::copied::digest(&bytes));
    j.str_field("revision", revision);
    j.str_field("writer", f.meta.get("writer").map_or("", String::as_str));
    let uid = f.meta.get("block_uid").map_or("", String::as_str);
    not_read(&mut j, &f, uid, sh);
    let today_page = || vleo_sheet::page::fragment(sh, &tree);

    // Its own checks, as a release's are checked before it is taken.
    let found = vleo_files::checks::check_node(&f);
    j.key("checks").open_arr();
    for e in found.errors() {
        j.open_obj();
        j.str_field("place", &e.place);
        j.str_field("what", &e.what);
        j.close_obj();
    }
    j.close_arr();
    if !found.holds() {
        j.bool_field("drawn", false);
        j.str_field(
            "said",
            &format!(
                "Your revision {revision} is not drawn: its checks find what today's design would refuse a release of it for. The page is today's."
            ),
        );
        j.str_field("html", &today_page());
        j.raw("}");
        return Ok(j.0);
    }

    // The plan, as today's design would make it of a release holding this.
    let node_toml = sh.dir.join("node.toml");
    let now = ctx
        .tree
        .read_to_string(&node_toml)
        .map_err(|e| format!("{}: {e}", node_toml.display()))?;
    let planned =
        vleo_files::intake::copy_form(&ctx.root, &f, sh, vleo_sheet::form::file_hash(&now))
            .and_then(|(form, not_taken)| {
                vleo_sheet::template::plan_form_against(&tree, form)
                    .map(|p| (p, not_taken))
                    .map_err(|e| e.to_string())
            });
    let (plan, not_taken) = match planned {
        Ok(p) => p,
        Err(why) => {
            j.bool_field("drawn", false);
            j.str_field(
                "said",
                &format!(
                    "Your revision {revision} is not drawn over today's design: {why}. The page is today's."
                ),
            );
            j.str_field("html", &today_page());
            j.raw("}");
            return Ok(j.0);
        }
    };
    j.key("changes").open_arr();
    for i in plan.items.iter().filter(|i| i.verdict == Verdict::Apply) {
        j.open_obj();
        j.str_field("what", &i.what);
        j.str_field("from", &i.from);
        j.str_field("to", &i.to);
        j.close_obj();
    }
    j.close_arr();
    j.key("withheld").open_arr();
    for i in &plan.items {
        if let Verdict::Conflict(why) | Verdict::Refused(why) = &i.verdict {
            j.open_obj();
            j.str_field("what", &i.what);
            j.str_field("why", why);
            j.close_obj();
        }
    }
    if let Some(why) = &not_taken {
        j.open_obj();
        j.str_field("what", "the method's declaration");
        j.str_field(
            "why",
            &format!("transcribed, but {why}: taken as a relation an assistant supplied"),
        );
        j.close_obj();
    }
    j.close_arr();
    // One change withheld refuses the whole release (`today::take`), so
    // nothing of it is drawn: not the changes that would apply alone.
    if plan.blocked() > 0 || not_taken.is_some() {
        j.bool_field("drawn", false);
        j.str_field(
            "said",
            &format!(
                "Your revision {revision} is not drawn: today's design would refuse a release of it whole, for what is withheld. The page is today's."
            ),
        );
        j.str_field("html", &today_page());
        j.raw("}");
        return Ok(j.0);
    }
    if plan.version.is_some() {
        j.str_field(
            "record",
            "Its record of which belief broke is the node's own, the last row of its versions.csv. A group's release is taken with the group's record, and nothing carries a node's row into it yet (question 12).",
        );
    }

    let applies = plan.applicable() > 0;
    let Some(text) = plan.text.filter(|_| applies) else {
        j.bool_field("drawn", true);
        j.bool_field("changes_nothing", true);
        j.str_field(
            "said",
            &format!(
                "Your revision {revision} changes nothing on today's page: what a release of it would bring is what today's design holds. The page is today's."
            ),
        );
        j.str_field("html", &today_page());
        j.raw("}");
        return Ok(j.0);
    };
    let over = today::Overlay {
        base: ctx.tree.clone(),
        replaced: BTreeMap::from([(node_toml, text.into_bytes())]),
    };
    let yours = match vleo_sheet::load::load_all_from(&over, &ctx.root) {
        Ok(t) => t,
        Err(e) => {
            j.bool_field("drawn", false);
            j.str_field(
                "said",
                &format!(
                    "Your revision {revision} does not load over today's design, so it is not drawn: {e}. The page is today's."
                ),
            );
            j.str_field("html", &today_page());
            j.raw("}");
            return Ok(j.0);
        }
    };
    let Some(mine) = yours.sheets.get(id) else {
        return Err(format!(
            "your revision of {id}, laid over today's design, holds no sheet {id}: it is not drawn"
        ));
    };
    j.bool_field("drawn", true);
    j.bool_field("changes_nothing", false);
    j.str_field(
        "said",
        &format!(
            "Your revision {revision}, from your copy, over today's design: not sealed, not in today's design, and not run. Nothing of today's design is written."
        ),
    );
    j.str_field("html", &vleo_sheet::page::fragment(mine, &yours));
    j.raw("}");
    Ok(j.0)
}

/// What the copy's file says that a release does not bring into today's
/// design, where it differs from today's sheet: the node's question and its
/// explanation's three texts, which stay the design's own until one node
/// shape is picked (F3-g0).
fn not_read(j: &mut Json, f: &vleo_files::model::File, uid: &str, sh: &vleo_sheet::model::Sheet) {
    let question = f
        .blocks
        .iter()
        .find(|b| b.uid == uid)
        .map_or(String::new(), |b| b.question.clone());
    let parts = [
        ("its question", question, &sh.question),
        (
            "said simply",
            edit::section(f, uid, Part::Simply),
            &sh.explain.simply,
        ),
        (
            "where it breaks",
            edit::section(f, uid, Part::Breaks),
            &sh.explain.breaks,
        ),
        (
            "the common misreading",
            edit::section(f, uid, Part::Wrong),
            &sh.explain.wrong,
        ),
    ];
    j.key("not_taken").open_arr();
    for (what, yours, page) in parts {
        if yours.trim() != page.trim() {
            j.open_obj();
            j.str_field("what", what);
            j.str_field("yours", yours.trim());
            j.str_field("page", page.trim());
            j.close_obj();
        }
    }
    j.close_arr();
}
