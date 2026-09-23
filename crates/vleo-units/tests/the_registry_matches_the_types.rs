//! `QUANTITIES` must name exactly the types `quantity!` declares.
//!
//! The list is maintained by hand because the macro is invoked once per type
//! and cannot accumulate one. A hand-maintained list that nothing checks is a
//! list that goes wrong the first time somebody adds a type — and the symptom
//! would be a sheet unable to declare a type that exists, or worse, a face
//! accepting one that does not.

#[test]
fn the_registry_matches_the_types() {
    let src = include_str!("../src/quantity.rs");
    // Every `quantity!(Name, …)` and every `quantity!(\n    Name,` form.
    let mut declared: Vec<String> = Vec::new();
    let mut lines = src.lines().peekable();
    while let Some(line) = lines.next() {
        let Some(rest) = line.strip_prefix("quantity!(") else {
            continue;
        };
        let name = if rest.trim().is_empty() {
            // The multi-line form: the name is on the next line.
            lines
                .next()
                .unwrap_or("")
                .trim()
                .trim_end_matches(',')
                .to_string()
        } else {
            rest.split(',').next().unwrap_or("").trim().to_string()
        };
        if !name.is_empty() {
            declared.push(name);
        }
    }
    declared.sort();
    assert!(
        declared.len() > 30,
        "only {} types were parsed out of quantity.rs — the parse is wrong, not the list",
        declared.len()
    );
    let mut registry: Vec<String> = vleo_units::QUANTITIES
        .iter()
        .map(|s| s.to_string())
        .collect();
    registry.sort();
    assert_eq!(
        declared, registry,
        "QUANTITIES and the quantity! invocations have drifted. A type missing from the \
         registry cannot be declared by a sheet; one in the registry that does not exist \
         generates Rust that will not compile."
    );
}
