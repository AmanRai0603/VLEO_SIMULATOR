# Sharing the tool with the team

> **Answer first.** The team gets the tool, not the repository — as **one Python package for every
> laptop** (`vleo-<version>-py3-none-any.whl`, started with `python -m vleo`), or as a **kit**: one
> zip per platform with the programs and the files they read, and `START_HERE.md` on top. They run
> it, keep their inputs and results on their own machine, and send back filled node forms. You
> check and apply each form, release, and send the next version. Their case and results carry over
> on their own.
>
> **Kind:** how-to · **For:** developers

The loop, end to end:

    you                                   a team member
    ───                                   ─────────────
    release → build a kit → share zip ──▶ unzip → start → use
                                          fill a node form → send the file
    intake → apply → publish → gate  ◀──  (e-mail, chat, a shared drive)
    release → build the next kit → share ─▶ replace the folder → start again

---

## 1 · Build a kit

    cargo build --release -p vleo-daemon -p vleo-cli
    cargo run -p xtask -- kit                   # → dist/vleo-<version>/

`dist/vleo-<version>/` holds:
- the two programs: `vleo-daemon` and `vleo` — on Windows `Start VLEO.exe` (the daemon, which opens
  the browser itself because of its name) and `vleo.exe`;
- the files they read: `web/`, `layers/`, `cases/`, `sources/`, `bundles/`, `docs/manual.toml`,
  and every node folder;
- `START_HERE.md` (this repository's `docs/TEAM_GUIDE.md`), `VERSION`, and `start.sh` off Windows.

No script starts the Windows program: a script launching an unknown program is one more thing an
antivirus weighs against it. The programs carry Windows version information (Orbitt Space, VLEO
design tool, the version) for the same reason — `tools/windows_identity.rs`.

There is no git history in it, no generator, and no kernel source. Zip the folder and share the
zip. `dist/` is ignored by git.

**One kit per platform.** The programs are built for the machine that built them. For a teammate
on another platform, use the release: `.github/workflows/release.yml` builds a kit on Linux,
macOS and Windows and attaches `vleo-<version>-<platform>.zip` to the GitHub release. Anyone
with access to the release page can download it without the repository.

**One package for every laptop.** The release also builds `vleo-<version>-py3-none-any.whl`: the
engine built for each system (Windows, Apple-silicon and Intel macOS, Linux) as a Python module,
with the kit's files beside it, in one file. `python -m pip install` puts it in place and
`python -m vleo` starts the same server inside python.exe, so there is no new program for Windows
or an antivirus to question — the reason it exists. It is checked by installing it and running
`python -m vleo --check` on each system. To build one yourself:

    cargo build --release --manifest-path crates/vleo-py/Cargo.toml
    cargo run -p xtask -- kit --files-only --out dist/wheel-kit
    python3 tools/build_wheel.py --version <version> --kit dist/wheel-kit \
        --native linux-x86_64=crates/vleo-py/target/release/lib_vleo.so

— with this machine's engine only; the release carries all of them.

## 2 · Take a form back

A teammate sends a filled `*.node-form.html`. It goes through the developer's loop in
[`AGENTS.md`](../AGENTS.md), unchanged:

    cargo run -p xtask -- intake <form.html>            # check — writes nothing
    cargo run -p xtask -- intake <form.html> --apply    # apply, with its de-risking record
    cargo run -p xtask -- publish <node>                # a new node: generate its code
    cargo run -p xtask -- fill <node> --hole <n> --body - --by "<who>" --model <model>
    cargo run -p xtask -- gate && cargo test

If it does not pass the check, send the file back with the lines intake printed. Commit naming
whoever filled the form, review, merge.

## 3 · Release and share the next kit

    cargo run -p xtask -- derisk
    cargo run -p xtask -- release <version>    # stamps every `next` version
    # commit, merge, tag v<version>; the pipeline builds the three kits and the package

The teammate replaces their folder with the new one. Their inputs, saved case and results are
under `~/.vleo/`, outside the folder, so they carry over; an input that no longer exists is set
aside by name, and a kept result whose belief has changed says so on the Results page.

## Where the simple version breaks

A kit is read-only by design, but nothing stops a teammate editing a file inside it. That edit is
invisible to everyone else and is lost when the next kit replaces the folder. Anything that should
change the design comes back as a form. And a kit is only as current as the release it was built
from: `VERSION` in the folder, and the version the tool prints on start, say which one it is.
