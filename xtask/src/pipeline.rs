//! The pipeline: every command this program has, in one table, and one way
//! every command that writes reports what it is doing.
//!
//! A command is a step in a node's journey — a form filled, taken, checked,
//! applied, published, built, gated, previewed and released. For each one the
//! table says where in that journey it sits, what it reads, what it writes,
//! what it checks, how to undo it, and which function in which file does it.
//! `explain`, `--dry-run` and `docs/PIPELINE.md` are read from the table, and a
//! test fails when the table and `help`, or the table and the code, disagree —
//! so the description of a command cannot drift from the command.
//!
//! A command that writes runs as numbered steps. Each step is printed as it
//! starts, with what it found when it ends; a step that stops says why, what
//! state the files are in, the exact command to try again, and where its code
//! is — the step's name is a string in that file, so searching for the printed
//! name finds the line. Every such run leaves a trace in `target/xtask-trace/`,
//! and `xtask trace` shows the last one.

use super::*;
use std::io::Write as _;

/// Where a command sits in a node's journey, in order.
pub(crate) const STAGES: &[(&str, &str)] = &[
    ("form", "a node engineer fills a node's form"),
    ("take", "the developer puts it on its own branch"),
    ("check", "what it would change, before anything is written"),
    (
        "apply",
        "the form written into the sheet — all of it or none",
    ),
    (
        "publish",
        "a filled row's code generated, and its holes written",
    ),
    (
        "build",
        "a node built from its method, and its tests shown to test",
    ),
    (
        "gate",
        "the checks every change passes, and what they generate",
    ),
    (
        "preview",
        "the node engineer tries the build and approves it",
    ),
    ("release", "the stamped release everyone gets"),
    (
        "read",
        "reports: what exists, what is open, why a node is what it is",
    ),
    ("setup", "once per person per clone"),
];

/// What `--dry-run` does for a command.
pub(crate) enum Dry {
    /// It only reads; it runs as it is.
    Reads,
    /// The same command with these arguments checks everything and writes
    /// nothing: `--dry-run` runs that.
    Check(&'static str),
    /// No check mode of its own: `--dry-run` prints the plan — the steps, what
    /// would be written and how to undo it — and touches nothing.
    Plan,
}

/// One command, as the table describes it.
pub(crate) struct Cmd {
    pub name: &'static str,
    pub stage: &'static str,
    pub reads: &'static str,
    /// Empty when it writes nothing.
    pub writes: &'static str,
    pub checks: &'static str,
    pub undo: &'static str,
    /// The file and the function that does it.
    pub code: (&'static str, &'static str),
    /// The steps it prints, by the names it prints them — each a string in
    /// `code.0`.
    pub steps: &'static [&'static str],
    pub dry: Dry,
}

const NOTHING: &str = "";
const READS_ONLY: &str = "nothing to undo: it writes nothing";
const GIT_UNDO: &str =
    "`git restore <files>` (or `git checkout -- .`) before committing; `git revert` after";

