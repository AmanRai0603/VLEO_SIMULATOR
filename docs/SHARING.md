# Sharing the tool with everyone

> **Answer first.** Everyone gets the tool, not the repository — as **one Python package for every
> laptop** (`vleo-<version>-py3-none-any.whl`, started with `python -m vleo`), or as a **kit**: one
> zip per platform with the programs and the files they read, and `START_HERE.md` on top. They run
> it and keep their inputs and results on their own machine. A group's work comes back as its
> sealed release; you take it in, release, and send the next version. Their case and results carry
> over on their own.
>
> **Kind:** how-to · **For:** developers

The loop, end to end:

    you                                   a group
    ───                                   ───────
    release → build a kit → share zip ──▶ unzip → start → use
                                          node files → a sealed release
    intake → build → test → accept   ◀──  (from the group's shared drive)
    release → build the next kit → share ─▶ replace the folder → start again

---

## 1 · Build a kit

    cargo build --release -p vleo-daemon -p vleo-cli
    cargo run -p xtask -- kit                   # → dist/vleo-<version>/

`dist/vleo-<version>/` holds:
- the two programs: `vleo-daemon` and `vleo` — on Windows `Start VLEO.exe` (the daemon, which opens
  the browser itself because of its name) and `vleo.exe`;
- the files they read: `web/`, `bundles/`, `docs/manual.toml`, and the design itself as one file,
  `design.vleo` — every node folder, the layers, the cases and the source list, written into one
  SQLite database by `cargo run -p xtask -- design` (below);
- `START_HERE.md` (this repository's `docs/TEAM_GUIDE.md`), `VERSION`, and `start.sh` off Windows.

No script starts the Windows program: a script launching an unknown program is one more thing an
antivirus weighs against it. The programs carry Windows version information (Orbitt Space, VLEO
design tool, the version) for the same reason — `tools/windows_identity.rs`.

There is no git history in it, no generator, and no kernel source. Zip the folder and share the
zip. `dist/` is ignored by git.

**The design travels as one file.** A developer edits the tree as folders, because review and the
gate work on them; the tool everyone runs needs the design whole, as one thing that cannot be
half-copied. So the kit carries `design.vleo`, and the daemon reads it through the same interface
it reads the folders through, with every check the loader makes. A page served from it is the page
served from the folders, byte for byte — `crates/vleo-server/tests/the_design_file_serves_the_same_pages.rs`
asks every node's page of both. To write one or hold one to the tree:

    cargo run -p xtask -- design                         # → target/design.vleo
    cargo run -p xtask -- design --check dist/vleo-<version>/design.vleo

It is ordinary SQLite (`crates/vleo-design/design.sql`): Python reads its `row` table, and any
file in it by its repository path, with the standard library. `VLEO_DESIGN` points the daemon at
another one; a design file that does not open stops the daemon rather than falling back to
whatever folders sit beside it.

**One kit per platform.** The programs are built for the machine that built them. For an engineer
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

## 2 · Take a group's release back

A group's work arrives as its sealed release, `<group>-<version>.vleo`, never as one node at a
time. Unpack it, then take it in:

    node tools/group_db.mjs --unpack <file.vleo> --out <dir>
    cargo run -p xtask -- group-intake <dir> --apply

It goes on its own branch, `group/<group>-<version>` from `maintainer`, and through `group-build`,
`group-test`, `group-deliver` and `group-accept`; it merges only with the subsystem engineer's
acceptance of the exact test application they tried. A release that cannot be taken in goes back
to the group with the lines that say why, and is never edited here. Every step is in
`docs/GROUP_APPS.md` and in the intake guide, `docs/roles/maintainer.html`.

## 3 · Release and share the next kit

    cargo run -p xtask -- ship <version>    # the release branch: narrative, stamp, gate, tests, push
    # merge its pull request, tag v<version>; the pipeline builds the three kits and the package

The engineer replaces their folder with the new one. Their inputs, saved case and results are
under `~/.vleo/`, outside the folder, so they carry over; an input that no longer exists is set
aside by name, and a kept result whose belief has changed says so on the Results page.

## 4 · The readers' folder — for people who only read

    cargo run -p xtask -- readers               # → target/readers/

A folder of plain pages for anyone who wants to read the design without running the tool:
`index.html`, every row's page (the tool's own generated page) and every lesson. Put it on a
shared drive — the pages open from a file — or on any internal web server. Nothing contacts a
network, and nothing in it is committed.

A lesson's try-it widgets work there too: the folder carries the engine compiled for the browser
(`crates/vleo-kernel-wasm`, about 300 KB compressed), which answers them with the same relations
the tool runs, on the declared values. It carries no reference data, so a row that reads a data
bundle refuses in the page and says to open it in the tool. Rebuild the folder with each release.

The pages look as the tool does: they are the one page template (`web/page.html`) with the tool's
stylesheet, and the tool's type (IBM Plex Mono, `web/fonts`, with its licence) travels in
`assets/fonts`. The pipeline builds the folder on every push and opens it from a file in a
browser (`tools/readers_check.py`): the pages load, the engine answers, refuses and sweeps, a
widget moves — and every answer the page's engine gave is asked again of the engine built
natively, and must be the same, byte for byte.

## Where the simple version breaks

A kit is read-only by design, but nothing stops an engineer editing a file inside it. That edit is
invisible to everyone else and is lost when the next kit replaces the folder. Anything that should
change the design comes back in its group's sealed release. And a kit is only as current as the release it was built
from: `VERSION` in the folder, and the version the tool prints on start, say which one it is.
