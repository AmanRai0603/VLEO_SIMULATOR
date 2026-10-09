//! An assistant's name never stands against a relation (AGENTS.md, rule 6).
//!
//! The check is `form::refuse_agent_attribution`, which the library's intake
//! and the node form's method both call. It was tested beside the save that
//! once wrote sheets; the save is gone, and the rule is not.

use vleo_sheet::form;

#[test]
fn an_agent_is_refused_whatever_the_face() {
    // The rule itself, as a function: no git, no files, no checkout. The names an
    // assistant arrives under, in the shapes a name arrives in.
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    for who in [
        "Claude",
        "claude opus 5",
        "Assistant",
        "Copilot",
        "Claude/Opus",
        "AGENT",
        "  claude  ",
    ] {
        assert!(
            form::refuse_agent_attribution(root, who).is_err(),
            "'{who}' must never be able to supply mathematics"
        );
    }
    for who in ["", "   "] {
        let e = form::refuse_agent_attribution(root, who).unwrap_err();
        assert!(
            e.message().contains("blank"),
            "a blank attribution is not a way round it: {e}"
        );
    }
    // And a person's name is not refused, or the rule would block the work it
    // exists to make possible.
    assert!(form::refuse_agent_attribution(root, "A. Rai").is_ok());
}
