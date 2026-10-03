"""The tool's database files, read from Python with the standard library.

    vleo.design()                 the design the tool reads: design.vleo
    vleo.results("study.vleor")   saved results, many in one file

Both are ordinary SQLite databases (crates/vleo-design/design.sql and
results.sql), so nothing here needs the engine: this module is plain Python and
`sqlite3`. A file that is not one of the tool's databases, is another kind, or
comes from a newer tool is refused by name rather than misread.
"""

import os
import sqlite3

APP_ID = 1447838031  # 'VLEO', in every database the tools write
DESIGN_FORMAT = 1
RESULTS_FORMAT = 1


def _open(path, kind, newest):
    path = os.fspath(path)
    with open(path, "rb") as f:
        if f.read(15) != b"SQLite format 3":
            raise ValueError("%s: not a database file" % path)
    db = sqlite3.connect("file:%s?mode=ro" % path, uri=True)
    if db.execute("PRAGMA application_id").fetchone()[0] != APP_ID:
        db.close()
        raise ValueError("%s: not a database the VLEO tools wrote" % path)
    fmt = db.execute("PRAGMA user_version").fetchone()[0]
    if fmt > newest:
        db.close()
        raise ValueError("%s: format %d, newer than this package reads (%d): use the newer package"
                         % (path, fmt, newest))
    try:
        found = db.execute("SELECT value FROM meta WHERE key = 'file_kind'").fetchone()
    except sqlite3.DatabaseError:
        found = None
    found = found[0] if found else None
    if found != kind:
        db.close()
        raise ValueError("%s: a %s file, not a %s file" % (path, found or "VLEO", kind))
    return db


class Design:
    """design.vleo, open: the design the tool reads, as the tree it came from.

    ``meta`` says what it is and what it was built from; ``rows()`` lists the
    rows of the tree; ``file(path)`` is any file of it by its repository path,
    e.g. ``crates/vleo-mod-solar/nodes/sw_ap_design/node.toml``.
    """

    def __init__(self, path):
        self.path = os.fspath(path)
        self._db = _open(self.path, "design", DESIGN_FORMAT)
        self.meta = dict(self._db.execute("SELECT key, value FROM meta"))

    def rows(self, subsystem=None):
        cur = self._db.execute(
            "SELECT id, folder, layer, ord, parent, subsystem, kind, state, owner, label, question "
            "FROM row" + (" WHERE subsystem = ?" if subsystem else "") + " ORDER BY id",
            (subsystem,) if subsystem else ())
        names = [c[0] for c in cur.description]
        return [dict(zip(names, r)) for r in cur]

    def published(self, group=None):
        """What each group publishes to the others: one dict per row and
        reader (``grp``, ``node``, ``unit``, ``version``, ``crosses_to``,
        ``read_by_grp``, ``read_by``)."""
        cur = self._db.execute(
            "SELECT grp, node, unit, version, crosses_to, read_by_grp, read_by FROM published"
            + (" WHERE grp = ?" if group else "") + " ORDER BY grp, node, read_by",
            (group,) if group else ())
        names = [c[0] for c in cur.description]
        return [dict(zip(names, r)) for r in cur]

    def paths(self):
        return [p for (p,) in self._db.execute("SELECT path FROM file ORDER BY path")]

    def file(self, path):
        r = self._db.execute("SELECT bytes FROM file WHERE path = ?", (path,)).fetchone()
        if r is None:
            raise KeyError("%s is not in %s" % (path, self.path))
        return bytes(r[0])

    def text(self, path):
        return self.file(path).decode("utf-8")

    def close(self):
        self._db.close()


def design(path=None):
    """The design file: ``path``, or the one the package carries."""
    if path is None:
        here = os.path.dirname(os.path.abspath(__file__))
        path = os.path.join(here, "_kit", "design.vleo")
        if not os.path.isfile(path):
            raise FileNotFoundError(
                "this package carries no design.vleo (a developer's build); "
                "pass the path to one: `cargo run -p xtask -- design` writes target/design.vleo")
    return Design(path)


def results(path):
    """Every saved result in a results file, as dicts, in the order written.

    Each has the result's identity (``name``, ``target``, ``question``,
    ``saved``, ``chain`` …), ``csv`` (the result whole, as its folder keeps
    it), ``sweep_csv`` and ``values``: every input, output and blocked row,
    with ``si`` the exact value in SI or None.
    """
    db = _open(path, "results", RESULTS_FORMAT)
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
