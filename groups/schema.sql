-- THE GROUP DATABASE, format 1.
--
-- One schema for the three kinds of file a group keeps on its shared drive:
--
--   structure   solar.vgroup         the nodes, their contracts and the arrows
--                                    between them, the group's own text, the
--                                    members. The group lead writes it.
--   node        <node>.vnode         one node's content, and a copy of its
--                                    contract. Its author writes it.
--   release     solar-1.0.vleo       the structure and every node assembled and
--                                    sealed. Written once, never changed.
--
-- A file says which of the three it is in `meta`. Every file is an ordinary
-- SQLite database: the group applications, Python and the developer's tools
-- open it as it is.
--
-- The content is the group folder pattern (groups/SPEC.toml) held in tables:
-- identity, contracts, people and the record of who did what are columns; the
-- texts are their Markdown; the tables the pattern names are their CSV; files
-- that are not tables — images, PDFs, code — are bytes. Reading a database as a
-- folder and a folder as a database is lossless both ways (web/js/gdb.js).
--
-- application_id is 'VLEO'. user_version is this format's number: an
-- application refuses a file newer than it knows and upgrades an older one,
-- keeping a copy of the file as it was.

PRAGMA application_id = 1447838031;
PRAGMA user_version = 1;

-- What this file is: file_kind (structure | node | release), group_id,
-- group_name, owner, version, summary, contract_version, node_uid (a node file),
-- revision, sealed (a release), fingerprint (a release), written_by_app.
CREATE TABLE meta (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL
);

CREATE TABLE member (
  name  TEXT PRIMARY KEY,
  role  TEXT NOT NULL CHECK (role IN ('owner', 'author', 'reviewer'))
);

-- A node's CONTRACT — what the group lead decides — and where it sits. `uid`
-- never changes; `id` is the name people read and may be renamed.
CREATE TABLE node (
  uid               TEXT PRIMARY KEY,
  id                TEXT NOT NULL UNIQUE,
  question          TEXT NOT NULL DEFAULT '',
  kind              TEXT NOT NULL CHECK (kind IN ('computed', 'declared', 'required', 'achieved')),
  output            TEXT NOT NULL DEFAULT '',
  unit              TEXT NOT NULL DEFAULT '',
  lower             TEXT NOT NULL DEFAULT '',
  upper             TEXT NOT NULL DEFAULT '',
  value             TEXT NOT NULL DEFAULT '',
  stage             TEXT NOT NULL DEFAULT '',
  author            TEXT NOT NULL DEFAULT '',
  ord               INTEGER NOT NULL DEFAULT 0,
  archived          INTEGER NOT NULL DEFAULT 0,
  contract_version  INTEGER NOT NULL DEFAULT 1,
  revision          INTEGER NOT NULL DEFAULT 0
);

-- A node's inputs — the arrows of the group. `source` is a node's uid in this
-- group, `group.node` for another group's published output, or `case` for a
-- value a user sets. Part of the contract.
CREATE TABLE input (
  node_uid  TEXT NOT NULL REFERENCES node (uid),
  ord       INTEGER NOT NULL,
  name      TEXT NOT NULL,
  symbol    TEXT NOT NULL DEFAULT '',
  source    TEXT NOT NULL,
  unit      TEXT NOT NULL DEFAULT '',
  dflt      TEXT NOT NULL DEFAULT '',
  min       TEXT NOT NULL DEFAULT '',
  max       TEXT NOT NULL DEFAULT '',
  says      TEXT NOT NULL DEFAULT '',
  PRIMARY KEY (node_uid, name)
);

-- Text: `scope` is 'group' or a node's uid; `kind` is explanation, theory, flow,
-- pseudocode or how_run.
CREATE TABLE doc (
  scope  TEXT NOT NULL,
  kind   TEXT NOT NULL,
  body   TEXT NOT NULL,
  PRIMARY KEY (scope, kind)
);

-- Every table the pattern names, and every dataset a picture draws, as CSV.
-- `path` is where the pattern puts it, relative to the group or to the node:
-- requirements.csv, figures.csv, results/isolation.csv, figures/speed.csv …
CREATE TABLE tbl (
  scope  TEXT NOT NULL,
  path   TEXT NOT NULL,
  csv    TEXT NOT NULL,
  PRIMARY KEY (scope, path)
);

-- Files that are not tables: images, video, PDFs, code. Stored once each.
CREATE TABLE media (
  scope   TEXT NOT NULL,
  path    TEXT NOT NULL,
  type    TEXT NOT NULL,
  sha256  TEXT NOT NULL,
  bytes   BLOB NOT NULL,
  PRIMARY KEY (scope, path)
);

-- Sign-offs, each against the content it was given for.
CREATE TABLE review (
  name         TEXT NOT NULL,
  scope        TEXT NOT NULL,
  version      TEXT NOT NULL DEFAULT '',
  fingerprint  TEXT NOT NULL,
  date         TEXT NOT NULL,
  verdict      TEXT NOT NULL CHECK (verdict IN ('ok', 'changes')),
  note         TEXT NOT NULL DEFAULT ''
);

-- Comments on any part, and an author's request to change their contract.
CREATE TABLE comment (
  scope     TEXT NOT NULL,
  section   TEXT NOT NULL DEFAULT '',
  author    TEXT NOT NULL,
  at        TEXT NOT NULL,
  body      TEXT NOT NULL,
  resolved  INTEGER NOT NULL DEFAULT 0
);
CREATE TABLE request (
  node_uid  TEXT NOT NULL,
  author    TEXT NOT NULL,
  at        TEXT NOT NULL,
  body      TEXT NOT NULL,
  status    TEXT NOT NULL DEFAULT 'open' CHECK (status IN ('open', 'accepted', 'declined')),
  answer    TEXT NOT NULL DEFAULT ''
);

-- Every change, as it was made: who, when, to what, and what it said before.
-- Undo, "what changed since 1.0" and the history all read this.
CREATE TABLE change (
  at      TEXT NOT NULL,
  who     TEXT NOT NULL,
  scope   TEXT NOT NULL,
  what    TEXT NOT NULL,
  before  TEXT,
  after   TEXT
);
