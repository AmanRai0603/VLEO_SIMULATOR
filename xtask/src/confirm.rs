//! Today's date as the sheets write it, and a person's name against a relation.

use super::*;

/// Today, as the sheets write it.
pub(super) fn today() -> String {
    // The sheets carry a plain ISO date and nothing reads it as a timestamp, so
    // a date is all this needs. It once came from the `date` program, which
    // Windows does not have.
    vleo_units::calendar::Civil::from_unix(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0),
    )
    .date()
    .to_string()
}

/// The relations with nobody's name against them, and who owes each one.
///
/// Two thirds of the tree is declared values and every one of those already
/// carries a confirmation. What is missing is the other third: the relations,
/// where the name matters most, because a declared value that nobody confirmed
/// is a number somebody has to defend and a relation that nobody confirmed
/// might not have come from a person at all.
pub(super) fn cmd_confirm(root: &Path, args: &[&str]) -> Result<(), String> {
    let tree = load(root)?;

    if args.first() == Some(&"--list") {
        let only = args.get(1).copied();
        let mut by_owner: BTreeMap<&str, Vec<&vleo_sheet::model::Sheet>> = BTreeMap::new();
        for sh in tree.ordered() {
            if sh.state != "published" || sh.is_declared() {
                continue;
            }
            if !sh.relation_by.trim().is_empty() {
                continue;
            }
            if only.is_some_and(|o| sh.subsystem != o) {
                continue;
            }
            by_owner.entry(sh.owner.as_str()).or_default().push(sh);
        }
        let total: usize = by_owner.values().map(Vec::len).sum();
        if total == 0 {
            println!("every written relation has a name against it");
            return Ok(());
        }
        for (owner, rows) in &by_owner {
            println!("\n\x1b[1m{owner}\x1b[0m — {} relation(s)", rows.len());
            for sh in rows {
                println!("  {:<32} {}", sh.id, sh.expression);
                println!("  {:<32} source: {}", "", sh.source);
            }
        }
        println!(
            "\n{total} relation(s) with nobody's name against them, across {} owner(s).",
            by_owner.len()
        );
        println!(
            "Each needs: cargo xtask confirm <node> --by \"<your name>\"\n\
             A name here is a person saying the relation is right and is theirs. It is not\n\
             a formatting step, and nothing else in the tool can supply it."
        );
        return Ok(());
    }

    let id = args
        .first()
        .ok_or("usage: cargo xtask confirm <node> --by \"<name>\"  |  confirm --list")?;
    let who = args
        .iter()
        .position(|a| *a == "--by")
        .and_then(|i| args.get(i + 1))
        .ok_or("who is confirming it: --by \"<name>\"")?
        .trim();

    let sh = tree
        .sheets
        .get(*id)
        .ok_or_else(|| format!("no node '{id}'"))?;
    if sh.is_declared() {
        return Err(format!(
            "'{id}' is a declared value. Its confirmation belongs under [value], not [maths], \
             and it already has one: {}",
            sh.confirmed_by
        ));
    }
    if sh.expression.trim().is_empty() {
        return Err(format!("'{id}' has no relation to confirm"));
    }
    if who.is_empty() {
        return Err("--by is empty".into());
    }

    // An assistant may never supply mathematics. Stated as a sentence it is a hope;
    // this makes it a fact about what can be written to the file.
    let lower = who.to_lowercase();
    for bad in vleo_sheet::form::agent_identities(root) {
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
    if !sh.relation_by.trim().is_empty() {
        return Err(format!(
            "'{id}' is already confirmed by {}. Changing an attribution is a review decision, \
             not a command — edit the sheet and say why in the commit.",
            sh.relation_by
        ));
    }

    // What is being confirmed, in front of the person at the moment they
    // confirm it. A name put against a relation nobody re-read is a keystroke,
    // not a confirmation.
    println!("\n\x1b[1m{}\x1b[0m — {}", sh.id, sh.label);
    println!("  asks       {}", sh.question);
    println!("  relation   {}", sh.expression);
    println!("  source     {}", sh.source);
    if !sh.assumptions.is_empty() {
        for a in &sh.assumptions {
            println!("  assumes    {} — fails when {}", a.text, a.fails_when);
        }
    }
    println!(
        "  answers    {} in {} over {} … {}",
        sh.symbol, sh.unit, sh.lower, sh.upper
    );

    let stamp = format!("{who} / {}", today());
    let path = sh.dir.join("node.toml");
    let text = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;

    // Textual insertion rather than a TOML round-trip: these sheets carry more
    // comment than content, and every one of those comments is somebody's
    // reason. A serialiser would silently drop the lot.
    let at = text
        .find("\n[maths]\n")
        .map(|i| i + "\n[maths]\n".len())
        .ok_or_else(|| format!("{}: no [maths] section", path.display()))?;
    let line =
        format!("confirmed_by = {stamp:?}   # a person read this relation against its source\n");
    let out = format!("{}{}{}", &text[..at], line, &text[at..]);
    fs::write(&path, &out).map_err(|e| format!("{}: {e}", path.display()))?;

    // Read it back through the loader, not the string we just wrote. Writing a
    // file and announcing success is how a malformed sheet gets found three
    // nodes later.
    let after = load(root)?;
    let got = after
        .sheets
        .get(*id)
        .map(|s| s.relation_by.clone())
        .unwrap_or_default();
    if got != stamp {
        fs::write(&path, &text).map_err(|e| format!("{}: {e}", path.display()))?;
        return Err(format!(
            "the sheet did not read back as expected (got {got:?}, wanted {stamp:?}). The file \
             has been put back as it was."
        ));
    }
    println!("\n  confirmed by {stamp}");
    println!("Now: cargo xtask gate {id} && cargo xtask ready {id}");
    Ok(())
}
