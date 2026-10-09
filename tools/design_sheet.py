"""A node's sheet, as the design holds it, for the tools outside the program.

    import design_sheet
    design_sheet.sheet("sw_band_confidence")        the sheet's TOML text
    design_sheet.parsed("sw_band_confidence")       the same, through tomllib
    design_sheet.in_design("sw_band_confidence")    whether the design holds it

WHY THIS EXISTS. The node sheets used to be files under `crates/*/nodes/`, and
every tool that wanted one opened the path. Those files are gone: the design is
`design/`, and there is ONE reader of it, in Rust. A second reader here, opening
the SQLite files itself, would be a second statement of the format that could
drift from the first and agree with itself while it did. So this asks the one
reader, through `cargo run -p xtask -- sheet`, which prints each sheet's text
exactly as every Rust reader sees it.

WHAT THE TEXT IS. A regenerated TOML, not the file a person once wrote: tables
may come in alphabetical order and numbers may be written differently. A word
or an id found in it is found in the design; a line layout or a number's
spelling is not something to rely on, so read a value through `parsed`.

ONE CALL PER PROCESS, where it can be. The command builds and reads the whole
design each time it runs, so `preload` fetches every id a tool will need at
once, and every answer is kept for the rest of the process. An id the design
does not hold is remembered as absent, so asking twice does not run it twice.

AND THE SELFTESTS. A selftest proves a check goes red by handing it a broken
sheet. With no file to break, `inject` puts the broken text into this cache in
place of the design's, for this process only, and `forget` takes it out again.
Nothing is ever written back: this module cannot change the design.
"""

import json
import os
import re
import subprocess

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

#: id -> sheet text, or None for an id the design was asked for and does not hold.
_CACHE = {}

#: What `inject` returns for an id it found nothing held for.
_NOT_HELD = object()

_ANSI = re.compile(r"\x1b\[[0-9;]*m")
_MISSING = re.compile(r"no node '([^']+)' in the design")


class NotInDesign(LookupError):
    """The design holds no node by this id. Not a failure to read the design."""

    def __init__(self, node):
        super().__init__("no node '%s' in the design" % node)
        self.node = node


def _run(args):
    """One run of the command. The JSON on success; otherwise the refusal."""
    cmd = ["cargo", "run", "-q", "-p", "xtask", "--", "sheet"] + list(args)
    try:
        p = subprocess.run(cmd, cwd=ROOT, capture_output=True, text=True)
    except FileNotFoundError:
        raise SystemExit("design_sheet: cargo is not on PATH, and the design is "
                         "read only through `cargo run -p xtask -- sheet`. Install "
                         "the Rust toolchain or run this where it is installed.")
    if p.returncode == 0:
        try:
            return json.loads(p.stdout), None
        except ValueError as exc:
            raise SystemExit("design_sheet: `%s` printed something that is not "
                             "JSON (%s) — the command and this reader disagree "
                             "about its output" % (" ".join(cmd), exc))
    return None, _ANSI.sub("", p.stderr).strip()


def preload(ids):
    """Fetch every id not already held, in one call where the design holds them all.

    A refusal names one missing id at a time, so a missing id is recorded as
    absent and the call is made again without it. Any other refusal is the
    design failing to read, and that stops the tool: a check that carried on
    would report every sheet as missing and be wrong about all of them.
    """
    want = [i for i in dict.fromkeys(ids) if i not in _CACHE]
    while want:
        got, err = _run(want)
        if got is not None:
            for i in want:
                _CACHE[i] = got.get(i)
            return
        m = _MISSING.search(err or "")
        if not m or m.group(1) not in want:
            raise SystemExit("design_sheet: `cargo run -p xtask -- sheet` failed, "
                             "so the design could not be read:\n%s" % (err or "(no message)"))
        _CACHE[m.group(1)] = None
        want.remove(m.group(1))


def preload_all():
    """Every sheet the design holds, in one call. For a tool that walks the tree."""
    got, err = _run(["--all"])
    if got is None:
        raise SystemExit("design_sheet: `cargo run -p xtask -- sheet --all` failed, "
                         "so the design could not be read:\n%s" % (err or "(no message)"))
    for i, text in got.items():
        _CACHE.setdefault(i, text)


def in_design(node):
    """Whether the design holds a node by this id."""
    preload([node])
    return _CACHE[node] is not None


def sheet(node):
    """The node's sheet text. Raises NotInDesign for an id the design does not hold."""
    preload([node])
    if _CACHE[node] is None:
        raise NotInDesign(node)
    return _CACHE[node]


def sheets(ids):
    """{id: sheet text} for every id the design holds; an absent id is left out."""
    preload(ids)
    return {i: _CACHE[i] for i in ids if _CACHE[i] is not None}


def parsed(node):
    """The node's sheet through tomllib. Raises NotInDesign as `sheet` does."""
    import tomllib
    return tomllib.loads(sheet(node))


def inject(node, text):
    """For a selftest: `text` stands for the node's sheet, in this process only.

    None stands for a node the design does not hold. Returns what was there, so
    `forget` can put it back.
    """
    before = _CACHE.get(node, _NOT_HELD)
    _CACHE[node] = text
    return before


def forget(node, before=None):
    """Undo `inject`, given what it returned: put back what was held, or, with
    nothing to put back, drop the entry so the next read asks the design."""
    if before is None or before is _NOT_HELD:
        _CACHE.pop(node, None)
    else:
        _CACHE[node] = before
