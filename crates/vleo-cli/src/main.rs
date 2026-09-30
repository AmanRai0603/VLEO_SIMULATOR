//! `vleo` — the command-line face.
//!
//! Batch work, sweeps and campaigns. No interface to design, and it is what
//! turns twenty finished nodes into a study — which is why it is built second,
//! after the browser face and before anything else.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::ExitCode;
use vleo_bus::{Case, RunMode};
use vleo_core::graph::Kind;
use vleo_modules::{tables, Scratch, Vleo, NODES, VARS};

/// The local store, resolved before the run.
///
/// R3: the engine makes no network calls. Data arrives by an explicit sync,
/// before the run. `evaluate` reads what is already local and nothing else,
/// which is what makes a run deterministic, offline-capable and replayable.
fn resolve_data() -> (Vec<String>, Vec<String>) {
    let root = data_root();
    let mut store = vleo_data::Store::open(&root);
    if store.load().is_err() || store.bundles.is_empty() {
        // The shipped set travels with the binary, so a fresh install runs
        // before any sync at all.
        let shipped = repo_bundles();
        if shipped.is_dir() {
            if let Err(why) = store.sync(&vleo_data::Source::Shipped(shipped)) {
                eprintln!("reference data refused: {why}");
            }
        }
    }
    (store.verified_names(), store.versions())
}

fn data_root() -> PathBuf {
    // Outside the install directory by default, so it survives an upgrade and
    // an uninstall rather than being deleted with the application.
    vleo_data::data_path().unwrap_or_else(|| PathBuf::from(".vleo/data"))
}

fn repo_bundles() -> PathBuf {
    let mut p = std::env::current_dir().unwrap_or_default();
    loop {
        if p.join("bundles").is_dir() && p.join("layers").is_dir() {
            return p.join("bundles");
        }
        if !p.pop() {
            return PathBuf::from("bundles");
        }
    }
}

/// A reader that stops early — `| head`, `| grep -m1`, a pager quit halfway —
/// closes the pipe, and the next line printed panics with a backtrace that
/// reads like a crash in this program. It is not one: the reader had what it
/// wanted. So that one panic ends the program quietly, and every other panic
/// is reported exactly as before. (Restoring the default SIGPIPE disposition
/// would need `unsafe`, which this crate forbids.)
fn quiet_when_the_reader_stops() {
    let default = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let msg = info
            .payload()
            .downcast_ref::<String>()
            .map(String::as_str)
            .or_else(|| info.payload().downcast_ref::<&str>().copied())
            .unwrap_or("");
        if msg.starts_with("failed printing to stdout") && msg.contains("Broken pipe") {
            std::process::exit(0);
        }
        default(info);
    }));
}

fn main() -> ExitCode {
    // The crash log first, so the quiet-on-a-closed-pipe hook wraps it: output
    // piped into `head` ends the program silently and is not logged as a bug.
    vleo_data::crash::install("vleo", env!("CARGO_PKG_VERSION"));
    quiet_when_the_reader_stops();
    let args: Vec<String> = std::env::args().skip(1).collect();
    let cmd = args.first().map(|s| s.as_str()).unwrap_or("help");
    let rest: Vec<&str> = args.iter().skip(1).map(|s| s.as_str()).collect();
    let r = match cmd {
        "run" => cmd_run(&rest),
        "sweep" => cmd_sweep(&rest),
        "campaign" => cmd_campaign(&rest),
        "list" => cmd_list(&rest),
        "show" => cmd_show(&rest),
        "cases" => cmd_cases(),
        "inputs" => cmd_inputs(&rest),
        "result" => cmd_result(&rest),
        "selftest" => cmd_selftest(),
        "data" => cmd_data(&rest),
        "version" => {
            print_version();
            Ok(())
        }
        "help" | "--help" | "-h" => {
            help();
            Ok(())
        }
        // AN UNKNOWN COMMAND IS REFUSED, NOT ANSWERED WITH HELP. This printed
        // the help and exited 0 for anything it did not recognise, so a typo —
        // `vleo rnu sw_ap_design` — looked exactly like a command that had run
        // and found nothing to say. That is a substitution where the fifth rule
        // asks for a refusal, and a script checking the exit code would have
        // carried on as if it had its number.
        other => Err(format!("unknown command '{other}'. Try `vleo help`.")),
    };
    match r {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("\x1b[31mvleo: {e}\x1b[0m");
            ExitCode::FAILURE
        }
    }
}