/// Every command, in the order of a node's journey.
pub(crate) const PIPELINE: &[Cmd] = &[
    Cmd {
        name: "form",
        stage: "form",
        reads: "the node's sheet, the rows it could read, web/method.wasm.gz",
        writes: "one HTML file: <node>.node-form.html, or --out",
        checks: "that the node exists",
        undo: "delete the file",
        code: ("xtask/src/forms.rs", "cmd_form"),
        steps: &[],
        dry: Dry::Plan,
    },
    Cmd {
        name: "lesson",
        stage: "apply",
        reads: "the row's sheet and lesson.toml, the tree's rows, web/method.wasm.gz; a filled lesson form",
        writes: "form: <node>.lesson-form.html, or --out; apply: lesson.toml beside the row's node.toml",
        checks: "the lesson, as the gate checks it: every key known, every claim tagged, no markup, every row a widget names real; apply gates the row",
        undo: "`git restore` (or delete) the row's lesson.toml; a refused apply puts it back itself",
        code: ("xtask/src/forms.rs", "cmd_lesson"),
        steps: &["check the lesson", "write it beside the row", "gate the row"],
        dry: Dry::Check("lesson check <file>: every reason it would be refused, nothing written"),
    },
    Cmd {
        name: "readers",
        stage: "release",
        reads: "the tree, generated/fragments, every lesson.toml, web/page.html, web/js, web/app.css and web/fonts, crates/vleo-kernel-wasm",
        writes: "the readers' docs folder: target/readers, or --out — rebuilt whole; nothing committed",
        checks: "every lesson passes its check; every link and asset a page names, and every font its stylesheet names, is in the folder; tools/readers_check.py then opens it in a browser",
        undo: "delete the folder",
        code: ("xtask/src/readers.rs", "cmd_readers"),
        steps: &["build the engine for the browser", "bundle the page script", "write the pages", "check every page has what it links"],
        dry: Dry::Plan,
    },
    Cmd {
        name: "take",
        stage: "take",
        reads: "the filled form, the `maintainer` branch, the tree",
        writes: "a branch form/<author>/<node>: the applied sheet, regenerated files, today's answers recorded again, a commit, a push — or <form>.returned.txt",
        checks: "the form, as intake does; the gate; cargo test; a node with a method built from it",
        undo: "delete the branch (`git branch -D form/<author>/<node>`, and on the remote); nothing on `maintainer` changes",
        code: ("xtask/src/flow.rs", "cmd_take"),
        steps: &[
            "the branch",
            "apply the form",
            "regenerate",
            "gate",
            "build the node from its method",
            "today's answers, recorded again",
            "tests (cargo test --workspace)",
            "commit and push",
        ],
        dry: Dry::Plan,
    },
    Cmd {
        name: "intake",
        stage: "check",
        reads: "the filled form, the node's sheet, every row an interface names",
        writes: "with --apply: node.toml (or a new node's folder), every generated file, CODEOWNERS for a new node",
        checks: "every field against the sheet; every interface; a conflict with a change made since; a relation an assistant supplied; the whole tree's gate",
        undo: "a refused apply is put back by intake itself; an applied one: `git restore` the node's folder, or `git revert`",
        code: ("xtask/src/forms.rs", "cmd_intake"),
        steps: &[
            "read the form",
            "check every change",
            "build the new node",
            "apply to the sheet",
        ],
        dry: Dry::Check("intake without --apply: the same check, nothing written"),
    },
    Cmd {
        name: "group-intake",
        stage: "check",
        reads: "a group's release written out by `node tools/group_db.mjs --unpack`, its RELEASE.toml, and each of its nodes' sheets",
        writes: "with --apply: each computed node's node.toml (its method and cases) and every generated file",
        checks: "every file against the seal's fingerprint; then each node as its form: a conflict with a change made since, a method or results an assistant supplied (a transcription without its source and a person who checked it counts as one), the de-risking record, the whole tree's gate",
        undo: "a refused apply is put back by the transaction itself; an applied one: `git restore` the node's folder, or `git revert`",
        code: ("xtask/src/group_intake.rs", "cmd_group_intake"),
        steps: &[],
        dry: Dry::Check("group-intake without --apply: the same check, nothing written"),
    },
    Cmd {
        name: "group-build",
        stage: "build",
        reads: "an unpacked sealed release and the design it was taken into",
        writes: "what build-node writes for each computed node: its kernel translation and generated files; baseline/today.csv, today's answers recorded again",
        checks: "the seal; that the design's method is the release's; then build-node on each — the node engineer's cases, the tests proved to test, the interface",
        undo: GIT_UNDO,
        code: ("xtask/src/group_test.rs", "cmd_group_build"),
        steps: &[],
        dry: Dry::Plan,
    },
    Cmd {
        name: "group-test",
        stage: "check",
        reads: "an unpacked sealed release, the design, and the engine built from it",
        writes: "target/group/<group>-<version>/group-test.csv, the report, and the runs it rests on",
        checks: "the design holds the release's cases; each node's tests pass; the group gives results/group.csv through the engine; both ends of every declared range answer or refuse by name",
        undo: READS_ONLY,
        code: ("xtask/src/group_test.rs", "cmd_group_test"),
        steps: &[],
        dry: Dry::Reads,
    },
    Cmd {
        name: "group-deliver",
        stage: "publish",
        reads: "an unpacked sealed release, its passing group-test report, the release-built programs",
        writes: "dist/vleo-<version>-<group>-<v>-test/: the kit, DELIVERY.toml, DELIVERY.md and the report",
        checks: "the seal; that group-test passed; that the checkout is committed, unless --uncommitted",
        undo: "delete the folder",
        code: ("xtask/src/group_test.rs", "cmd_group_deliver"),
        steps: &[],
        dry: Dry::Plan,
    },
    Cmd {
        name: "group-accept",
        stage: "release",
        reads: "the group's answer, written by the group application; the branch group/<group>-<version>",
        writes: "acceptances/<group>-<version>.toml on that branch, committed and pushed",
        checks: "the verdict is accepted and says what was tried; a person's name; the delivery record's hash, with --delivery; the accepted commit is this branch, unchanged since but for these records",
        undo: "`git revert` the acceptance commit, or delete the file before pushing",
        code: ("xtask/src/flow.rs", "cmd_group_accept"),
        steps: &[],
        dry: Dry::Plan,
    },
    Cmd {
        name: "new",
        stage: "apply",
        reads: "the sibling's sheet",
        writes: "a new node folder, its sheet cloned with what must be re-decided blanked",
        checks: "that the id is free and the sibling exists",
        undo: "delete the new folder",
        code: ("xtask/src/forms.rs", "cmd_new"),
        steps: &[],
        dry: Dry::Plan,
    },
    Cmd {
        name: "declare",
        stage: "publish",
        reads: "the node's sheet",
        writes: "with --source: the sheet's source line",
        checks: "which completion questions are still open",
        undo: GIT_UNDO,
        code: ("xtask/src/report.rs", "cmd_declare"),
        steps: &[],
        dry: Dry::Plan,
    },
    Cmd {
        name: "publish",
        stage: "publish",
        reads: "the node's sheet",
        writes: "node.toml's state, and the node's generated model, contract, evidence, module, page and metadata",
        checks: "every completion question answered; the whole tree's gate",
        undo: GIT_UNDO,
        code: ("xtask/src/main.rs", "cmd_publish"),
        steps: &["read the sheet", "publish, generate and gate"],
        dry: Dry::Plan,
    },
    Cmd {
        name: "fill",
        stage: "publish",
        reads: "the node's sheet and model.rs, the body",
        writes: "one HOLE block in model.rs; with --by: fills.toml",
        checks: "no HOLE marker, fault or early return in the body; portable maths only; a significant node's body is attributed; the body landed",
        undo: "`git restore` the node's model.rs and fills.toml",
        code: ("xtask/src/fills.rs", "cmd_fill"),
        steps: &["read the body", "check the body", "splice into the hole"],
        dry: Dry::Plan,
    },
    Cmd {
        name: "confirm",
        stage: "publish",
        reads: "the sheets",
        writes: "with <node> --by: that node's confirmation",
        checks: "that the relation and its source are printed first",
        undo: GIT_UNDO,
        code: ("xtask/src/fills.rs", "cmd_confirm"),
        steps: &[],
        dry: Dry::Plan,
    },
    Cmd {
        name: "method",
        stage: "build",
        reads: "the node's method and its node engineer's cases",
        writes: NOTHING,
        checks: "the method parses and its units agree; every case comes out as the node engineer said",
        undo: READS_ONLY,
        code: ("xtask/src/method.rs", "cmd_method"),
        steps: &[],
        dry: Dry::Reads,
    },
    Cmd {
        name: "build-node",
        stage: "build",
        reads: "the node's method, cases and node engineer's code",
        writes: "crates/vleo-core/src/physics/methods/<node>.rs and the node's generated files",
        checks: "the method on its cases; the node's tests; the node engineer's code rerun; a mutation the tests must catch; the tree assembles",
        undo: GIT_UNDO,
        code: ("xtask/src/method.rs", "cmd_build_node"),
        steps: &[
            "the method, against the node engineer's cases",
            "translate the method into the kernel, and regenerate the node",
            "the node's tests: the node engineer's cases, and the translation against the method",
            "the node engineer's own code, run again on their cases",
            "the tests really test: the answer is moved and the tests must notice",
            "only now, the interface: the node in the tree",
        ],
        dry: Dry::Plan,
    },
    Cmd {
        name: "rerun",
        stage: "build",
        reads: "the node engineer's code and their cases",
        writes: NOTHING,
        checks: "the node engineer's own code still gives their cases",
        undo: READS_ONLY,
        code: ("xtask/src/method.rs", "cmd_rerun"),
        steps: &[],
        dry: Dry::Reads,
    },
    Cmd {
        name: "method-wasm",
        stage: "build",
        reads: "crates/vleo-method-wasm and the method language's sources",
        writes: "web/method.wasm.gz and its stamp",
        checks: "with --check: whether the committed checker is built from the sources as they are",
        undo: "`git restore web/method.wasm.gz web/method.wasm.stamp`",
        code: ("xtask/src/method.rs", "cmd_method_wasm"),
        steps: &[],
        dry: Dry::Check("method-wasm --check: whether it is current, nothing built"),
    },
    Cmd {
        name: "group-app",
        stage: "gate",
        reads: "groups/SPEC.toml, groups/schema.sql, web/pages/group.html, node-app.html and group.css, web/app.css and its fonts, web/vendor/sqlite, web/method.wasm.gz, web/js",
        writes: "web/group.html, web/node.html, docs/GROUP_FOLDER.md and groups/skill/vleo-group-folder/SKILL.md",
        checks: "the vendored SQLite against the hashes web/vendor/sqlite/SOURCE.toml records; with --check, whether the committed four are built from their sources as they are",
        undo: "`git restore web/group.html web/node.html docs/GROUP_FOLDER.md groups/skill/vleo-group-folder/SKILL.md`",
        code: ("xtask/src/group.rs", "cmd_group_app"),
        steps: &[],
        dry: Dry::Check("group-app --check: whether they are current, nothing written"),
    },
    Cmd {
        name: "group-export",
        stage: "form",
        reads: "every sheet, fixtures.toml and source of one group in the tree",
        writes: "the group's folder in the pattern of groups/SPEC.toml, under target/groups/<group> or --out",
        checks: "nothing: what the tree lacks is left empty, and the group application lists it",
        undo: "delete the folder it wrote; nothing in the repository changes",
        code: ("xtask/src/group.rs", "cmd_group_export"),
        steps: &[],
        dry: Dry::Reads,
    },
    Cmd {
        name: "differential",
        stage: "build",
        reads: "every recorded body for the node's holes",
        writes: NOTHING,
        checks: "the node against each other recorded body",
        undo: READS_ONLY,
        code: ("xtask/src/fills.rs", "cmd_differential"),
        steps: &[],
        dry: Dry::Reads,
    },
    Cmd {
        name: "mutate",
        stage: "build",
        reads: "the node's code and tests",
        writes: "the node's code, moved and always put back",
        checks: "that the node's own tests notice a moved answer",
        undo: "nothing to undo: the moved code is put back before it returns",
        code: ("xtask/src/mutate.rs", "cmd_mutate"),
        steps: &[],
        dry: Dry::Plan,
    },
    Cmd {
        name: "docs",
        stage: "gate",
        reads: "every sheet",
        writes: "each node's generated files — model, contract, module, evidence, metadata — and docs/PSEUDOCODE.md; a page.html left in a node folder from before is removed",
        checks: "that each sheet generates",
        undo: GIT_UNDO,
        code: ("xtask/src/main.rs", "cmd_docs"),
        steps: &[],
        dry: Dry::Plan,
    },
    Cmd {
        name: "assemble",
        stage: "gate",
        reads: "every sheet",
        writes: "the index, the document and the graph tables",
        checks: "that the tree assembles",
        undo: GIT_UNDO,
        code: ("xtask/src/main.rs", "cmd_assemble"),
        steps: &[],
        dry: Dry::Plan,
    },
    Cmd {
        name: "gate",
        stage: "gate",
        reads: "every sheet and its generated files",
        writes: NOTHING,
        checks: "every node check, in order, and the assembly checks",
        undo: READS_ONLY,
        code: ("xtask/src/main.rs", "cmd_gate"),
        steps: &[],
        dry: Dry::Reads,
    },
    Cmd {
        name: "ready",
        stage: "gate",
        reads: "the node's sheet, gate and gap",
        writes: NOTHING,
        checks: "the gate, the gap pass, what criticality demands",
        undo: READS_ONLY,
        code: ("xtask/src/report.rs", "cmd_ready"),
        steps: &[],
        dry: Dry::Reads,
    },
    Cmd {
        name: "codeowners",
        stage: "gate",
        reads: "areas/teams.toml and each sheet's owner",
        writes: "CODEOWNERS",
        checks: "that every owner has a review team",
        undo: "`git restore CODEOWNERS`",
        code: ("xtask/src/release.rs", "cmd_codeowners"),
        steps: &[],
        dry: Dry::Plan,
    },
    Cmd {
        name: "variables",
        stage: "gate",
        reads: "every sheet",
        writes: "docs/VARIABLES.md",
        checks: NOTHING,
        undo: "`git restore docs/VARIABLES.md`",
        code: ("xtask/src/report.rs", "cmd_variables"),
        steps: &[],
        dry: Dry::Plan,
    },
    Cmd {
        name: "guides",
        stage: "gate",
        reads: "docs/manual.toml",
        writes: "docs/roles/user.html, maintainer.html and developer.html",
        checks: NOTHING,
        undo: "`git restore docs/roles/`",
        code: ("xtask/src/release.rs", "cmd_guides"),
        steps: &[],
        dry: Dry::Plan,
    },
    Cmd {
        name: "pipeline",
        stage: "gate",
        reads: "this table",
        writes: "docs/PIPELINE.md",
        checks: "with --check: that docs/PIPELINE.md is the table as it is",
        undo: "`git restore docs/PIPELINE.md`",
        code: ("xtask/src/pipeline.rs", "cmd_pipeline"),
        steps: &[],
        dry: Dry::Check("pipeline --check: whether docs/PIPELINE.md is current, nothing written"),
    },
    Cmd {
        name: "preview",
        stage: "preview",
        reads: "the current branch and its PREVIEW.json",
        writes: NOTHING,
        checks: "that this is a form branch",
        undo: READS_ONLY,
        code: ("xtask/src/flow.rs", "cmd_preview"),
        steps: &[],
        dry: Dry::Reads,
    },
    Cmd {
        name: "approve",
        stage: "preview",
        reads: "the node engineer's approval file and this branch",
        writes: "approvals/<author>--<node>.toml, a commit and a push",
        checks: "the approval is for the build of exactly what is here now",
        undo: "`git revert` the approval commit",
        code: ("xtask/src/flow.rs", "cmd_approve"),
        steps: &[],
        dry: Dry::Plan,
    },
    Cmd {
        name: "queue",
        stage: "preview",
        reads: "every form branch and every group branch",
        writes: NOTHING,
        checks: "where each one stands",
        undo: READS_ONLY,
        code: ("xtask/src/flow.rs", "cmd_queue"),
        steps: &[],
        dry: Dry::Reads,
    },
    Cmd {
        name: "ship",
        stage: "release",
        reads: "main, every sheet",
        writes: "a branch release/<version>: the narrative, the stamp, regenerated files, a commit, a push",
        checks: "the release rules; the gate; cargo test",
        undo: "delete the branch (`git branch -D release/<version>`, and on the remote)",
        code: ("xtask/src/flow.rs", "cmd_ship"),
        steps: &[
            "the branch",
            "the de-risking narrative",
            "stamp the release",
            "regenerate",
            "gate",
            "tests (cargo test --workspace)",
            "commit and push",
        ],
        dry: Dry::Plan,
    },
    Cmd {
        name: "release",
        stage: "release",
        reads: "every sheet's versions, Cargo.toml",
        writes: "every `next` version stamped, the workspace version, Cargo.lock files, regenerated docs and narrative",
        checks: "a release only moves forward; no version names a later one; with --check, that nothing is unstamped",
        undo: GIT_UNDO,
        code: ("xtask/src/release.rs", "cmd_release"),
        steps: &[
            "check the versions",
            "stamp the versions",
            "set the workspace version",
            "regenerate",
        ],
        dry: Dry::Check("release <version> --check: whether the release is stamped, nothing written"),
    },
    Cmd {
        name: "derisk",
        stage: "release",
        reads: "every sheet's [[version]] and [[risk]]",
        writes: "docs/DERISK_NARRATIVE.md and docs/derisking.csv",
        checks: NOTHING,
        undo: "`git restore docs/DERISK_NARRATIVE.md docs/derisking.csv`",
        code: ("xtask/src/release.rs", "cmd_derisk"),
        steps: &[],
        dry: Dry::Plan,
    },
    Cmd {
        name: "kit",
        stage: "release",
        reads: "the release-built programs and the files they read",
        writes: "dist/vleo-<version>/, which git ignores",
        checks: "that the programs are there",
        undo: "delete dist/vleo-<version>/",
        code: ("xtask/src/release.rs", "cmd_kit"),
        steps: &[],
        dry: Dry::Plan,
    },
    Cmd {
        name: "design",
        stage: "release",
        reads: "every file of the tree the loader reads: the node folders, the layers, the cases and the source list",
        writes: "target/design.vleo, or --out; with --check, nothing",
        checks: "that the tree loads; with --check, each file against its SHA-256 and the folders",
        undo: "delete the file it wrote; nothing in the repository changes",
        code: ("xtask/src/design.rs", "cmd_design"),
        steps: &[],
        dry: Dry::Plan,
    },
    Cmd {
        name: "bundle",
        stage: "release",
        reads: "a bundle's payload files",
        writes: "publish: the bundle's manifest",
        checks: "verify: every hash in bundles/",
        undo: "publication is irreversible by design: publish a new version",
        code: ("xtask/src/release.rs", "cmd_bundle"),
        steps: &[],
        dry: Dry::Plan,
    },
    Cmd {
        name: "status",
        stage: "read",
        reads: "every sheet",
        writes: NOTHING,
        checks: NOTHING,
        undo: READS_ONLY,
        code: ("xtask/src/report.rs", "cmd_status"),
        steps: &[],
        dry: Dry::Reads,
    },
    Cmd {
        name: "active",
        stage: "read",
        reads: "every sheet",
        writes: NOTHING,
        checks: NOTHING,
        undo: READS_ONLY,
        code: ("xtask/src/report.rs", "cmd_active"),
        steps: &[],
        dry: Dry::Reads,
    },
    Cmd {
        name: "catalogue",
        stage: "read",
        reads: "every sheet",
        writes: "with --csv, the catalogue as a table where you say; otherwise nothing",
        checks: NOTHING,
        undo: READS_ONLY,
        code: ("xtask/src/catalogue.rs", "cmd_catalogue"),
        steps: &[],
        dry: Dry::Reads,
    },
    Cmd {
        name: "impact",
        stage: "read",
        reads: "every sheet",
        writes: NOTHING,
        checks: NOTHING,
        undo: READS_ONLY,
        code: ("xtask/src/catalogue.rs", "cmd_impact"),
        steps: &[],
        dry: Dry::Reads,
    },
    Cmd {
        name: "reach",
        stage: "read",
        reads: "every sheet",
        writes: NOTHING,
        checks: NOTHING,
        undo: READS_ONLY,
        code: ("xtask/src/report.rs", "cmd_reach"),
        steps: &[],
        dry: Dry::Reads,
    },
    Cmd {
        name: "gap",
        stage: "read",
        reads: "every sheet",
        writes: NOTHING,
        checks: NOTHING,
        undo: READS_ONLY,
        code: ("xtask/src/report.rs", "cmd_gap"),
        steps: &[],
        dry: Dry::Reads,
    },
    Cmd {
        name: "graph",
        stage: "read",
        reads: "every sheet and the crate manifests",
        writes: NOTHING,
        checks: "the crate direction",
        undo: READS_ONLY,
        code: ("xtask/src/graph.rs", "cmd_graph"),
        steps: &[],
        dry: Dry::Reads,
    },
    Cmd {
        name: "migration",
        stage: "read",
        reads: "every sheet",
        writes: "with --forms: one node form per row, in the folder named",
        checks: NOTHING,
        undo: "delete the forms folder",
        code: ("xtask/src/method.rs", "cmd_migration"),
        steps: &[],
        dry: Dry::Plan,
    },
    Cmd {
        name: "explain",
        stage: "read",
        reads: "this table and the help",
        writes: NOTHING,
        checks: NOTHING,
        undo: READS_ONLY,
        code: ("xtask/src/pipeline.rs", "cmd_explain"),
        steps: &[],
        dry: Dry::Reads,
    },
    Cmd {
        name: "why",
        stage: "read",
        reads: "the node's sheet, its git history, approvals, fills and traces",
        writes: NOTHING,
        checks: "the node's gate, run now",
        undo: READS_ONLY,
        code: ("xtask/src/pipeline.rs", "cmd_why"),
        steps: &[],
        dry: Dry::Reads,
    },
    Cmd {
        name: "trace",
        stage: "read",
        reads: "target/xtask-trace/",
        writes: NOTHING,
        checks: NOTHING,
        undo: READS_ONLY,
        code: ("xtask/src/pipeline.rs", "cmd_trace"),
        steps: &[],
        dry: Dry::Reads,
    },
    Cmd {
        name: "setup",
        stage: "setup",
        reads: "this clone's git config",
        writes: "core.hooksPath in this clone's git config",
        checks: NOTHING,
        undo: "`git config --unset core.hooksPath`",
        code: ("xtask/src/hooks.rs", "cmd_setup"),
        steps: &[],
        dry: Dry::Plan,
    },
    Cmd {
        name: "help",
        stage: "setup",
        reads: NOTHING,
        writes: NOTHING,
        checks: NOTHING,
        undo: READS_ONLY,
        code: ("xtask/src/main.rs", "help"),
        steps: &[],
        dry: Dry::Reads,
    },
];

