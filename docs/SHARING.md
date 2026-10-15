# Sharing the tool with everyone

> **Answer first.** Everyone gets the tool, not the repository — as **one Python package for every
> laptop** (`vleo-<version>-py3-none-any.whl`, started with `python -m vleo`), or as a **kit**: one
> zip per platform with the programs and the files they read, and `START_HERE.md` on top. They run
> it and keep their inputs and results on their own machine. A group's work comes back as its
> sealed release on the shared drive, and the tool builds today's design from it; you release and
> send the next version of the tool. Their case and results carry over on their own.
>
> **Kind:** how-to · **For:** developers

The loop, end to end:

    you                                   a group
    ───                                   ───────
    release → build a kit → share zip ──▶ unzip → start → use
                                          node files → a sealed release
                                          → groups/<group>/releases/ on the drive
                                          → today's design, for everyone who opens it
    release → build the next kit → share ─▶ replace the folder → start again

---

## 1 · Build a kit

    cargo build --release -p vleo-daemon -p vleo-cli
    cargo run -p xtask -- kit                   # → dist/vleo-<version>/

`dist/vleo-<version>/` holds:
- the two programs: `vleo-daemon` and `vleo` — on Windows `Start VLEO.exe` (the daemon, which opens
  the browser itself because of its name) and `vleo.exe`;
- the files they read: `web/`, `bundles/`, `docs/manual.toml`, the design itself as its files,
  `design/`, and beside it what each node folder holds that is the code's (its generated module,
  contract and evidence) and the prior implementation's grid (`parity.csv`), never a sheet;
- `START_HERE.md` (this repository's `docs/TEAM_GUIDE.md`), `VERSION`, and `start.sh` off Windows.

No script starts the Windows program: a script launching an unknown program is one more thing an
antivirus weighs against it. The programs carry Windows version information (Orbitt Space, VLEO
design tool, the version) for the same reason — `tools/windows_identity.rs`.

There is no git history in it, no generator, and no kernel source. Zip the folder and share the
zip. `dist/` is ignored by git.

**The design travels as its files.** The kit carries `design/`, each group's file and each node's,
as a checkout holds them, and the daemon reads them through the same interface in both, with every
check the loader makes. A page served from a kit is the page served from a checkout, byte for
byte — `crates/vleo-server/tests/a_kit_serves_the_pages_a_checkout_serves.rs` asks every node's
page of both. `VLEO_DESIGN` points the daemon at another folder of the design's files; one that is
not a folder, or does not open, stops the daemon rather than falling back to whatever sits beside
it, and so does a file of it written by a newer application than the daemon.

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

## 2 · A group's release, on the drive

A group's work arrives as its sealed release, `<group>-<version>.vleo`, never as one node at a
time. Its subsystem engineer puts it in the group's folder on the shared drive,
`groups/<group>/releases/`, and nobody takes it in by hand. When the tool opens on the drive
(`VLEO_DRIVE`), it builds today's design from every group's latest sealed release that passes its
checks — its seal, its content, and each node against the design. A group whose latest release is
refused is built from its last good release, and the tool says which release each group's part is
and why one was refused. A refused release goes back to the group, and is never edited here. The
group's side is in `docs/GROUP_APPS.md`.

## 3 · Release and share the next kit

    cargo run -p xtask -- ship <version>    # the release branch: narrative, version, gate, tests, push
    # merge its pull request, tag v<version>; the pipeline builds the three kits and the package

The engineer replaces their folder with the new one. Their inputs, saved case and results are
under `~/.vleo/`, outside the folder, so they carry over; an input that no longer exists is set
aside by name, and a kept result whose belief has changed says so under Results kept.

## 4 · For people who only read

There is no separate folder for them now. The readers' folder (`xtask readers`) was removed in
phase F2 (docs/PLAN_F.md, decision 1): the application replaces today's pages, and nothing is
kept beside it. Someone who only reads gets the tool like everyone else, from a kit or the
package, and opens the design in it.

The engine compiled for the browser (`crates/vleo-kernel-wasm`) is kept, though no page carries
it. The pipeline builds it, opens the design in it and asks it to answer, refuse and sweep
(`tools/kernel_check.mjs`), and every answer it gave is asked again of the engine built natively
and must be the same, byte for byte. Opening the application without installing it is not
planned; somebody who needs it raises a request (W14).

## Where the simple version breaks

A kit is read-only by design, but nothing stops an engineer editing a file inside it. That edit is
invisible to everyone else and is lost when the next kit replaces the folder. Anything that should
change the design comes back in its group's sealed release. And a kit is only as current as the release it was built
from: `VERSION` in the folder, and the version the tool prints on start, say which one it is.
