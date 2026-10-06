-- THE DESIGN FILE, format 2: one schema for every kind of file.
--
-- docs/OPERATING_1_0.md, section 10. Every file a person keeps is one SQLite
-- database in this schema: a node, a group, a group's release, a preview and
-- its answer, an issue, a daily snapshot, a released design, a case, results
-- and a key. Its `meta` says which kind it is, its version, what it was based
-- on and who wrote it (section 13); the tables a kind does not use are empty.
-- One schema, so every check reads one shape and nothing a person writes has
-- to be carried across from another.
--
-- The model is docs/SYSTEM_MODEL.md: a block, to any depth, with its ports;
-- wires between ports; a mount where a group hangs; a closure of a
-- requirement against an achieved value; a loop declared on the block that
-- holds it; cases as evidence; texts, tables and media; signatures; and the
-- record of every change, request, issue and comment.
--
-- Read and written only by crates/vleo-files, installed and in the page.
-- application_id is every VLEO file's, as format 1 had it; user_version is this
-- format's number: a file newer than the reader is refused by name, and an
-- older one is upgraded with a copy kept of it as it was (phase C4).

PRAGMA application_id = 1447838031;
PRAGMA user_version = 2;

-- What this file is. Every file: file_kind, format, written_by_app. The keys
-- each kind must have, and the versions they hold, are in src/meta.rs.
CREATE TABLE meta (
  key    TEXT PRIMARY KEY,
  value  TEXT NOT NULL
);

-- The people a file registers, and their keys (section 14). The programme's
-- file lists the system engineer, the subsystem engineers and every deputy; a
-- group's file lists its node engineers. A replaced key stays, with the date
-- it stopped, so what it signed before still checks.
CREATE TABLE person (
  name        TEXT PRIMARY KEY,
  role        TEXT NOT NULL CHECK (role IN ('programme manager', 'system engineer',
                'subsystem engineer', 'node engineer', 'developer')),
  deputy_for  TEXT NOT NULL DEFAULT ''
);
CREATE TABLE person_key (
  person         TEXT NOT NULL REFERENCES person (name),
  public_key     TEXT NOT NULL,
  registered_at  TEXT NOT NULL,
  registered_by  TEXT NOT NULL,
  revoked_from   TEXT NOT NULL DEFAULT '',
  PRIMARY KEY (person, public_key)
);

-- Who writes which block, from which contract version. One writer per file.
CREATE TABLE assignment (
  block_uid         TEXT NOT NULL,
  person            TEXT NOT NULL,
  contract_version  INTEGER NOT NULL,
  PRIMARY KEY (block_uid, person)
);

-- A block, to any depth: `parent_uid` is the block that contains it, '' at
-- the top of a file. `uid` never changes; `id` is the name people read.
CREATE TABLE block (
  uid               TEXT PRIMARY KEY,
  id                TEXT NOT NULL UNIQUE,
  parent_uid        TEXT NOT NULL DEFAULT '',
  question          TEXT NOT NULL DEFAULT '',
  behaviour         TEXT NOT NULL CHECK (behaviour IN ('method', 'children', 'stated',
                      'lookup', 'open')),
  perspective       TEXT NOT NULL DEFAULT '' CHECK (perspective IN ('', 'management',
                      'system', 'subsystem')),
  ord               INTEGER NOT NULL DEFAULT 0,
  archived          INTEGER NOT NULL DEFAULT 0,
  contract_version  INTEGER NOT NULL DEFAULT 1,
  revision          INTEGER NOT NULL DEFAULT 0
);

-- A block's inputs and outputs (SYSTEM_MODEL, sections 3 and 4): its type,
-- unit, range and the reason for the range; its state and maturity; while
-- open, who owns it and the gate it is due by. Ports in one `bundle` travel
-- as one wire.
CREATE TABLE port (
  block_uid     TEXT NOT NULL REFERENCES block (uid),
  direction     TEXT NOT NULL CHECK (direction IN ('in', 'out')),
  name          TEXT NOT NULL,
  port_type     TEXT NOT NULL CHECK (port_type IN ('number', 'integer', 'choice', 'boolean',
                  'list', 'table', 'series', 'uncertain', 'text', 'file')),
  unit          TEXT NOT NULL DEFAULT '',
  lower         TEXT NOT NULL DEFAULT '',
  upper         TEXT NOT NULL DEFAULT '',
  range_reason  TEXT NOT NULL DEFAULT '',
  state         TEXT NOT NULL CHECK (state IN ('decided', 'allocated', 'open', 'achieved')),
  maturity      TEXT NOT NULL DEFAULT '' CHECK (maturity IN ('', 'estimated', 'calculated',
                  'measured')),
  value         TEXT NOT NULL DEFAULT '',
  choices       TEXT NOT NULL DEFAULT '',
  bundle        TEXT NOT NULL DEFAULT '',
  open_owner    TEXT NOT NULL DEFAULT '',
  open_due      TEXT NOT NULL DEFAULT '',
  says          TEXT NOT NULL DEFAULT '',
  ord           INTEGER NOT NULL DEFAULT 0,
  PRIMARY KEY (block_uid, direction, name)
);

-- A wire into an input. `from_ref` is `<block uid>.<port>` in this file,
-- `<group>.<block id>.<port>` in another group, or `case` for a value a run
-- sets.
CREATE TABLE wire (
  to_block  TEXT NOT NULL,
  to_port   TEXT NOT NULL,
  from_ref  TEXT NOT NULL,
  PRIMARY KEY (to_block, to_port)
);