pub(crate) fn find(name: &str) -> Option<&'static Cmd> {
    PIPELINE.iter().find(|c| c.name == name)
}

/// A command's own paragraph of `help`: its usage lines and what it does.
pub(crate) fn help_entry(name: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut taking = false;
    for l in HELP.lines() {
        let top = l.starts_with("  ") && !l.starts_with("   ");
        if top {
            taking = l.split_whitespace().next() == Some(name);
        } else if !l.starts_with("   ") {
            taking = false;
        }
        if taking {
            out.push(l.trim().to_string());
        }
    }
    out
}

/// Split a help paragraph into its usage lines and its prose.
fn usage_and_what(name: &str) -> (Vec<String>, String) {
    let mut usage = Vec::new();
    let mut what = Vec::new();
    let top: Vec<String> = HELP
        .lines()
        .filter(|l| l.starts_with("  ") && !l.starts_with("   "))
        .filter(|l| l.split_whitespace().next() == Some(name))
        .map(|l| l.trim().to_string())
        .collect();
    for l in help_entry(name) {
        if top.contains(&l) {
            // `name args     prose` on one line: the usage, then the prose.
            match l.find("   ") {
                Some(k) => {
                    usage.push(l[..k].trim().to_string());
                    what.push(l[k..].trim().to_string());
                }
                None => usage.push(l),
            }
        } else if what.is_empty() && l.starts_with(name) {
            usage.push(l);
        } else {
            what.push(l);
        }
    }
    (usage, what.join(" "))
}

