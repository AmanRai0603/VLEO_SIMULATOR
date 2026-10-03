//! The method language: the reference page.

use super::*;

/// `docs/PSEUDOCODE.md`, from the tables above, so the page and the checker
/// cannot describe two different languages — with the worked example from
/// `crate::example`, the one every node form shows.
pub fn reference_md() -> String {
    let example_node = crate::example::TITLE;
    let example_method = crate::example::METHOD;
    let mut o = String::new();
    o.push_str("<!-- GENERATED from crates/vleo-sheet/src/method.rs by `cargo run -p xtask -- docs`. Do not edit. -->\n");
    let _ = writeln!(o, "# The method language, version {LANGUAGE_VERSION}\n");
    o.push_str(
        "> **Answer first.** Every node's relation is written once more as a *method*: a few lines in a\n\
         > small fixed language that the tool can check, run and translate. The checker refuses a\n\
         > sum of unlike units, a logarithm of a length, an answer of the wrong quantity, and a path\n\
         > that ends without an answer or a refusal — before any code exists.\n\
         >\n\
         > **Kind:** reference · **For:** node authors and developers\n\n",
    );
    o.push_str("## Said simply\n\n");
    o.push_str(
        "A method is the recipe for the node's answer, written so a machine can follow it exactly. It\n\
         reads the node's inputs by their names, may use the constants below, works out named values\n\
         with `let`, and ends every path with `return` (the answer) or `refuse` (the reason it will not\n\
         answer). Every number carries its unit: write `6371 [km]`, and the tool works in SI from there.\n\n",
    );
    o.push_str("## Why a method, when my code already works\n\n");
    o.push_str(
        "Your code produced your test cases. The method is a second, independent statement of the same\n\
         relation, and the Rust the tool ships is generated from the method. The three are compared on\n\
         your cases: if any one of them is wrong, a case disagrees, and it is caught before the change\n\
         reaches anyone.\n\n",
    );
    o.push_str("## Statements\n\n| Form | Meaning | Example |\n|---|---|---|\n");
    for s in STATEMENTS {
        let _ = writeln!(
            o,
            "| `{}` | {} | `{}` |",
            s.form.replace('|', "\\|"),
            s.meaning.replace('|', "\\|"),
            s.example.replace('\n', " ⏎ ").replace('|', "\\|")
        );
    }
    o.push_str(
        "\nOperators: `+ - * / ^`, comparisons `< <= > >= == !=`, and `and`, `or`, `not`. A power of a\n\
         dimensioned value is written as a number (`r ^ 3`, `a ^ 0.5` when every exponent stays whole).\n\
         A bare `0` is zero of any unit; any other number that is not a pure ratio needs its unit.\n\n",
    );
    o.push_str("## Functions\n\n| Function | Units | Meaning |\n|---|---|---|\n");
    for f in FUNCTIONS {
        let rule = match f.rule {
            FnRule::Pure => "a pure number in, a pure number out",
            FnRule::Same => "keeps the unit",
            FnRule::SameTwo => "two like values, that unit out",
            FnRule::Ratio => "two like values, a pure number out",
            FnRule::Sqrt => "halves every exponent (m^2 → m)",
            FnRule::Cbrt => "thirds every exponent",
            FnRule::Interp => "x like the table's x row; the y row's unit out",
            FnRule::Pow => "as ^",
        };
        let _ = writeln!(o, "| `{}` | {} | {} |", f.name, rule, f.meaning);
    }
    o.push_str(
        "\n## Kernel functions\n\nRelations too long to write as a formula — an integral up the \
         atmosphere, a decay over many orbits — already live in the kernel, reviewed once. A method \
         calls them by name, each argument in the unit shown; the checker holds the units, and the \
         engine runs the kernel itself, so the method and the built node cannot differ.\n\n\
         | Call | Answer | Meaning | In the kernel |\n|---|---|---|---|\n",
    );
    for k in KERNEL_FUNCTIONS {
        let args: Vec<String> = k.args.iter().map(|(n, u)| format!("{n} [{u}]")).collect();
        let _ = writeln!(
            o,
            "| `{}({})` | `[{}]` | {} | `{}` |",
            k.name,
            args.join(", "),
            k.out,
            k.meaning,
            k.kernel
        );
    }
    o.push_str("\n## Constants every method may use\n\n| Name | Value | Unit | Meaning |\n|---|---|---|---|\n");
    for c in KERNEL_CONSTANTS {
        let _ = writeln!(
            o,
            "| `{}` | {:e} | `{}` | {} |",
            c.name, c.value, c.unit, c.meaning
        );
    }
    o.push_str("\n## Units\n\nWrite a unit in brackets straight after a number: `7.8 [km/s]`, `3.986e14 [m^3/s^2]`,\n`30 [deg]`, `1 [1]` for a pure number. Every symbol a sheet may declare is accepted:\n\n");
    let syms: Vec<String> = Unit::NAMES
        .iter()
        .filter_map(|n| Unit::from_name(n))
        .map(|u| format!("`{}`", u.symbol()))
        .collect();
    o.push_str(&syms.join(" · "));
    o.push_str(
        "\n\nand the simple ones combine with `*` or `.`, `/` and whole-number powers `^`.\n\n",
    );
    let _ = writeln!(o, "## The worked example: {example_node}\n");
    let _ = writeln!(
        o,
        "*{}.* The same example sits behind **Show the example** on every node form.\n",
        crate::example::TAG
    );
    o.push_str("**The method.**\n\n```text\n");
    o.push_str(example_method.trim_end());
    o.push_str("\n```\n\n");
    let f = |k: &str| crate::example::field(k).unwrap_or("");
    let _ = writeln!(
        o,
        "**The author's own code** ({}, `{}`), which produced the cases below.\n\n```text\n{}\n```\n",
        f("author_language"),
        f("author_entry"),
        f("author_code")
    );
    let _ = writeln!(
        o,
        "**The test code** that ran it on each case.\n\n```text\n{}\n```\n",
        f("author_test_code")
    );
    o.push_str("**The cases**, in SI, and what the method gives for each.\n\n| Case | Inputs | The author's code | The method |\n|---|---|---|---|\n");
    if let Ok(r) = report_toml(&crate::example::sheet_text()) {
        for (i, (c, v)) in r.cases.iter().enumerate() {
            let inputs = c
                .inputs
                .iter()
                .map(|(k, x)| format!("{k} = {}", show(*x)))
                .collect::<Vec<_>>()
                .join(", ");
            let got = match r.got.get(i).copied().flatten() {
                Some(g) if !c.refuses() => format!("{} — {}", show(g), v.text(c)),
                _ => v.text(c),
            };
            let _ = writeln!(
                o,
                "| {} | `{inputs}` | {} | {got} |",
                c.label,
                c.expect.map(show).unwrap_or_else(|| "refuses".into())
            );
        }
    }
    o.push('\n');
    o.push_str("## Where the simple version breaks\n\n");
    o.push_str(
        "- **One answer per method.** A node that publishes a set of values (a few rows do) keeps its\n\
           \x20 hand-written hole for now; the method language answers one quantity.\n\
         - **No iteration to convergence.** A loop runs a fixed count. A solver that stops when it\n\
           \x20 converges is marked for a developer, who writes it, and your cases still decide.\n\
         - **Tables are written out.** A lookup into a large data file is a reference-data bundle,\n\
           \x20 not a method.\n",
    );
    o
}
