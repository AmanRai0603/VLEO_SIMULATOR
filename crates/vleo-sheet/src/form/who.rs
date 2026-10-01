//! The declaration form: who is answering: a person, never an assistant.

use super::*;

/// Every name that is an assistant, not a person, lowercased.
///
/// A relation is confirmed by a person who has read it against its source — the
/// developer who applies a node form, or who writes the relation in. An
/// assistant may implement a relation a person supplied; it may never be the one
/// who supplied it. These are the names an assistant arrives under.
pub fn agent_identities(_root: &std::path::Path) -> Vec<String> {
    [
        "claude",
        "agent",
        "assistant",
        "copilot",
        "chatgpt",
        "gpt",
        "gemini",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}

/// Who this checkout says it is, from `git config user.name`.
///
/// NOT typed into the form. An attribution a person types is a name they chose
/// for that box; this is the name their commits already carry, so the sheet and
/// the history agree about who did it and nobody can put a colleague's name on
/// their own work by typing it.
///
/// It is not authentication and this does not pretend otherwise: anyone who can
/// edit a checkout can edit its git config. What it removes is the casual case —
/// typing somebody else's name into a text box — and it makes the sheet's
/// attribution and the commit's author the same claim rather than two.
pub fn git_identity(root: &std::path::Path) -> Result<String, String> {
    let out = std::process::Command::new("git")
        .args(["-C"])
        .arg(root)
        .args(["config", "user.name"])
        .output()
        .map_err(|e| format!("git could not be run: {e}"))?;
    let name = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if name.is_empty() {
        return Err(
            "this checkout has no `git config user.name`, so there is no name to put against \
             the relation. Set it — `git config user.name \"Your Name\"` — and the sheet will \
             carry the same name your commits do. Nothing was written."
                .into(),
        );
    }
    Ok(name)
}

/// Whether this attribution is an assistant's, and so must never be written.
///
/// An assistant may never supply mathematics. Stated as a sentence that is a hope;
/// here it is a fact about what can reach the file — and it has to hold at every
/// face, or the browser becomes the way round a rule the terminal enforces.
pub fn refuse_agent_attribution(root: &std::path::Path, who: &str) -> Result<(), String> {
    let lower = who.trim().to_lowercase();
    if lower.is_empty() {
        return Err(
            "an attribution cannot be blank — it takes the name of a person who has \
                    read the relation against its source and is prepared to own it"
                .into(),
        );
    }
    for bad in agent_identities(root) {
        if lower == bad
            || lower.starts_with(&format!("{bad} "))
            || lower.contains(&format!("{bad}/"))
        {
            return Err(format!(
                "refused: '{who}' is an assistant's name. An assistant may never supply mathematics, and this \
                 field is the only thing that can tell whether one did. It takes the name of a \
                 person who has read the relation against its source and is prepared to own it. \
                 Nothing was written."
            ));
        }
    }
    Ok(())
}

/// Today, as the sheets write it.
fn today() -> String {
    crate::template::today()
}

/// Put a name against the relation, replacing whatever was there.
///
/// WHEN THE RELATION CHANGES THE ATTRIBUTION MUST MOVE WITH IT. The old name
/// was against the old mathematics; leaving it on the new attributes work to
/// somebody who never saw it, which is worse than either having no name or
/// having the editor's.
pub(crate) fn stamp_relation(text: &str, who: &str) -> Result<String, String> {
    set(text, "confirmed_by", &format!("{who} / {}", today()))
}