fn stage_no(key: &str) -> (usize, &'static str) {
    STAGES
        .iter()
        .position(|(k, _)| *k == key)
        .map(|i| (i + 1, STAGES[i].1))
        .unwrap_or((0, ""))
}

/// `explain [<command>]` — one command, or the whole journey.
pub(crate) fn cmd_explain(_root: &Path, args: &[&str]) -> Result<(), String> {
    let Some(name) = args.first() else {
        println!(
            "A node's journey, and the command at each step (`xtask explain <command>` for one):\n"
        );
        for (i, (key, what)) in STAGES.iter().enumerate() {
            let names: Vec<&str> = PIPELINE
                .iter()
                .filter(|c| c.stage == *key)
                .map(|c| c.name)
                .collect();
            println!("  {:>2} · {:<8} {what}", i + 1, key);
            println!("              {}", names.join(" · "));
        }
        println!("\nEvery command that writes takes --dry-run. docs/PIPELINE.md is this, in full.");
        return Ok(());
    };
    let c = find(name).ok_or_else(|| {
        format!("no command '{name}'. `cargo run -p xtask -- explain` lists them all")
    })?;
    let (usage, what) = usage_and_what(c.name);
    let (n, stage) = stage_no(c.stage);
    println!(
        "\x1b[1mxtask {}\x1b[0m — step {n} of {} · {}: {stage}",
        c.name,
        STAGES.len(),
        c.stage
    );
    for u in &usage {
        println!("  usage     {u}");
    }
    println!("  what      {what}");
    println!("  reads     {}", or_none(c.reads));
    println!(
        "  writes    {}",
        if c.writes.is_empty() {
            "nothing"
        } else {
            c.writes
        }
    );
    println!("  checks    {}", or_none(c.checks));
    println!("  undo      {}", c.undo);
    if !c.steps.is_empty() {
        println!("  steps     {}", numbered(c.steps));
    }
    println!("  dry run   {}", dry_words(c));
    println!("  code      {} — {}", c.code.0, c.code.1);
    Ok(())
}

