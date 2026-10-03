-- THE DESIGN FILE, format 1: design.vleo.
--
-- The design as the tool a team runs reads it: every file of the tree the
-- loader reads — each node folder under crates/vleo-mod-*/nodes/, the layers,
-- the cases and the source list — held whole, byte for byte, under the path it
-- has in the repository. Written by `cargo run -p xtask -- design` from the
-- tree; never edited. The folders are what a developer changes; this file is
-- what a release carries in their place.
--
-- It is an ordinary SQLite database, like the files a group keeps
-- (groups/schema.sql): the same application_id, the same `meta` table, and
-- `file_kind` = design. Python opens it with the standard library:
--
--   import sqlite3
--   db = sqlite3.connect("design.vleo")
--   db.execute("SELECT id, question FROM row WHERE subsystem = 'solar'")
--   db.execute("SELECT bytes FROM file WHERE path = ?", (path,))

PRAGMA application_id = 1447838031;
PRAGMA user_version = 1;

-- What this file is: file_kind (design), format, tool (the version that wrote
-- it), commit, built, rows, files, fingerprint.
CREATE TABLE meta (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL
);

-- Every file, by its path in the repository, with '/' between parts.
CREATE TABLE file (
  path    TEXT PRIMARY KEY,
  sha256  TEXT NOT NULL,
  bytes   BLOB NOT NULL
);

-- One line per row of the tree, for a reader who wants the rows without
-- reading their sheets. Derived from the files above; the files are the design.
CREATE TABLE row (
  id         TEXT PRIMARY KEY,
  folder     TEXT NOT NULL,
  layer      INTEGER NOT NULL,
  ord        INTEGER NOT NULL,
  parent     TEXT NOT NULL,
  subsystem  TEXT NOT NULL,
  kind       TEXT NOT NULL,
  state      TEXT NOT NULL,
  owner      TEXT NOT NULL,
  label      TEXT NOT NULL,
  question   TEXT NOT NULL
);