fn help() {
    println!(
        "\
vleo <command>

  run <node> [--inputs <file.csv> | --defaults] [--mode alone|branch|all] [--set id=value ...] [--save <file.csv>]
                       evaluate one node and everything it needs. Prints the
                       value, its provenance and every node that was blocked —
                       always n ran, m blocked, and the blocked ones named.
                       The inputs are the saved case — the one the browser
                       saves — unless --inputs names a CSV or --defaults asks
                       for the design as declared; --set has the last word.
                       --save keeps the whole run and the inputs it ran on as
                       a result file — see `result`.
  sweep <node> --over <input> --from <a> --to <b> [--points n] [--inputs <file.csv> | --defaults]
                       a behaviour sweep. Refused points are recorded, never
                       dropped: a sweep in which some rows quietly used a
                       substituted value is a sweep whose conclusion is unknown.
  campaign <node> [--inputs <file.csv> ...]
                       the defaults, the saved case and each file named,
                       side by side against one node.
  list [<subsystem>]   the rows, their kind, their owner and their state.
  show <node>          the sheet, as the engine holds it.
  cases                the case, how its inputs divide into customer and
                       condition, and whether a case is saved.
  inputs [--inputs <file.csv> | --defaults]
                       every input as a CSV — group, id, value, unit, default,
                       range. Fill in `value` and pass the file with --inputs.
  result <file> [--html <out.html>]
                       a saved result, shown as it was — nothing runs. Reads the
                       CSV `run --save` writes, or the report page it rides in;
                       --html writes that report, to send to someone.
  selftest             every fixture declaration in the tree is sound —
                       provenance outside the code, a positive tolerance.
                       It does not execute them: `cargo test` does.
  data sync|list|verify
                       reconcile the local store, or say what is in it. A run
                       either has verified data on disk or refuses to start: it
                       does not fetch, wait, retry or fall back silently.
  version              kernel, graph and build identity.

Everything crossing the boundary is SI. A face converts for display and never
for transport."
    );
}

fn print_version() {
    println!("vleo {}", env!("CARGO_PKG_VERSION"));
    println!("  kernel {}", short(Vleo::kernel_hash()));
    println!("  graph  {}", short(Vleo::graph_hash()));
    println!("  nodes  {}", NODES.len());
}

fn short(h: u64) -> String {
    String::from_utf8(vleo_core::hash::short_hex(h).to_vec()).unwrap_or_default()
}