fn or_none(s: &str) -> &str {
    if s.is_empty() {
        "—"
    } else {
        s
    }
}

fn numbered(steps: &[&str]) -> String {
    steps
        .iter()
        .enumerate()
        .map(|(i, s)| format!("{} {s}", i + 1))
        .collect::<Vec<_>>()
        .join(" · ")
}

fn dry_words(c: &Cmd) -> String {
    match &c.dry {
        Dry::Reads => "it only reads, so it runs as it is".to_string(),
        Dry::Check(how) => format!("--dry-run runs {how}"),
        Dry::Plan => {
            "--dry-run prints this plan — the steps, what it would write, how to undo it — and \
             touches nothing"
                .to_string()
        }
    }
}

/// `<command> … --dry-run`: what would happen, doing nothing.
pub(crate) fn dry_run(root: &Path, cmd: &str, rest: &[&str]) -> Result<(), String> {
    let Some(c) = find(cmd) else {
        return Err(format!("unknown command '{cmd}'. Try `cargo xtask help`."));
    };
    let args: Vec<&str> = rest.iter().copied().filter(|a| *a != "--dry-run").collect();
    match &c.dry {
        Dry::Reads => {
            println!("--dry-run: `{cmd}` only reads, so there is nothing to hold back — it runs as it is.\n");
            crate::dispatch(root, cmd, &args)
        }
        Dry::Check(how) => {
            let mut a: Vec<&str> = args
                .iter()
                .copied()
                .filter(|x| !matches!(*x, "--apply" | "--partial"))
                .collect();
            // intake and group-intake check by leaving out --apply
            if !matches!(cmd, "intake" | "group-intake") && !a.contains(&"--check") {
                a.push("--check");
            }
            println!(
                "--dry-run: running {how}.\n  cargo run -p xtask -- {cmd} {}\n",
                a.join(" ")
            );
            crate::dispatch(root, cmd, &a)
        }
        Dry::Plan => {
            cmd_explain(root, &[cmd])?;
            println!(
                "\n--dry-run: nothing was read or written. The command would be:\n  \
                 cargo run -p xtask -- {cmd} {}",
                args.join(" ")
            );
            Ok(())
        }
    }
}

// ---------------------------------------------------------------------------
// steps, and the trace every writing run leaves

/// What a stopped step says about the files, and how to try again.
pub(crate) struct OnStop {
    pub files: String,
    pub retry: String,
}

impl OnStop {
    pub(crate) fn new(files: impl Into<String>, retry: impl Into<String>) -> OnStop {
        OnStop {
            files: files.into(),
            retry: retry.into(),
        }
    }
}

/// A command that writes, run as numbered steps.
pub(crate) struct Run {
    cmd: &'static Cmd,
    total: usize,
    n: usize,
    trace: Option<PathBuf>,
    shown: String,
    started: std::time::Instant,
}

/// Where traces are kept, and how many.
const TRACES: &str = "target/xtask-trace";
const KEEP: usize = 50;

impl Run {
    /// Begin a run of `cmd` in `total` steps, and its trace.
    pub(crate) fn start(root: &Path, cmd: &str, args: &[&str], total: usize) -> Run {
        let c = find(cmd).expect("a command the pipeline table has");
        let dir = root.join(TRACES);
        let when = now_utc();
        let path = dir.join(format!("{}-{cmd}.log", when.replace(':', "-")));
        let trace = fs::create_dir_all(&dir)
            .ok()
            .and_then(|_| fs::File::create(&path).ok())
            .map(|mut f| {
                let head = git_head(root);
                let _ = writeln!(
                    f,
                    "xtask {cmd} {}\nstarted {when}\ngit {head}\nsteps {total}\n",
                    args.join(" ")
                );
                path.clone()
            });
        prune(&dir);
        let shown = trace
            .as_ref()
            .map(|p| p.strip_prefix(root).unwrap_or(p).display().to_string())
            .unwrap_or_else(|| "(no trace: target/ could not be written)".into());
        Run {
            cmd: c,
            total,
            n: 0,
            trace,
            shown,
            started: std::time::Instant::now(),
        }
    }

