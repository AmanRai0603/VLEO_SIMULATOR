-- THE RESULTS FILE, format 1: results.vleor.
--
-- Saved results, many in one file: to send a set of them to someone, to keep
-- a study's results together, or to read them in Python or a spreadsheet tool
-- that speaks SQL. Written by `vleo results export`, read back into a results
-- folder by `vleo results import`. Nothing in it is run again: a saved result
-- is a record of what one engine said on one set of inputs.
--
-- Each result is held whole as the CSV a result's folder keeps (`csv`, and
-- `sweep_csv` for a sweep), which is what import reads, so a result goes out
-- and comes back exactly as it was. The `value` table is the same numbers as
-- rows, for a reader who wants SQL rather than CSV:
--
--   import sqlite3
--   db = sqlite3.connect("study.vleor")
--   db.execute("SELECT r.target, v.si FROM value v JOIN result r ON r.name = v.result
--               WHERE v.section = 'output' AND v.id = r.target")
--
-- An ordinary SQLite database, with the same identity (application_id 'VLEO')
-- and `meta` table as every database the tools write, and `file_kind` = results.

PRAGMA application_id = 1447838031;
PRAGMA user_version = 1;

-- What this file is: file_kind (results), format, tool, written, results.
CREATE TABLE meta (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL
);

-- One saved result. `name` is its folder's name in a results folder.
CREATE TABLE result (
  name       TEXT PRIMARY KEY,
  question   TEXT NOT NULL,
  target     TEXT NOT NULL,
  mode       TEXT NOT NULL,
  saved      TEXT NOT NULL,
  label      TEXT NOT NULL,
  chain      TEXT NOT NULL,
  kernel     TEXT NOT NULL,
  graph      TEXT NOT NULL,
  case_id    TEXT NOT NULL,
  ran        INTEGER NOT NULL,
  blocked    INTEGER NOT NULL,
  thinned    TEXT NOT NULL,
  pinned     INTEGER NOT NULL,
  csv        TEXT NOT NULL,
  sweep_csv  TEXT NOT NULL
);

-- Every row of every result: an input it ran on, a value it returned, or a
-- row it could not run (`section` input | output | blocked). `si` is the exact
-- value in SI, empty where there is none.
CREATE TABLE value (
  result       TEXT NOT NULL REFERENCES result (name),
  section      TEXT NOT NULL CHECK (section IN ('input', 'output', 'blocked')),
  ord          INTEGER NOT NULL,
  id           TEXT NOT NULL,
  name         TEXT NOT NULL,
  value        TEXT NOT NULL,
  unit         TEXT NOT NULL,
  si           REAL,
  credibility  TEXT NOT NULL,
  governing    TEXT NOT NULL,
  note         TEXT NOT NULL,
  PRIMARY KEY (result, section, ord)
);