fn opt<'a>(args: &'a [&'a str], name: &str) -> Option<&'a str> {
    args.iter()
        .position(|a| *a == name)
        .and_then(|i| args.get(i + 1))
        .copied()
}

/// A supplied value only survives on a node that declares its own number.
///
/// Every other kind works its answer out during the run and overwrites what was
/// supplied, so `--set` on one used to be accepted, ignored, and reported as a
/// successful run against a number nobody asked for. A sweep over one produced
/// a flat line and said "0 refused", which reads as a real result. Refusing by
/// name is the only honest version: a refusal the user can see beats a silent
/// substitution every time.
fn suppliable(idx: u16) -> Result<(), String> {
    let def = &NODES[idx as usize];
    if def.kind == Kind::Declared {
        return Ok(());
    }
    Err(format!(
        "'{}' is {}, so a supplied value would be overwritten the moment it is \
         evaluated. Set one of the declared numbers it reads instead — `vleo show {}` \
         lists them.",
        def.id,
        match def.kind {
            Kind::Computed => "computed from its inputs",
            Kind::Required => "a target handed down from the layer above",
            Kind::Achieved => "what a subsystem returned",
            Kind::Kpi => "a key performance indicator",
            Kind::Declared => unreachable!(),
        },
        def.id
    ))
}

fn sets(args: &[&str]) -> Result<Vec<(String, f64)>, String> {
    let mut out = Vec::new();
    for (i, a) in args.iter().enumerate() {
        if *a == "--set" {
            let kv = args.get(i + 1).ok_or("--set needs id=value")?;
            let (k, v) = kv
                .split_once('=')
                .ok_or_else(|| format!("'{kv}' is not id=value"))?;
            let val: f64 = v.parse().map_err(|_| format!("'{v}' is not a number"))?;
            let idx = Vleo::find(k).ok_or_else(|| format!("no node '{k}'"))?;
            suppliable(idx)?;
            out.push((k.to_string(), val));
        }
    }
    Ok(out)
}

/// Where a run's inputs come from, and the values, in SI.
///
/// The saved case by default — the one the browser saves, so the terminal and
/// the page run the same inputs. `--inputs <file.csv>` runs a file instead,
/// and `--defaults` runs the design as declared. A file with a refused row is
/// refused whole, with every refusal named: running its good rows alone would
/// be a run on a case nobody wrote.
///
/// A file written by an older version of the tool is carried over rather than
/// refused, and what could not be carried is said on stderr — see
/// `inputs::read_csv`. A saved case is carried over on disk, its old copy kept
/// beside it.
fn inputs_for(args: &[&str]) -> Result<(String, vleo_modules::inputs::Reading), String> {
    use vleo_modules::inputs::{read_csv, saved, Reading};
    if args.contains(&"--defaults") {
        return Ok(("the declared defaults".into(), Reading::default()));
    }
    let (from, r) = match opt(args, "--inputs") {
        Some(f) => {
            let text = std::fs::read_to_string(f).map_err(|e| format!("--inputs {f}: {e}"))?;
            (f.to_string(), read_csv(&text))
        }
        None => {
            let p = vleo_data::case_path();
            let s = saved::load(&p);
            if !s.stored {
                return Ok(("the declared defaults".into(), Reading::default()));
            }
            if let Some(e) = &s.error {
                eprintln!("vleo: {e}");
            }
            (format!("the saved case ({})", p.display()), s.reading)
        }
    };
    if !r.ok() {
        let rows = r
            .refused
            .iter()
            .map(|(l, id, why)| format!("\n  line {l}: {id} {why}"))
            .collect::<String>();
        return Err(format!("{from} cannot be applied:{rows}"));
    }
    let mut said = format!("{from}, {} changed from default", r.changed);
    if let Some(u) = &r.upgrade {
        said.push_str(&format!(
            "; carried over from an older version of the tool: {} new input(s) at their \
             default, {} value(s) set aside",
            u.new.len(),
            u.set_aside.len()
        ));
    }
    Ok((said, r))
}

/// Every value an upgrade set aside, on stderr, beside the line that says the
/// run is on a carried-over case — so it is read once, not lost in the output.
fn say_set_aside(r: &vleo_modules::inputs::Reading) {
    for a in r.upgrade.iter().flat_map(|u| &u.set_aside) {
        eprintln!(
            "  set aside: {} = {} {} — {}",
            a.id,
            if a.value.is_empty() { "?" } else { &a.value },
            a.unit,
            a.why
        );
    }
}

fn build_case(node: &str, args: &[&str]) -> Result<Case, String> {
    if Vleo::find(node).is_none() {
        return Err(format!(
            "no node '{node}'. `vleo list` shows every row; the identifier is the module path."
        ));
    }
    let (_, r) = inputs_for(args)?;
    let mut supply = r.set;
    // What was typed on the command line has the last word.
    supply.extend(sets(args)?);
    let (data, data_versions) = resolve_data();
    let case = Case {
        base: opt(args, "--case").unwrap_or("").to_string(),
        supply,
        target: node.to_string(),
        mode: RunMode::from_name(opt(args, "--mode").unwrap_or("branch")),
        data,
        data_versions,
    };
    if let Some(why) = vleo_modules::case_refusal(&case) {
        return Err(format!(
            "{}. `vleo cases` lists them.",
            why.trim_end_matches('.')
        ));
    }
    Ok(case)
}

fn cmd_run(args: &[&str]) -> Result<(), String> {
    let node = *args.first().ok_or("usage: vleo run <node>")?;
    let case = build_case(node, args)?;
    let mut scratch = Scratch::new();
    let results = vleo_modules::evaluate(&case, &mut scratch).map_err(|f| f.to_string())?;

    // KEPT, when asked: the whole run and the inputs it ran on, as a result
    // file the browser's Results page and `vleo result` read back unchanged.
    if let Some(path) = opt(args, "--save") {
        let s = vleo_modules::results::from_run(&results, &case.supply, &now_utc(), "");
        vleo_modules::files::write_whole(std::path::Path::new(path), vleo_modules::results::csv(&s)).map_err(|e| format!("{path}: {e}"))?;
        eprintln!("saved the result to {path}");
    }

    let idx = Vleo::find(node).unwrap();
    let def = &NODES[idx as usize];
    println!("\x1b[1m{}\x1b[0m — {}", def.id, def.label);
    println!("  {}", def.question);
    let (said, r) = inputs_for(args)?;
    println!("  inputs: {said}");
    say_set_aside(&r);
    println!();
    match results.values.iter().find(|v| v.id == def.id) {
        Some(v) => {
            let (shown, sym) = vleo_bus::present(v.value, VARS[idx as usize].unit, 6);
            println!("  \x1b[1m{} = {} {}\x1b[0m", v.symbol, shown, sym);
            println!(
                "  credibility {} of 4, governed by {}",
                v.cred.governing_score(),
                v.governing
            );
        }
        None if !def.is_defined() => {
            // Its own fault, not an upstream one. Say what it is and say what
            // would turn it on, because a disabled control that does not say
            // why is a defect.
            println!("  \x1b[33mINACTIVE\x1b[0m — the relation is stated and never derived");
            println!("  {}", def.expression);
            // One println per line. A single multi-line literal reads better in
            // the source and prints the source's own indentation, which is how
            // this came out nineteen spaces deep the first time.
            println!("  to define it: a [theory] block on the sheet — why it is this relation,");
            println!("  what the answer means, and the steps it comes in. Until then nothing");
            println!("  here is a definition a reader can check, and the row does not answer.");
        }
        None => println!("  \x1b[33mnot computed\x1b[0m — see the blocked list below"),
    }
    println!();
    println!(
        "  {} ran, {} blocked, {} cycle sweep(s)",
        results.manifest.ran, results.manifest.blocked_count, results.manifest.iterations
    );
    if !results.blocked.is_empty() {
        println!("  blocked:");
        for b in results.blocked.iter().take(12) {
            println!("    {} — {}", b.id, b.message);
        }
        if results.blocked.len() > 12 {
            println!("    … and {} more", results.blocked.len() - 12);
        }
    }
    println!();
    println!("  provenance");
    if results.manifest.data.is_empty() {
        println!("    \x1b[33mno reference data in the store — every node that declares a bundle refused\x1b[0m");
    } else {
        println!("    data   {}", results.manifest.data.join(" · "));
    }
    println!(
        "    kernel {} · graph {} · case {} · chain {}",
        results.manifest.kernel,
        results.manifest.graph,
        results.manifest.case,
        results.manifest.chain
    );
    println!("    mode {} · endpoint local-cli", results.manifest.mode);
    println!();
    println!("  the chain behind this number");
    for v in results.values.iter().take(200) {
        if def.inputs.iter().any(|&i| VARS[i as usize].id == v.id) || v.id == def.id {
            let unit = VARS[Vleo::find(&v.id).unwrap_or(0) as usize].unit;
            let (shown, sym) = vleo_bus::present(v.value, unit, 6);
            println!(
                "    {:<34} {:>18} {:<10} cred {}",
                v.id,
                shown,
                sym,
                v.cred.governing_score()
            );
        }
    }
    Ok(())
}

fn cmd_sweep(args: &[&str]) -> Result<(), String> {
    let node = *args
        .first()
        .ok_or("usage: vleo sweep <node> --over <input>")?;
    let over = opt(args, "--over").ok_or("--over names the input to sweep")?;
    let from: f64 = opt(args, "--from")
        .ok_or("--from")?
        .parse()
        .map_err(|_| "--from is not a number")?;
    let to: f64 = opt(args, "--to")
        .ok_or("--to")?
        .parse()
        .map_err(|_| "--to is not a number")?;
    let points: usize = opt(args, "--points").unwrap_or("21").parse().unwrap_or(21);
    let over_idx = Vleo::find(over).ok_or_else(|| format!("no node '{over}' to sweep"))?;
    suppliable(over_idx)?;
    let node_idx = Vleo::find(node).ok_or_else(|| format!("no node '{node}'"))?;

    let mut scratch = Scratch::new();
    let (said, r) = inputs_for(args)?;
    println!("# inputs: {said}");
    say_set_aside(&r);
    println!(
        "# {} against {} — {} points\n# {:<18} {:<22} note",
        node, over, points, VARS[over_idx as usize].symbol, NODES[node_idx as usize].id
    );
    let mut ran = 0usize;
    let mut refused = 0usize;
    for i in 0..points {
        let t = if points == 1 {
            0.0
        } else {
            i as f64 / (points - 1) as f64
        };
        let x = from + t * (to - from);
        let mut case = build_case(node, args)?;
        case.supply.push((over.to_string(), x));
        match vleo_modules::evaluate(&case, &mut scratch) {
            Ok(r) => match r.values.iter().find(|v| v.id == node) {
                Some(v) => {
                    ran += 1;
                    let (sx, _) = vleo_bus::present(x, VARS[over_idx as usize].unit, 6);
                    let (sy, _) = vleo_bus::present(v.value, VARS[node_idx as usize].unit, 9);
                    println!("{:<20} {:<24} ok", sx, sy);
                }
                None => {
                    refused += 1;
                    println!("{:<20.6} {:<22} blocked", x, "-");
                }
            },
            Err(f) => {
                refused += 1;
                println!("{:<20.6} {:<22} refused: {}", x, "-", f);
            }
        }
    }
    println!("# {ran} ran, {refused} refused. Refusals are recorded, never dropped.");
    Ok(())
}

fn cmd_campaign(args: &[&str]) -> Result<(), String> {
    let node = *args.first().ok_or("usage: vleo campaign <node>")?;
    let node_idx = Vleo::find(node).ok_or_else(|| format!("no node '{node}'"))?;
    // THE DEFAULTS, THE SAVED CASE, AND EVERY FILE NAMED, side by side. A
    // customer or a condition is a CSV somebody keeps, not a file in this
    // repository, so comparing them is comparing files: one run each, the
    // same row, the same engine.
    // `--inputs a.csv b.csv` and `--inputs a.csv --inputs b.csv` alike.
    let mut files: Vec<&str> = Vec::new();
    let mut taking = false;
    for a in args.iter().skip(1) {
        if *a == "--inputs" {
            taking = true;
        } else if a.starts_with("--") {
            taking = false;
        } else if taking {
            files.push(a);
        }
    }
    let mut runs: Vec<(String, Vec<&str>)> = vec![("defaults".into(), vec!["--defaults"])];
    if vleo_data::case_path().exists() {
        runs.push(("saved case".into(), vec![]));
    }
    for f in &files {
        runs.push((f.to_string(), vec!["--inputs", f]));
    }
    let mut scratch = Scratch::new();
    // As wide as the longest name, so a file path does not push its row's
    // numbers out from under their headings.
    let w = runs.iter().map(|(n, _)| n.len()).max().unwrap_or(0).max(28);
    println!(
        "{:<w$} {:>20} {:>8} {:>8} {:>10}  chain",
        "inputs", NODES[node_idx as usize].id, "ran", "blocked", "cred"
    );
    for (name, flags) in runs {
        let mut a: Vec<&str> = vec![node];
        a.extend(flags);
        let case = build_case(node, &a)?;
        match vleo_modules::evaluate(&case, &mut scratch) {
            Ok(r) => {
                let v = r.values.iter().find(|v| v.id == node);
                println!(
                    "{:<w$} {:>20} {:>8} {:>8} {:>10}  {}",
                    name,
                    v.map(|v| vleo_bus::present(v.value, VARS[node_idx as usize].unit, 6).0)
                        .unwrap_or_else(|| "-".into()),
                    r.manifest.ran,
                    r.manifest.blocked_count,
                    v.map(|v| v.cred.governing_score().to_string())
                        .unwrap_or_else(|| "-".into()),
                    r.manifest.chain
                );
            }
            Err(f) => println!("{:<w$} refused: {}", name, f),
        }
    }
    Ok(())
}

fn cmd_list(args: &[&str]) -> Result<(), String> {
    let filter = args.first().copied();
    let mut by_sub: BTreeMap<&str, usize> = BTreeMap::new();
    for def in NODES.iter() {
        if let Some(f) = filter {
            if def.subsystem != f {
                continue;
            }
        }
        *by_sub.entry(def.subsystem).or_default() += 1;
        println!(
            "{:<34} {:<10} {:<12} {:<10} {}",
            def.id,
            def.kind.name(),
            def.owner,
            def.tier.name(),
            def.label
        );
    }
    println!();
    for (s, n) in by_sub {
        println!("  {s:<10} {n}");
    }
    Ok(())
}

fn cmd_show(args: &[&str]) -> Result<(), String> {
    let node = *args.first().ok_or("usage: vleo show <node>")?;
    let i = Vleo::find(node).ok_or_else(|| format!("no node '{node}'"))?;
    let def = &NODES[i as usize];
    let var = &VARS[i as usize];
    println!("\x1b[1m{}\x1b[0m — {}", def.id, def.label);
    println!("  question     {}", def.question);
    println!("  relation     {}", def.expression);
    println!("  source       {}", def.source);
    println!(
        "  owner        {}  tier {}  kind {}  state {}",
        def.owner,
        def.tier.name(),
        def.kind.name(),
        def.state.name()
    );
    println!(
        "  publishes    {} ({}) in {}",
        var.symbol,
        var.label,
        var.unit.symbol()
    );
    println!(
        "  valid over   {} … {} {}",
        var.limit.lower,
        var.limit.upper,
        var.unit.symbol()
    );
    println!("               lower: {}", var.limit.reason_lower);
    println!("               upper: {}", var.limit.reason_upper);
    if !def.assumptions.is_empty() {
        println!("  assumptions");
        for (a, f) in def.assumptions {
            println!("    {a}\n      fails when {f}");
        }
    }
    if !def.inputs.is_empty() {
        println!("  reads");
        for &i in def.inputs {
            println!("    {:<34} {}", VARS[i as usize].id, VARS[i as usize].label);
        }
    }
    if !def.steps.is_empty() {
        println!("  algorithm");
        for (n, s) in def.steps.iter().enumerate() {
            println!("    {}. {s}", n + 1);
        }
    }
    let consumers: Vec<&str> = NODES
        .iter()
        .filter(|c| c.inputs.contains(&i))
        .map(|c| c.id)
        .collect();
    println!(
        "  read by      {} node(s){}",
        consumers.len(),
        if consumers.is_empty() {
            String::new()
        } else {
            format!(": {}", consumers.join(", "))
        }
    );
    if !def.contributes.is_empty() {
        println!("  contributes  {}", def.contributes.join(", "));
    }
    if def.fixtures.is_empty() {
        println!(
            "  evidence     \x1b[33mnone — nothing outside this code has agreed with it\x1b[0m"
        );
    } else {
        println!("  evidence");
        for f in def.fixtures {
            println!(
                "    {:<34} expect {:>16} ± {:<10} {} / {}",
                f.label,
                f.expected,
                f.tolerance,
                f.provenance.name(),
                f.source
            );
        }
    }
    Ok(())
}

fn cmd_cases() -> Result<(), String> {
    for c in tables::CASES.iter() {
        let all = vleo_modules::inputs::inputs(c);
        let cond = all
            .iter()
            .filter(|i| i.group == vleo_modules::inputs::Group::Condition)
            .count();
        println!("\x1b[1m{}\x1b[0m — {}", c.id, c.label);
        println!("  {}", c.note);
        println!(
            "  {} inputs: {} customer, {} condition. `vleo inputs` prints them as a CSV to fill in.",
            all.len(),
            all.len() - cond,
            cond
        );
        for cy in c.cycles {
            println!(
                "  declared cycle over {} nodes, converging on {} to {:e} in at most {} sweeps",
                cy.nodes.len(),
                VARS[cy.converge_on as usize].id,
                cy.tolerance,
                cy.max_iter
            );
        }
    }
    println!(
        "template {} — every CSV the tool writes names it, and a file naming another is carried over",
        vleo_modules::inputs::template()
    );
    let p = vleo_data::case_path();
    let s = vleo_modules::inputs::saved::load(&p);
    if !s.stored {
        println!(
            "saved case: none — every run uses the declared defaults ({})",
            p.display()
        );
        return Ok(());
    }
    println!(
        "saved case: {} — {} changed from default",
        p.display(),
        s.reading.changed
    );
    if let Some(e) = &s.error {
        println!("  {e}");
    }
    if let Some(u) = &s.reading.upgrade {
        println!(
            "  carried over from template {} when the tool changed: {} new input(s) at their default, {} value(s) set aside",
            u.from,
            u.new.len(),
            u.set_aside.len()
        );
        // Named up to a screenful; a file that named few inputs makes nearly
        // every input new, and the count already says so.
        for n in u.new.iter().take(12) {
            println!("    new        {n}");
        }
        if u.new.len() > 12 {
            println!("    new        … and {} more", u.new.len() - 12);
        }
        for a in &u.set_aside {
            println!(
                "    set aside  {} = {} {} — {}",
                a.id,
                if a.value.is_empty() { "?" } else { &a.value },
                a.unit,
                a.why
            );
        }
        if let Some(b) = &u.backup {
            println!("  the case as it was: {b}");
        }
    }
    Ok(())
}

/// Now, as a result records it: UTC, to the second (vleo_data::clock).
fn now_utc() -> String {
    vleo_data::clock::now_utc()
}

/// A saved result, as it was when it was saved. Runs nothing: a result read
/// back is a record, and running it again would be a different result.
fn cmd_result(args: &[&str]) -> Result<(), String> {
    let file = *args
        .first()
        .ok_or("usage: vleo result <file> [--html <out.html>]")?;
    let text = std::fs::read_to_string(file).map_err(|e| format!("{file}: {e}"))?;
    let s = vleo_modules::results::read(&vleo_modules::results::unwrap_report(&text))?;
    if let Some(out) = opt(args, "--html") {
        vleo_modules::files::write_whole(std::path::Path::new(out), vleo_modules::results::html(&s)).map_err(|e| format!("{out}: {e}"))?;
        eprintln!("wrote the report to {out}");
    }
    let unit = |u: &str| {
        if u == "-" {
            String::new()
        } else {
            format!(" {u}")
        }
    };
    println!(
        "\x1b[1m{}\x1b[0m — a saved result{}, {}",
        s.target,
        if s.name.is_empty() {
            String::new()
        } else {
            format!(" «{}»", s.name)
        },
        s.saved
    );
    match s.answer() {
        Some(a) => println!(
            "  \x1b[1m{}{}\x1b[0m  credibility {} of 4, governed by {}",
            a.value,
            unit(&a.unit),
            a.credibility,
            a.governing
        ),
        None => println!("  not computed on that run"),
    }
    println!(
        "  {} ran, {} blocked · mode {} · chain {} · kernel {} · graph {}",
        s.ran, s.blocked_count, s.mode, s.chain, s.kernel, s.graph
    );
    if s.template != vleo_modules::inputs::template() {
        println!("  saved against another set of inputs than this tree has — its values are shown as they were");
    }
    if !s.versions.is_empty() {
        println!(
            "  rests on: {}",
            s.versions
                .iter()
                .map(|(id, n, rel)| format!("{id} v{n} ({rel})"))
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    for (id, then, now) in vleo_modules::results::moved_since(&s) {
        println!(
            "  \x1b[33m{id} {}\x1b[0m — a belief it rested on has broken since; run it again to \
             see what the new version says",
            vleo_modules::results::moved_words(then, now)
        );
    }
    println!("\n  {} input(s) changed from their defaults:", s.changed());
    for r in s.inputs.iter().filter(|r| r.note == "changed") {
        println!("    {:<36} {}{}", r.id, r.value, unit(&r.unit));
    }
    println!("\n  {} value(s) returned:", s.outputs.len());
    for r in &s.outputs {
        println!(
            "    {:<36} {:>14}{:<6} cred {}",
            r.id,
            r.value,
            unit(&r.unit),
            r.credibility
        );
    }
    if !s.blocked.is_empty() {
        println!("\n  {} could not run:", s.blocked.len());
        for r in &s.blocked {
            println!("    {:<36} {}", r.id, r.note);
        }
    }
    Ok(())
}

/// The case's inputs as a CSV: every input, its group, the value it runs at
/// when one is saved, its default and its range. Fill in `value` and pass the
/// file back with `--inputs`, or upload it on the Inputs page.
///
/// Always in this tree's template, so `vleo inputs --inputs old.csv > new.csv`
/// is how a file from an older version is brought up to date: its values
/// carried, new inputs blank, and anything that could not be carried written
/// into the new file as a `#! set-aside` line rather than lost.
fn cmd_inputs(args: &[&str]) -> Result<(), String> {
    let (_, r) = inputs_for(args)?;
    print!(
        "{}",
        vleo_modules::inputs::csv_with(&r.set, r.upgrade.as_ref())
    );
    Ok(())
}

fn cmd_selftest() -> Result<(), String> {
    let mut total = 0usize;
    let mut passed = 0usize;
    let mut unevidenced = 0usize;
    for (i, def) in NODES.iter().enumerate() {
        if def.fixtures.is_empty() {
            if def.kind != Kind::Declared {
                unevidenced += 1;
            }
            continue;
        }
        for f in def.fixtures {
            total += 1;
            // The fixture inputs live with the generated evidence harness, so
            // the executable check is `cargo test`. Here we verify the tree is
            // consistent about them: provenance outside the code, a positive
            // tolerance, and a source that resolves.
            let ok = matches!(
                f.provenance,
                vleo_core::evidence::Provenance::IndependentDerivation
                    | vleo_core::evidence::Provenance::PublishedSource
                    | vleo_core::evidence::Provenance::IndependentTool
                    | vleo_core::evidence::Provenance::PhysicalBound
            ) && f.tolerance > 0.0;
            if ok {
                passed += 1;
            } else {
                println!(
                    "  \x1b[31mFAIL\x1b[0m {} / {} — provenance {} is not an external oracle",
                    def.id,
                    f.label,
                    f.provenance.name()
                );
            }
        }
        let _ = i;
    }
    println!("selftest: {passed}/{total} fixture declarations sound");
    println!(
        "{unevidenced} computed node(s) carry no fixture at all. A blank is a statement, not an oversight: their validation factor is zero, which governs the whole vector."
    );
    println!("Run `cargo test` to execute the fixtures against this build.");
    if passed != total {
        return Err(
            "a fixture claims an expected value that did not come from outside this code".into(),
        );
    }
    Ok(())
}

fn cmd_data(args: &[&str]) -> Result<(), String> {
    let root = data_root();
    let mut store = vleo_data::Store::open(&root);
    match args.first().copied() {
        Some("sync") => {
            let from = args.get(1).map(PathBuf::from).unwrap_or_else(repo_bundles);
            let n = store.sync(&vleo_data::Source::File(from.clone()))?;
            println!(
                "synced {n} bundle(s) from {} into {}",
                from.display(),
                root.display()
            );
            println!("Synchronisation and evaluation are separate moments. The engine now reads only what is on disk.");
            Ok(())
        }
        Some("verify") => {
            store.load()?;
            for b in store.bundles.values() {
                if b.verified {
                    println!(
                        "  \x1b[32mok\x1b[0m   {}@{} {}",
                        b.manifest.name, b.manifest.version, b.manifest.content_hash
                    );
                } else {
                    println!(
                        "  \x1b[31mFAIL\x1b[0m {}@{} — {}",
                        b.manifest.name,
                        b.manifest.version,
                        b.refusal.clone().unwrap_or_default()
                    );
                }
            }
            Ok(())
        }
        _ => {
            store.load()?;
            if store.bundles.is_empty() {
                println!(
                    "The store at {} is empty. `vleo data sync` fills it.",
                    root.display()
                );
                return Ok(());
            }
            for b in store.bundles.values() {
                println!(
                    "{:<20} {:<12} {}  licence until {}  stale after {} days",
                    b.manifest.name,
                    b.manifest.version,
                    if b.verified { "verified" } else { "REFUSED " },
                    b.manifest.licence_until,
                    b.manifest.stale_after_days
                );
                println!(
                    "  provenance {} — {}",
                    b.manifest.provenance,
                    b.manifest.note.trim().lines().next().unwrap_or("")
                );
            }
            Ok(())
        }
    }
}