    fn log(&self, line: &str) {
        if let Some(p) = &self.trace {
            if let Ok(mut f) = fs::OpenOptions::new().append(true).open(p) {
                let _ = writeln!(f, "{line}");
            }
        }
    }

    /// Run one step. `f` returns its result and a few words on what it found.
    /// A step that stops prints why, the state of the files, how to retry and
    /// where its code is, and the run ends there.
    pub(crate) fn step<T>(
        &mut self,
        name: &str,
        on_stop: OnStop,
        f: impl FnOnce() -> Result<(T, String), String>,
    ) -> Result<T, String> {
        self.n += 1;
        let head = format!("{}/{} · {name}", self.n, self.total);
        println!("\n\x1b[1m{head}\x1b[0m");
        self.log(&format!("{} {head}", now_utc()));
        match f() {
            Ok((v, said)) => {
                if !said.is_empty() {
                    println!("      {said}");
                }
                self.log(&format!("    done  {said}"));
                Ok(v)
            }
            Err(why) => {
                let block = format!(
                    "    why       {why}\n    files     {}\n    retry     {}\n    code      {} — the step \"{name}\" in {}\n    trace     {}",
                    on_stop.files,
                    on_stop.retry,
                    self.cmd.code.0,
                    self.cmd.code.1,
                    self.shown
                );
                println!(
                    "\x1b[31m{}/{} · {name} — stopped\x1b[0m\n{block}",
                    self.n, self.total
                );
                self.log(&format!("    stopped\n{block}"));
                self.log(&format!(
                    "stopped after {:.1}s",
                    self.started.elapsed().as_secs_f64()
                ));
                Err(format!(
                    "{} stopped at step {}/{} ({name}): {why}",
                    self.cmd.name, self.n, self.total
                ))
            }
        }
    }

    /// How many steps there are, once the run knows — a form for a new node
    /// has one more than a form for an existing one.
    pub(crate) fn set_total(&mut self, total: usize) {
        self.total = total;
        self.log(&format!("steps {total}"));
    }

    /// A step the run skips, said rather than silently dropped.
    pub(crate) fn skip(&mut self, name: &str, why: &str) {
        self.n += 1;
        println!("\n{}/{} · {name} — skipped: {why}", self.n, self.total);
        self.log(&format!(
            "{} {}/{} · {name} — skipped: {why}",
            now_utc(),
            self.n,
            self.total
        ));
    }

    /// The run finished; say so once, and in the trace.
    pub(crate) fn done(self, summary: &str) {
        println!("\n{summary}");
        self.log(&format!(
            "finished after {:.1}s — {summary}",
            self.started.elapsed().as_secs_f64()
        ));
    }
}

fn now_utc() -> String {
    vleo_units::calendar::Civil::from_unix(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0),
    )
    .to_string()
}

fn git_head(root: &Path) -> String {
    std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .current_dir(root)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|| "(not a git checkout)".into())
}

/// Keep the newest traces; the rest go.
fn prune(dir: &Path) {
    let mut all = traces(dir);
    while all.len() > KEEP {
        let _ = fs::remove_file(all.remove(0));
    }
}

/// Every trace, oldest first. The names start with the time, so name order is
/// time order.
fn traces(dir: &Path) -> Vec<PathBuf> {
    let mut all: Vec<PathBuf> = fs::read_dir(dir)
        .map(|rd| {
            rd.flatten()
                .map(|e| e.path())
                .filter(|p| p.extension().is_some_and(|x| x == "log"))
                .collect()
        })
        .unwrap_or_default();
    all.sort();
    all
}

/// `trace [<command>] [--list]` — the last run's trace, or every trace kept.
pub(crate) fn cmd_trace(root: &Path, args: &[&str]) -> Result<(), String> {
    let dir = root.join(TRACES);
    let all = traces(&dir);
    if args.contains(&"--list") {
        if all.is_empty() {
            println!("no trace yet: every command that writes leaves one in {TRACES}/");
        }
        for p in all.iter().rev() {
            println!("{}", p.strip_prefix(root).unwrap_or(p).display());
        }
        return Ok(());
    }
    let only = args.iter().find(|a| !a.starts_with("--"));
    let last = all
        .iter()
        .rev()
        .find(|p| {
            only.is_none_or(|c| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.ends_with(&format!("-{c}.log")))
            })
        })
        .ok_or_else(|| match only {
            Some(c) => format!(
                "no trace of `{c}` in {TRACES}/ — it has not run here since the folder was cleaned"
            ),
            None => format!("no trace yet: every command that writes leaves one in {TRACES}/"),
        })?;
    println!(
        "\x1b[1m{}\x1b[0m\n",
        last.strip_prefix(root).unwrap_or(last).display()
    );
    print!("{}", fs::read_to_string(last).map_err(|e| e.to_string())?);
    Ok(())
}

// ---------------------------------------------------------------------------
// why a node is what it is

