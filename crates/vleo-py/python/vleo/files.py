"""The tool's results file, read from Python with the standard library.

    vleo.results("study.vleor")   saved results, many in one file

It is an ordinary SQLite database (crates/vleo-results/results.sql), so
nothing here needs the engine: this module is plain Python and `sqlite3`. A
file that is not one of the tool's databases, is another kind, or comes from a
newer tool is refused by name rather than misread — by the one rule every
reader follows, whose table of file kinds is generated into this file from
crates/vleo-kinds and held to it by a test.
"""

import os
import sqlite3

# --- kinds: GENERATED from crates/vleo-kinds (python_table); do not edit ---
APP_ID = 1447838031  # 'VLEO', in every database the tools write
# (name, format, reader, becomes, says): every database kind the tools write.
KINDS = [
    ("structure", 1, "vleo-files", ("group", 2), "a group's structure before 1.0, written by the group application"),
    ("node", 1, "vleo-files", ("node", 2), "a node's file before 1.0, written by the node application"),
    ("release", 1, "vleo-files", ("group release", 2), "a group's sealed release before 1.0"),
    ("results", 1, "vleo-results", None, "saved results, many in one file"),
    ("node", 2, "vleo-files", None, "one node's file"),
    ("group", 2, "vleo-files", None, "a group's file"),
    ("group release", 2, "vleo-files", None, "a group's sealed release"),
    ("preview", 2, "vleo-files", None, "a preview sent to a group"),
    ("preview answer", 2, "vleo-files", None, "a group's answer to a preview"),
    ("issue", 2, "vleo-files", None, "an issue raised"),
    ("daily snapshot", 2, "vleo-files", None, "how the whole design stood on one day"),
    ("released design", 2, "vleo-files", None, "a design the system engineer released; never today's design"),
    ("case", 2, "vleo-files", None, "a case: the inputs a run sets"),
    ("results", 2, "vleo-files", None, "saved results"),
    ("key", 2, "vleo-files", None, "a person's key, locked by their passphrase"),
]
# --- end kinds ---

READER = "vleo-results"  # the files read here are the ones vleo-results reads


def identify(app_id, fmt, file_kind, names, called, reader=READER):
    """The kind a database file is, or the refusal, by the one rule every
    reader follows (crates/vleo-kinds, `identify`): answers ``(kind, None)``
    or ``(None, refusal)``, the refusal in words with ``{place}`` for the file."""
    newest_here = max([k[1] for k in KINDS if k[2] == reader] or [0])

    def newer(name):
        return ("{place}: a %s file in format %d, newer than this tool reads (%d): use the newer tool"
                % (name, fmt, newest_here))

    if app_id != APP_ID:
        return None, "{place}: not a database the VLEO tools wrote"
    if file_kind is None:
        if fmt > newest_here:
            return None, newer("VLEO")
        return None, "{place}: a VLEO file in format %d that does not say what kind it is" % fmt
    same = [k for k in KINDS if k[0] == file_kind]
    if not same:
        if fmt > newest_here:
            return None, newer(file_kind)
        return None, "{place}: says it is a %s file in format %d, which no VLEO tool writes" % (file_kind, fmt)
    this = [k for k in same if k[1] == fmt]
    becomes = bool(this) and this[0][3] is not None and this[0][3][0] in names
    if file_kind not in names and not becomes:
        found = (this or same)[0]
        return None, "{place}: a %s file, not %s — %s" % (found[0], called, found[4])
    mine = [k for k in this if k[2] == reader]
    if mine:
        return mine[0], None
    formats = [k[1] for k in same if k[2] == reader]
    if formats and fmt > max(formats):
        return None, ("{place}: a %s file in format %d, newer than this tool reads (%d): use the newer tool"
                      % (file_kind, fmt, max(formats)))
    return None, "{place}: says it is a %s file in format %d, which no VLEO tool writes" % (file_kind, fmt)


def _open(path, kind, called):
    path = os.fspath(path)
    with open(path, "rb") as f:
        if f.read(15) != b"SQLite format 3":
            raise ValueError("%s: not a database file" % path)
    db = sqlite3.connect("file:%s?mode=ro" % path, uri=True)
    app = db.execute("PRAGMA application_id").fetchone()[0]
    fmt = db.execute("PRAGMA user_version").fetchone()[0]
    found = None
    if app == APP_ID:
        try:
            row = db.execute("SELECT value FROM meta WHERE key = 'file_kind'").fetchone()
            found = row[0] if row else None
        except sqlite3.DatabaseError:
            found = None
    taken, refused = identify(app, fmt, found, [kind], called)
    if refused:
        db.close()
        raise ValueError(refused.replace("{place}", path))
    return db


def results(path):
    """Every saved result in a results file, as dicts, in the order written.

    Each has the result's identity (``name``, ``target``, ``question``,
    ``saved``, ``chain`` …), ``csv`` (the result whole, as its folder keeps
    it), ``sweep_csv`` and ``values``: every input, output and blocked row,
    with ``si`` the exact value in SI or None.
    """
    db = _open(path, "results", "a results file")
    try:
        cur = db.execute("SELECT * FROM result ORDER BY rowid")
        names = [c[0] for c in cur.description]
        out = [dict(zip(names, r)) for r in cur]
        for r in out:
            r["pinned"] = bool(r["pinned"])
            cur = db.execute(
                "SELECT section, id, name, value, unit, si, credibility, governing, note "
                "FROM value WHERE result = ? ORDER BY ord", (r["name"],))
            cols = [c[0] for c in cur.description]
            r["values"] = [dict(zip(cols, v)) for v in cur]
        return out
    finally:
        db.close()