-- Where a group hangs: the block it mounts on, and the release mounted.
CREATE TABLE mount (
  block_uid  TEXT PRIMARY KEY,
  group_id   TEXT NOT NULL,
  release    TEXT NOT NULL DEFAULT ''
);

-- A requirement closed against an achieved value, and which way it binds:
-- '<=' the achieved value stays under the bound, '>=' it reaches it. Never
-- defaulted (AGENTS.md).
CREATE TABLE closure (
  uid           TEXT PRIMARY KEY,
  block_uid     TEXT NOT NULL,
  required_ref  TEXT NOT NULL,
  achieved_ref  TEXT NOT NULL,
  sense         TEXT NOT NULL CHECK (sense IN ('<=', '>=')),
  says          TEXT NOT NULL DEFAULT ''
);

-- A loop, declared on the smallest block that holds it: which blocks it runs
-- through, what must settle, how tightly, and how many passes before it is
-- refused by name.
CREATE TABLE loop (
  uid             TEXT PRIMARY KEY,
  block_uid       TEXT NOT NULL,
  members         TEXT NOT NULL,
  settles         TEXT NOT NULL,
  tolerance       TEXT NOT NULL,
  max_iterations  INTEGER NOT NULL
);

-- Evidence: a case is inputs, the answer expected, a tolerance and where the
-- answer came from. An expected value never comes from the code under test:
-- `self-snapshot` and `agent-generated` are refused (rule 4).
CREATE TABLE test_case (
  block_uid   TEXT NOT NULL,
  name        TEXT NOT NULL,
  inputs      TEXT NOT NULL,
  expected    TEXT NOT NULL,
  tolerance   TEXT NOT NULL DEFAULT '',
  provenance  TEXT NOT NULL CHECK (provenance IN ('independent-derivation',
                'published-source', 'independent-tool', 'physical-bound', 'authors-code',
                'self-snapshot', 'agent-generated')),
  source      TEXT NOT NULL DEFAULT '',
  PRIMARY KEY (block_uid, name)
);

-- Text, as Markdown: `scope` is 'file' or a block's uid; `kind` is what the
-- text is (explanation, theory, method, how_run, notes …).
CREATE TABLE text (
  scope  TEXT NOT NULL,
  kind   TEXT NOT NULL,
  body   TEXT NOT NULL,
  PRIMARY KEY (scope, kind)
);

-- Tables, as CSV, by the path a person reads them under.
CREATE TABLE tbl (
  scope  TEXT NOT NULL,
  path   TEXT NOT NULL,
  csv    TEXT NOT NULL,
  PRIMARY KEY (scope, path)
);

-- Files that are not tables: images, video, PDFs, code.
CREATE TABLE media (
  scope   TEXT NOT NULL,
  path    TEXT NOT NULL,
  media_type  TEXT NOT NULL,
  sha256  TEXT NOT NULL,
  bytes   BLOB NOT NULL,
  PRIMARY KEY (scope, path)
);

-- A signature, against exactly what it signed: the scope, its revision, and
-- the SHA-256 digest of its content, with the key that made it.
CREATE TABLE signature (
  scope       TEXT NOT NULL,
  revision    TEXT NOT NULL,
  signer      TEXT NOT NULL,
  public_key  TEXT NOT NULL,
  digest      TEXT NOT NULL,
  signature   TEXT NOT NULL,
  signed_at   TEXT NOT NULL,
  verdict     TEXT NOT NULL CHECK (verdict IN ('ok', 'changes')),
  note        TEXT NOT NULL DEFAULT ''
);

-- Every change, as it was made: who, when, to what, and what it said before.
CREATE TABLE change (
  at      TEXT NOT NULL,
  who     TEXT NOT NULL,
  scope   TEXT NOT NULL,
  what    TEXT NOT NULL,
  before  TEXT,
  after   TEXT
);

-- A request: a node engineer asking for their contract to change, or anyone
-- asking the developer for something the code cannot do yet (W14).
CREATE TABLE request (
  uid        TEXT PRIMARY KEY,
  kind       TEXT NOT NULL CHECK (kind IN ('contract', 'code')),
  scope      TEXT NOT NULL,
  raised_by  TEXT NOT NULL,
  at         TEXT NOT NULL,
  body       TEXT NOT NULL,
  status     TEXT NOT NULL DEFAULT 'open' CHECK (status IN ('open', 'accepted', 'declined')),
  answer     TEXT NOT NULL DEFAULT ''
);

-- An issue (W11): what is wrong, where, the evidence, whom it is for, and the
-- release that closed it.
CREATE TABLE issue (
  number        INTEGER PRIMARY KEY,
  group_id      TEXT NOT NULL,
  raised_by     TEXT NOT NULL,
  at            TEXT NOT NULL,
  place         TEXT NOT NULL,
  what          TEXT NOT NULL,
  evidence      TEXT NOT NULL DEFAULT '',
  addressed_to  TEXT NOT NULL,
  closed_by     TEXT NOT NULL DEFAULT ''
);

-- A comment on any part, kept in the file it was written on.
CREATE TABLE comment (
  scope     TEXT NOT NULL,
  section   TEXT NOT NULL DEFAULT '',
  author    TEXT NOT NULL,
  at        TEXT NOT NULL,
  body      TEXT NOT NULL,
  resolved  INTEGER NOT NULL DEFAULT 0
);

-- A key file: a person's private key, sealed by their passphrase (src/keys.rs).
CREATE TABLE locked_key (
  person      TEXT PRIMARY KEY,
  public_key  TEXT NOT NULL,
  salt        TEXT NOT NULL,
  nonce       TEXT NOT NULL,
  iterations  INTEGER NOT NULL,
  sealed      TEXT NOT NULL
);