/// `why <node>` — every recorded belief, every change and who made it, its
/// approvals, how its code came to be, and its gate now.
pub(crate) fn cmd_why(root: &Path, args: &[&str]) -> Result<(), String> {
    let id = *args
        .first()
        .ok_or("usage: cargo run -p xtask -- why <node>")?;
    let tree = load(root)?;
    let sh = tree
        .sheets
        .get(id)
        .ok_or_else(|| format!("no node '{id}'. `cargo run -p xtask -- status` lists the tree"))?;
    let rel = sh
        .dir
        .strip_prefix(root)
        .unwrap_or(&sh.dir)
        .display()
        .to_string();
    println!("\x1b[1m{id}\x1b[0m — {}", sh.label);
    println!(
        "  {} · {} · owner {} · {} · {}",
        rel,
        if sh.is_seeded() {
            "seeded"
        } else {
            "published"
        },
        sh.owner,
        sh.criticality,
        if sh.is_declared() {
            "declared: a person picks its number"
        } else {
            "computed"
        }
    );
    if !sh.question.is_empty() {
        println!("  asks: {}", sh.question);
    }

    println!("\n\x1b[1mwhat it believes, and why it changed\x1b[0m");
    if sh.versions.is_empty() {
        println!("  no recorded version — its first belief comes on its form (docs/DERISKING.md)");
    }
    for v in &sh.versions {
        println!(
            "  v{} · {} · {} · by {} — {}",
            v.n,
            v.date,
            v.release,
            if v.by.is_empty() { "(unnamed)" } else { &v.by },
            one_line(&v.changed)
        );
        if !v.learned.is_empty() {
            println!("       learned: {}", one_line(&v.learned));
        }
        if !v.rests_on.is_empty() {
            println!("       rests on: {}", one_line(&v.rests_on));
        }
    }

    println!("\n\x1b[1mwho changed it (git)\x1b[0m");
    let log = std::process::Command::new("git")
        .args([
            "log",
            "-n",
            "10",
            "--format=%h %ad %an — %s",
            "--date=short",
            "--",
            &rel,
        ])
        .current_dir(root)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();
    if log.is_empty() {
        println!("  no history here (not a git checkout, or a shallow one)");
    }
    for l in log.lines() {
        println!("  {l}");
    }

    let approvals: Vec<String> = fs::read_dir(root.join("approvals"))
        .map(|rd| {
            rd.flatten()
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .filter(|n| n.ends_with(&format!("--{id}.toml")))
                .collect()
        })
        .unwrap_or_default();
    println!("\n\x1b[1mapprovals\x1b[0m");
    if approvals.is_empty() {
        println!("  none recorded in approvals/");
    }
    for a in approvals {
        println!("  approvals/{a}");
    }

    println!("\n\x1b[1mhow its code came to be\x1b[0m");
    if sh.is_declared() {
        println!("  a declared value: no code of its own");
    } else if vleo_sheet::method::node_program(sh).is_some() {
        let p = root
            .join("crates/vleo-core/src/physics/methods")
            .join(format!("{}.rs", sh.rust_ident()));
        println!(
            "  built from its method: {} {}",
            p.strip_prefix(root).unwrap_or(&p).display(),
            if p.is_file() {
                "— translated by rule"
            } else {
                "— not built yet: `cargo run -p xtask -- build-node` makes it"
            }
        );
    } else {
        let holes = vleo_sheet::load::read_holes(&sh.dir);
        let filled = holes.values().filter(|b| !b.trim().is_empty()).count();
        println!(
            "  hand-written holes: {filled} of {} filled",
            sh.steps.len()
        );
        if let Ok(t) = fs::read_to_string(sh.dir.join("fills.toml")) {
            for l in t
                .lines()
                .filter(|l| l.trim_start().starts_with("by") || l.trim_start().starts_with("model"))
            {
                println!("    {}", l.trim());
            }
        }
    }
    println!(
        "  fixtures: {} value(s) from outside the code",
        sh.fixtures.len()
    );

    println!("\n\x1b[1mits gate, run now\x1b[0m");
    let checks = gate::gate_node(sh, &tree);
    let bad: Vec<&gate::Check> = checks.iter().filter(|c| c.failed()).collect();
    if bad.is_empty() {
        println!("  \x1b[32mpasses\x1b[0m — {} check(s)", checks.len());
    }
    for c in bad {
        if let gate::Verdict::Fail(w) = &c.verdict {
            println!("  \x1b[31mFAIL\x1b[0m {} — {w}", c.name);
        }
    }

    let named: Vec<PathBuf> = traces(&root.join(TRACES))
        .into_iter()
        .rev()
        .filter(|p| {
            fs::read_to_string(p)
                .map(|t| {
                    t.lines()
                        .next()
                        .is_some_and(|l| l.split_whitespace().any(|w| w == id))
                })
                .unwrap_or(false)
        })
        .take(3)
        .collect();
    if !named.is_empty() {
        println!("\n\x1b[1mits last runs here\x1b[0m");
        for p in named {
            println!("  {}", p.strip_prefix(root).unwrap_or(&p).display());
        }
    }
    Ok(())
}

fn one_line(s: &str) -> String {
    let one = s.split_whitespace().collect::<Vec<_>>().join(" ");
    if one.chars().count() > 110 {
        format!("{}…", one.chars().take(109).collect::<String>())
    } else {
        one
    }
}

// ---------------------------------------------------------------------------
// docs/PIPELINE.md

/// The table, as a document.
pub(crate) fn pipeline_md() -> String {
    let mut o = String::new();
    o.push_str(
        "<!-- Generated by `cargo run -p xtask -- pipeline` from xtask/src/pipeline.rs. \
         Do not edit: change the table and regenerate. -->\n\n",
    );
    o.push_str("# The pipeline\n\n");
    o.push_str(
        "> **Answer first.** Every `xtask` command is a step in a node's journey — a form filled, \
         taken, checked, applied, published, built, gated, previewed and released. This page says, \
         for each one, what it reads, writes and checks, how to undo it, and where its code is. \
         Every command that writes prints numbered steps, stops by saying why, what state the files \
         are in and how to retry, leaves a trace in `target/xtask-trace/`, and takes `--dry-run`.\n>\n\
         > **Kind:** reference · **For:** the developer and their deputy\n\n",
    );
    o.push_str(
        "Generated from the one table in `xtask/src/pipeline.rs`; `cargo run -p xtask -- explain \
         <command>` prints the same for one command, and a test fails when the table, `help` and \
         the code disagree.\n\n## The journey\n\n",
    );
    for (i, (key, what)) in STAGES.iter().enumerate() {
        let names: Vec<String> = PIPELINE
            .iter()
            .filter(|c| c.stage == *key)
            .map(|c| format!("`{}`", c.name))
            .collect();
        o.push_str(&format!(
            "{}. **{key}** — {what}: {}\n",
            i + 1,
            names.join(", ")
        ));
    }
    o.push_str(
        "\n## When a step stops\n\nEvery command that writes prints its steps as `n/total · name`. \
         A step that stops prints:\n\n```\n3/4 · apply to the sheet — stopped\n    why       what \
         went wrong, in a sentence\n    files     the state the files are in\n    retry     the \
         exact command to try again\n    code      the file, and the step's name — search for it\n    \
         trace     target/xtask-trace/<time>-<command>.log\n```\n\n\
         `cargo run -p xtask -- trace` shows the last trace; `trace <command>` the last of one \
         command; `trace --list` every one kept (the newest 50). `cargo run -p xtask -- why <node>` \
         puts a node's history in one place: its recorded versions, who changed it, its approvals, \
         how its code came to be, and its gate now.\n\n## Every command\n",
    );
    for (key, what) in STAGES {
        let cmds: Vec<&Cmd> = PIPELINE.iter().filter(|c| c.stage == *key).collect();
        if cmds.is_empty() {
            continue;
        }
        o.push_str(&format!("\n### {key} — {what}\n"));
        for c in cmds {
            let (usage, prose) = usage_and_what(c.name);
            o.push_str(&format!("\n#### `{}`\n\n", c.name));
            for u in &usage {
                o.push_str(&format!("    cargo run -p xtask -- {u}\n"));
            }
            if !usage.is_empty() {
                o.push('\n');
            }
            if !prose.is_empty() {
                o.push_str(&format!("{prose}\n\n"));
            }
            o.push_str("| | |\n|---|---|\n");
            let cell = |s: &str| s.replace('|', "\\|");
            o.push_str(&format!("| reads | {} |\n", cell(or_none(c.reads))));
            o.push_str(&format!(
                "| writes | {} |\n",
                cell(if c.writes.is_empty() {
                    "nothing"
                } else {
                    c.writes
                })
            ));
            o.push_str(&format!("| checks | {} |\n", cell(or_none(c.checks))));
            o.push_str(&format!("| undo | {} |\n", cell(c.undo)));
            if !c.steps.is_empty() {
                o.push_str(&format!("| steps | {} |\n", cell(&numbered(c.steps))));
            }
            o.push_str(&format!("| dry run | {} |\n", cell(&dry_words(c))));
            o.push_str(&format!("| code | `{}` — `{}` |\n", c.code.0, c.code.1));
        }
    }
    o
}

/// `pipeline [--check]` — write docs/PIPELINE.md, or say whether it is current.
pub(crate) fn cmd_pipeline(root: &Path, args: &[&str]) -> Result<(), String> {
    let path = root.join("docs/PIPELINE.md");
    let want = pipeline_md();
    let have = fs::read_to_string(&path).unwrap_or_default();
    if args.contains(&"--check") {
        if have != want {
            return Err(
                "docs/PIPELINE.md is not the table as it is. Run `cargo run -p xtask -- pipeline` \
                 and commit"
                    .into(),
            );
        }
        println!("pipeline: docs/PIPELINE.md is the table as it is");
        return Ok(());
    }
    if have == want {
        println!("pipeline: docs/PIPELINE.md is current — nothing to write");
        return Ok(());
    }
    fs::write(&path, want).map_err(|e| format!("docs/PIPELINE.md: {e}"))?;
    println!(
        "pipeline: wrote docs/PIPELINE.md — {} commands",
        PIPELINE.len()
    );
    Ok(())
}

#[cfg(test)]
mod the_table_is_true {
    use super::*;

    fn root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
    }

    #[test]
    fn every_command_help_lists_is_in_the_table_and_nothing_else() {
        let in_help: std::collections::BTreeSet<&str> = HELP
            .lines()
            .filter(|l| l.starts_with("  ") && !l.starts_with("   "))
            .filter_map(|l| l.split_whitespace().next())
            .collect();
        let mut in_table: std::collections::BTreeSet<&str> =
            PIPELINE.iter().map(|c| c.name).collect();
        in_table.remove("help");
        assert_eq!(
            in_help, in_table,
            "help and the pipeline table list different commands"
        );
    }

    #[test]
    fn every_command_names_a_stage_and_a_function_that_exists() {
        let root = root();
        for c in PIPELINE {
            assert!(
                STAGES.iter().any(|(k, _)| *k == c.stage),
                "{}: no stage «{}»",
                c.name,
                c.stage
            );
            let src = fs::read_to_string(root.join(c.code.0))
                .unwrap_or_else(|e| panic!("{}: {}: {e}", c.name, c.code.0));
            assert!(
                src.contains(&format!("fn {}(", c.code.1)),
                "{}: {} has no fn {}",
                c.name,
                c.code.0,
                c.code.1
            );
            assert!(
                !c.undo.is_empty(),
                "{}: says nothing about undoing it",
                c.name
            );
            if c.writes.is_empty() {
                assert!(
                    matches!(c.dry, Dry::Reads),
                    "{}: writes nothing, so its dry run is to run it",
                    c.name
                );
            }
        }
    }

    #[test]
    fn every_step_the_table_names_is_a_string_in_its_code() {
        // So a printed step name, searched for, finds its line.
        let root = root();
        for c in PIPELINE {
            let src = fs::read_to_string(root.join(c.code.0)).unwrap();
            for s in c.steps {
                assert!(
                    src.contains(&format!("\"{s}\"")),
                    "{}: step «{s}» is not a string in {}",
                    c.name,
                    c.code.0
                );
            }
        }
    }

    #[test]
    fn a_check_mode_the_table_promises_is_a_flag_the_command_takes() {
        for c in PIPELINE {
            if let Dry::Check(_) = c.dry {
                if matches!(c.name, "intake" | "group-intake") {
                    continue; // their check is the command without --apply
                }
                assert!(
                    crate::flags_in_help(c.name).is_some_and(|f| f.contains(&"--check")),
                    "{}: --dry-run runs --check, which it does not take",
                    c.name
                );
            }
        }
    }

    #[test]
    fn the_pipeline_page_is_the_table() {
        let have = fs::read_to_string(root().join("docs/PIPELINE.md")).unwrap_or_default();
        assert!(
            have == pipeline_md(),
            "docs/PIPELINE.md is not the table as it is. Run `cargo run -p xtask -- pipeline`"
        );
    }

    #[test]
    fn a_stopped_step_says_why_the_files_retry_code_and_trace() {
        let dir = std::env::temp_dir().join(format!("xtask-run-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let mut run = Run::start(&dir, "publish", &["x"], 2);
        let first: u32 = run
            .step("read the sheet", OnStop::new("unchanged", "again"), || {
                Ok((7, "read".into()))
            })
            .unwrap();
        assert_eq!(first, 7);
        let err = run
            .step::<()>(
                "publish, generate and gate",
                OnStop::new("put back as they were", "cargo run -p xtask -- publish x"),
                || Err("the gate refused".into()),
            )
            .unwrap_err();
        assert!(
            err.contains("2/2")
                && err.contains("publish, generate and gate")
                && err.contains("the gate refused"),
            "{err}"
        );
        let t = traces(&dir.join(TRACES));
        assert_eq!(t.len(), 1, "one run, one trace");
        let text = fs::read_to_string(&t[0]).unwrap();
        for want in [
            "xtask publish x",
            "1/2 · read the sheet",
            "done  read",
            "why       the gate refused",
            "files     put back as they were",
            "retry     cargo run -p xtask -- publish x",
            "code      xtask/src/main.rs — the step \"publish, generate and gate\" in cmd_publish",
            "stopped after",
        ] {
            assert!(
                text.contains(want),
                "the trace does not say «{want}»:\n{text}"
            );
        }
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn old_traces_are_pruned_to_the_newest() {
        let dir = std::env::temp_dir().join(format!("xtask-prune-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        for i in 0..(KEEP + 7) {
            fs::write(dir.join(format!("2026-01-01T00-00-{i:03}Z-x.log")), "x").unwrap();
        }
        prune(&dir);
        let left = traces(&dir);
        assert_eq!(left.len(), KEEP);
        assert!(
            left[0].to_string_lossy().contains(&format!("{:03}Z", 7)),
            "the oldest were not the ones removed"
        );
        let _ = fs::remove_dir_all(&dir);
    }
}
