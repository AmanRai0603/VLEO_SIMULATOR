#!/usr/bin/env python3
"""The team's shared drive, kept by the pipeline rather than by hand.

    tools/drive.py pack --out target/drive          # the folder, built here
    tools/drive.py pack --sealed R.vleo --delivery DIR --zip   # ...with a group's
                                                    # sealed release and its delivery,
                                                    # as one zip to put in Drive by hand
    tools/drive.py pack --update --zip              # ...only what the repository
                                                    # owns, to replace on a release
    tools/drive.py upload target/drive --folder ID  # mirrored into Drive
    tools/drive.py auth --client-id ID --client-secret S   # once, on your computer
    tools/drive.py --selftest

The drive is where a team opens the design without the repository: each
group's structure, node files and releases (docs/GROUP_APPS.md), the two pages
that open them, the role guides, and design.vleo. Every file in it is written
from the repository, so it is built by one command and copied by another — a
drive filled by hand drifts from the release it claims to be within a week.

# What `pack` writes

    apps/group.html, apps/node.html      the group and node applications
    guides/<role>.html                   the three role guides
    design/design.vleo                   the tree, as the one file a kit carries
    readable/Groups.csv, Nodes.csv,      the released design, to read in any
      Interfaces.csv                     spreadsheet: each group, every live
                                         node, every value one group reads
                                         from another
    groups/<group>/                      every group, as group-export writes it
      <group>.vgroup, nodes/*.vnode
      releases/<group>-<version>.vleo    only a SEALED release, given by --sealed
    groups/READY.csv                     what each group's own checks still ask
    groups/l3_solar/                     the solar worked example (groups/solar),
                                         in place of solar's plain export
    deliveries/<group>-<version>/        a test application's record, given by
                                         --delivery: DELIVERY.toml, DELIVERY.md,
                                         group-test.csv — what the lead accepts

`releases/` holds sealed releases and nothing else. The export assembles an
unsealed release for every group; it is left out, because a file of that name
in the drive reads as the group's release, and an upload by hand beside the
lead's sealed one makes two files of one name. A lead assembles and seals
their own (docs/GROUP_APPS.md); the developer adds a sealed one with --sealed.

# By hand, without the sign-in

`pack --zip` writes the same folder as one zip, its six folders at the top.
The drive's owner unzips it and drags the six folders into the drive's folder:
that is the whole upload, and it needs no secrets.

Who owns what decides what a later zip replaces. apps/, guides/, design/ and
readable/ are the repository's: a release replaces them whole. groups/ is the
groups' once it is in the drive — the lead's structure, the authors' node
files, the sealed releases — so a later zip leaves it out (`--update`), and
only a new group's folder is added by hand. deliveries/ only grows: each test
application adds its own folder, and the lead adds their answer to it.
docs/DRIVE_START_HERE.md is the drive's START HERE page, kept as a Google Doc
beside the six folders.

# What `upload` does, and what it refuses to do

It mirrors the folder into the Drive folder it is given: a folder or file
missing there is created, a file whose bytes differ is replaced in place — the
same file, so every link to it still works — and a file whose bytes are the
same is left alone. It never deletes anything. A file in Drive that the pack no
longer writes is named in the report and kept, because a deletion nobody asked
for, in a folder people work in, is the one mistake this cannot undo.

It signs in as the drive's owner, with a refresh token from `auth`
(GDRIVE_CLIENT_ID, GDRIVE_CLIENT_SECRET, GDRIVE_REFRESH_TOKEN). A service
account cannot: it has no storage of its own, and a personal Drive refuses the
files it would create. docs/DRIVE_SETUP.md is the one-time setup.

Only the standard library: the pipeline runs this with nothing installed.
"""

import argparse
import hashlib
import http.server
import json
import os
import shutil
import subprocess
import sys
import tempfile
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

FOLDER = "application/vnd.google-apps.folder"
SCOPE = "https://www.googleapis.com/auth/drive"
TOKEN_URL = "https://oauth2.googleapis.com/token"
API = "https://www.googleapis.com/drive/v3"
UPLOAD = "https://www.googleapis.com/upload/drive/v3"

#: The role guides `xtask guides` writes, by file.
GUIDES = ["user.html", "maintainer.html", "developer.html"]


# ---------------------------------------------------------------------------
# pack


def run(cmd, cwd=ROOT):
    print("  $ " + " ".join(str(c) for c in cmd))
    subprocess.run([str(c) for c in cmd], cwd=cwd, check=True)


def pack(out, sealed=(), deliveries=(), zip_it=False, update=False):
    """Build the drive's folder at `out` from the repository as it stands."""
    out = Path(out).resolve()
    if out.exists():
        shutil.rmtree(out)
    (out / "apps").mkdir(parents=True)
    (out / "guides").mkdir()
    (out / "design").mkdir()
    print("1. the applications and the guides")
    for page in ["group.html", "node.html"]:
        shutil.copy2(ROOT / "web" / page, out / "apps" / page)
    for g in GUIDES:
        shutil.copy2(ROOT / "docs" / "roles" / g, out / "guides" / g)
    print("2. the design, as one file")
    run(["cargo", "run", "-q", "-p", "xtask", "--", "design", "--out", out / "design" / "design.vleo"])
    with tempfile.TemporaryDirectory(prefix="vleo-drive-") as tmp:
        tmp = Path(tmp)
        print("3. every group, written from the tree")
        run(["cargo", "run", "-q", "-p", "xtask", "--", "group-export", "--all", "--out", tmp / "groups"])
        run(["node", "tools/group_db.mjs", "--all", tmp / "groups", "--out", out / "groups"])
        print("4. the solar worked example, in place of solar's plain export")
        run(["node", "tools/group_db.mjs", "groups/solar", "--out", tmp / "solar"])
        solar = out / "groups" / "l3_solar"
        if solar.exists():
            shutil.rmtree(solar)
        shutil.copytree(tmp / "solar", solar)
        print("4b. the released design, to read in a spreadsheet")
        write_readable(out, tmp / "groups")
    print("5. releases: sealed ones only")
    dropped = place_releases(out, sealed)
    print("  %d unsealed assembl%s left out; %d sealed release(s) in" % (dropped, "y" if dropped == 1 else "ies", len(sealed)))
    if deliveries:
        print("6. the test applications' records")
        for d in place_deliveries(out, deliveries):
            print("  deliveries/" + d)
    if update:
        print("7. --update: groups/ left out, as the groups' own once it is in the drive")
        shutil.rmtree(out / "groups")
    files = [p for p in out.rglob("*") if p.is_file()]
    size = sum(p.stat().st_size for p in files)
    print("packed %d files, %.0f MB, at %s" % (len(files), size / 1e6, out))
    if zip_it:
        z = shutil.make_archive(str(out), "zip", root_dir=out, base_dir=".")
        tops = sorted(p.name for p in out.iterdir())
        print("zipped: %s (%.0f MB) — unzip it and drag its folders (%s) into the drive" % (z, Path(z).stat().st_size / 1e6, ", ".join(tops)))
    return out


def write_readable(out, export):
    """readable/: the released design as three CSV files, written from the
    same export as the groups' files, so the two never disagree about what the
    tree held. UTF-8 with a byte-order mark, which Excel needs to read the
    dashes and the symbols."""
    import csv
    import sqlite3

    home = Path(out) / "readable"
    home.mkdir(exist_ok=True)
    ready = {}
    rp = Path(out) / "groups" / "READY.csv"
    if rp.is_file():
        with open(rp, newline="", encoding="utf-8") as f:
            ready = {r["group"]: r for r in csv.DictReader(f)}
    with open(Path(export) / "GROUPS.csv", newline="", encoding="utf-8") as f:
        groups = list(csv.DictReader(f))
    asks = ["errors", "warnings", "empty_sections", "pseudocode_to_write", "results_to_supply", "values_to_decide", "defaults_to_give", "other"]
    with open(home / "Groups.csv", "w", newline="", encoding="utf-8-sig") as f:
        w = csv.writer(f)
        w.writerow(["group", "name", "layer", "owner", "nodes", "computed", "with_method"] + asks)
        for g in groups:
            r = ready.get(g["group"], {})
            w.writerow([g[k] for k in ["group", "name", "layer", "owner", "nodes", "computed", "with_method"]] + [r.get(a, "") for a in asks])
    with open(home / "Nodes.csv", "w", newline="", encoding="utf-8-sig") as f:
        w = csv.writer(f)
        cols = ["id", "question", "kind", "output", "unit", "lower", "upper", "value"]
        w.writerow(["group"] + cols)
        for g in groups:
            with open(Path(export) / g["group"] / "nodes.csv", newline="", encoding="utf-8") as n:
                for r in csv.DictReader(n):
                    w.writerow([g["group"]] + [r.get(c, "") for c in cols])
    c = sqlite3.connect("file:%s?mode=ro" % (Path(out) / "design" / "design.vleo"), uri=True)
    try:
        rows = c.execute(
            "select p.grp, p.node, r.label, r.kind, r.state, p.unit, p.version, p.crosses_to, p.read_by_grp, p.read_by "
            "from published p left join row r on r.id = p.node order by p.grp, p.node, p.read_by"
        ).fetchall()
    finally:
        c.close()
    with open(home / "Interfaces.csv", "w", newline="", encoding="utf-8-sig") as f:
        w = csv.writer(f)
        w.writerow(["group", "node", "label", "kind", "state", "unit", "version", "crosses_to", "read_by_group", "read_by_node"])
        w.writerows(rows)


def release_meta(path):
    """What a database file says of itself: its meta table, as a dict."""
    import sqlite3

    try:
        c = sqlite3.connect("file:%s?mode=ro" % Path(path).resolve(), uri=True)
        try:
            return dict(c.execute("select key, value from meta"))
        finally:
            c.close()
    except sqlite3.Error as e:
        raise SystemExit("%s is not a group database file: %s" % (path, e))


def place_releases(out, sealed):
    """Leave out every unsealed release the export assembled, and put each
    sealed one given in its group's releases/. Returns how many were left out."""
    dropped = 0
    for f in sorted((Path(out) / "groups").glob("*/releases/*.vleo")):
        if not release_meta(f).get("sealed"):
            f.unlink()
            dropped += 1
    for d in sorted((Path(out) / "groups").glob("*/releases")):
        if not any(d.iterdir()):
            d.rmdir()
    for r in sealed:
        m = release_meta(r)
        g, v = m.get("group_id", ""), m.get("version", "")
        if not m.get("sealed") or not m.get("fingerprint"):
            raise SystemExit("%s is not sealed: only a sealed release goes in releases/" % r)
        home = Path(out) / "groups" / g
        if not g or not home.is_dir():
            raise SystemExit("%s is a release of %r, which is not a group in this design" % (r, g))
        (home / "releases").mkdir(exist_ok=True)
        shutil.copy2(r, home / "releases" / ("%s-%s.vleo" % (g, v)))
    return dropped


#: What a delivery's folder in the drive holds, from the test application.
DELIVERY_FILES = ["DELIVERY.toml", "DELIVERY.md", "group-test.csv"]


def place_deliveries(out, dirs):
    """Copy each test application's record into deliveries/<group>-<version>/.
    Refused when the drive does not also hold the sealed release it was built
    from: the lead accepts a delivery against that release, and nothing else."""
    placed = []
    for d in dirs:
        d = Path(d)
        toml = d / "DELIVERY.toml"
        if not toml.is_file():
            raise SystemExit("%s has no DELIVERY.toml: give the folder `xtask group-deliver` wrote" % d)
        rec = {}
        for line in toml.read_text().splitlines():
            k, sep, v = line.partition("=")
            if sep and not line.lstrip().startswith("#"):
                rec[k.strip()] = v.strip().strip('"')
        g, v, fp = rec.get("group", ""), rec.get("version", ""), rec.get("fingerprint", "")
        rel = Path(out) / "groups" / g / "releases" / ("%s-%s.vleo" % (g, v))
        if not rel.is_file() or release_meta(rel).get("fingerprint") != fp:
            raise SystemExit(
                "the delivery of %s %s was built from the sealed release with fingerprint %s…, "
                "and the drive does not hold it: give that release with --sealed" % (g, v, fp[:12])
            )
        home = Path(out) / "deliveries" / ("%s-%s" % (g, v))
        home.mkdir(parents=True, exist_ok=True)
        for name in DELIVERY_FILES:
            if (d / name).is_file():
                shutil.copy2(d / name, home / name)
        placed.append("%s-%s" % (g, v))
    return placed


# ---------------------------------------------------------------------------
# the Drive API, as much of it as a mirror needs


class Drive:
    """Drive v3 over HTTPS, signed in with the owner's refresh token."""

    def __init__(self, client_id, client_secret, refresh_token):
        body = urllib.parse.urlencode(
            {
                "client_id": client_id,
                "client_secret": client_secret,
                "refresh_token": refresh_token,
                "grant_type": "refresh_token",
            }
        ).encode()
        tok = json.loads(self._send(urllib.request.Request(TOKEN_URL, data=body)))
        self.auth = {"Authorization": "Bearer " + tok["access_token"]}

    @staticmethod
    def _send(req):
        try:
            with urllib.request.urlopen(req, timeout=300) as r:
                return r.read()
        except urllib.error.HTTPError as e:
            raise SystemExit("Drive refused %s %s: %s %s" % (req.get_method(), req.full_url.split("?")[0], e.code, e.read().decode(errors="replace")[:400]))

    def _json(self, method, url, body=None, headers=None):
        h = dict(self.auth)
        h.update(headers or {})
        data = None
        if body is not None:
            data = json.dumps(body).encode()
            h["Content-Type"] = "application/json; charset=UTF-8"
        return json.loads(self._send(urllib.request.Request(url, data=data, headers=h, method=method)) or b"{}")

    def children(self, parent):
        """Every file and folder directly in `parent`, by name."""
        out, page = {}, None
        while True:
            q = {
                "q": "'%s' in parents and trashed = false" % parent,
                "fields": "nextPageToken, files(id, name, mimeType, md5Checksum)",
                "pageSize": "1000",
                "supportsAllDrives": "true",
                "includeItemsFromAllDrives": "true",
            }
            if page:
                q["pageToken"] = page
            r = self._json("GET", API + "/files?" + urllib.parse.urlencode(q))
            for f in r.get("files", []):
                out.setdefault(f["name"], f)
            page = r.get("nextPageToken")
            if not page:
                return out

    def folder(self, parent, name):
        r = self._json("POST", API + "/files?supportsAllDrives=true", {"name": name, "mimeType": FOLDER, "parents": [parent]})
        return {"id": r["id"], "name": name, "mimeType": FOLDER}

    def put(self, parent, name, path, existing=None):
        """Create the file, or replace the bytes of `existing` in place."""
        if existing:
            url = UPLOAD + "/files/%s?uploadType=resumable&supportsAllDrives=true" % existing
            meta, method = {}, "PATCH"
        else:
            url = UPLOAD + "/files?uploadType=resumable&supportsAllDrives=true"
            meta, method = {"name": name, "parents": [parent]}, "POST"
        h = dict(self.auth)
        h["Content-Type"] = "application/json; charset=UTF-8"
        req = urllib.request.Request(url, data=json.dumps(meta).encode(), headers=h, method=method)
        try:
            with urllib.request.urlopen(req, timeout=300) as r:
                where = r.headers["Location"]
        except urllib.error.HTTPError as e:
            raise SystemExit("Drive refused %s: %s %s" % (name, e.code, e.read().decode(errors="replace")[:400]))
        data = Path(path).read_bytes()
        self._send(urllib.request.Request(where, data=data, headers={"Content-Type": "application/octet-stream"}, method="PUT"))


def md5(path):
    h = hashlib.md5()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def mirror(drive, local, parent, report, rel=""):
    """Copy the folder `local` into the Drive folder `parent`, as above."""
    there = drive.children(parent)
    here = sorted(Path(local).iterdir(), key=lambda p: (p.is_file(), p.name))
    names = {p.name for p in here}
    for p in here:
        at = (rel + "/" + p.name).lstrip("/")
        have = there.get(p.name)
        if p.is_dir():
            if have and have["mimeType"] != FOLDER:
                raise SystemExit("%s is a file in Drive and a folder here; move it aside first" % at)
            if not have:
                have = drive.folder(parent, p.name)
                report["folders"].append(at)
            mirror(drive, p, have["id"], report, at)
        else:
            if have and have["mimeType"] == FOLDER:
                raise SystemExit("%s is a folder in Drive and a file here; move it aside first" % at)
            if have and have.get("md5Checksum") == md5(p):
                report["same"].append(at)
                continue
            drive.put(parent, p.name, p, have["id"] if have else None)
            report["replaced" if have else "created"].append(at)
    for name in sorted(set(there) - names):
        report["kept"].append((rel + "/" + name).lstrip("/"))


def upload(local, folder):
    need = ["GDRIVE_CLIENT_ID", "GDRIVE_CLIENT_SECRET", "GDRIVE_REFRESH_TOKEN"]
    missing = [n for n in need if not os.environ.get(n)]
    if missing:
        raise SystemExit("cannot sign in to Drive: %s not set (docs/DRIVE_SETUP.md)" % ", ".join(missing))
    drive = Drive(*(os.environ[n] for n in need))
    report = {"folders": [], "created": [], "replaced": [], "same": [], "kept": []}
    mirror(drive, Path(local), folder, report)
    say(report)
    return report


def say(report):
    print(
        "drive: %d created, %d replaced in place, %d unchanged, %d folders made"
        % (len(report["created"]), len(report["replaced"]), len(report["same"]), len(report["folders"]))
    )
    for k in ["created", "replaced"]:
        for f in report[k]:
            print("  %-9s %s" % (k, f))
    if report["kept"]:
        print("in Drive and no longer written by the pack — kept, not deleted:")
        for f in report["kept"]:
            print("  kept      %s" % f)


# ---------------------------------------------------------------------------
# auth, once, on the owner's computer


def auth(client_id, client_secret):
    """Sign in as the drive's owner in a browser and print the refresh token."""
    got = {}

    class Back(http.server.BaseHTTPRequestHandler):
        def do_GET(self):
            q = urllib.parse.parse_qs(urllib.parse.urlparse(self.path).query)
            got.update({k: v[0] for k, v in q.items()})
            self.send_response(200)
            self.send_header("Content-Type", "text/plain; charset=utf-8")
            self.end_headers()
            self.wfile.write("Signed in. Go back to the terminal.".encode())

        def log_message(self, *a):
            pass

    srv = http.server.HTTPServer(("127.0.0.1", 0), Back)
    redirect = "http://127.0.0.1:%d" % srv.server_port
    url = "https://accounts.google.com/o/oauth2/v2/auth?" + urllib.parse.urlencode(
        {
            "client_id": client_id,
            "redirect_uri": redirect,
            "response_type": "code",
            "scope": SCOPE,
            "access_type": "offline",
            "prompt": "consent",
        }
    )
    print("Open this in a browser signed in as the drive's owner, and allow it:\n\n  %s\n" % url)
    srv.handle_request()
    if "code" not in got:
        raise SystemExit("no sign-in came back: %s" % got.get("error", "nothing"))
    body = urllib.parse.urlencode(
        {
            "code": got["code"],
            "client_id": client_id,
            "client_secret": client_secret,
            "redirect_uri": redirect,
            "grant_type": "authorization_code",
        }
    ).encode()
    tok = json.loads(Drive._send(urllib.request.Request(TOKEN_URL, data=body)))
    if "refresh_token" not in tok:
        raise SystemExit("Google gave no refresh token; remove the app's access at myaccount.google.com/permissions and run this again")
    print("GDRIVE_REFRESH_TOKEN, for the repository's secrets — keep it like a password:\n\n  %s\n" % tok["refresh_token"])


# ---------------------------------------------------------------------------
# selftest


class Fake:
    """Drive in memory: the same four calls, and a count of uploads."""

    def __init__(self):
        self.items = {"root": {"id": "root", "mimeType": FOLDER, "parent": None, "name": ""}}
        self.uploads = 0

    def _new(self, parent, name, mime, data=None):
        i = "id%d" % len(self.items)
        self.items[i] = {"id": i, "name": name, "mimeType": mime, "parent": parent, "data": data}
        return i

    def children(self, parent):
        out = {}
        for f in self.items.values():
            if f["parent"] == parent:
                d = dict(f)
                if f.get("data") is not None:
                    d["md5Checksum"] = hashlib.md5(f["data"]).hexdigest()
                out.setdefault(f["name"], d)
        return out

    def folder(self, parent, name):
        return {"id": self._new(parent, name, FOLDER), "name": name, "mimeType": FOLDER}

    def put(self, parent, name, path, existing=None):
        self.uploads += 1
        data = Path(path).read_bytes()
        if existing:
            self.items[existing]["data"] = data
        else:
            self._new(parent, name, "application/octet-stream", data)


def selftest():
    bad = 0

    def check(what, ok):
        nonlocal bad
        print("  %s  %s" % ("ok  " if ok else "FAIL", what))
        bad += 0 if ok else 1

    def fresh():
        return {"folders": [], "created": [], "replaced": [], "same": [], "kept": []}

    with tempfile.TemporaryDirectory() as t:
        t = Path(t)
        (t / "groups" / "solar" / "nodes").mkdir(parents=True)
        (t / "apps").mkdir()
        (t / "apps" / "group.html").write_text("page")
        (t / "groups" / "solar" / "solar.vgroup").write_bytes(b"structure")
        (t / "groups" / "solar" / "nodes" / "a.vnode").write_bytes(b"node a")
        d = Fake()
        r = fresh()
        mirror(d, t, "root", r)
        check("a first upload creates every folder and file", len(r["created"]) == 3 and len(r["folders"]) == 4)
        r = fresh()
        mirror(d, t, "root", r)
        check("a second upload of the same bytes uploads nothing", d.uploads == 3 and len(r["same"]) == 3)
        a = [f for f in d.items.values() if f["name"] == "a.vnode"][0]
        (t / "groups" / "solar" / "nodes" / "a.vnode").write_bytes(b"node a, changed")
        r = fresh()
        mirror(d, t, "root", r)
        a2 = [f for f in d.items.values() if f["name"] == "a.vnode"]
        check("a changed file is replaced in place, as the same file", r["replaced"] == ["groups/solar/nodes/a.vnode"] and len(a2) == 1 and a2[0]["id"] == a["id"] and a2[0]["data"] == b"node a, changed")
        d._new("root", "notes by a person.txt", "text/plain", b"mine")
        (t / "apps" / "group.html").unlink()
        r = fresh()
        mirror(d, t, "root", r)
        names = {f["name"] for f in d.items.values()}
        check("nothing in Drive is deleted, and what the pack no longer writes is named", "notes by a person.txt" in names and "group.html" in names and sorted(r["kept"]) == ["apps/group.html", "notes by a person.txt"])
        d2 = Fake()
        d2._new("root", "apps", "text/plain", b"x")
        try:
            mirror(d2, t, "root", fresh())
            check("a file where a folder belongs is refused, not overwritten", False)
        except SystemExit as e:
            check("a file where a folder belongs is refused, not overwritten", "apps" in str(e))
        os.environ.pop("GDRIVE_REFRESH_TOKEN", None)
        try:
            upload(t, "root")
            check("an upload with no sign-in stops and names what is missing", False)
        except SystemExit as e:
            check("an upload with no sign-in stops and names what is missing", "GDRIVE_REFRESH_TOKEN" in str(e))
    with tempfile.TemporaryDirectory() as t:
        import sqlite3

        t = Path(t)

        def db(path, **meta):
            path.parent.mkdir(parents=True, exist_ok=True)
            c = sqlite3.connect(path)
            c.execute("create table meta (key text primary key, value text not null)")
            c.executemany("insert into meta values (?, ?)", meta.items())
            c.commit()
            c.close()
            return path

        out = t / "drive"
        db(out / "groups" / "solar" / "releases" / "solar-1.0.vleo", group_id="solar", version="1.0", sealed="")
        db(out / "groups" / "orbit" / "releases" / "orbit-0.1.vleo", group_id="orbit", version="0.1", sealed="")
        good = db(t / "in" / "solar-1.1.vleo", group_id="solar", version="1.1", sealed="2026-10-03", fingerprint="ab" * 32)
        dropped = place_releases(out, [good])
        check(
            "releases/ keeps sealed releases only: the export's unsealed assemblies are left out",
            dropped == 2 and sorted(p.name for p in out.rglob("*.vleo")) == ["solar-1.1.vleo"] and not (out / "groups" / "orbit" / "releases").exists(),
        )
        for bad_release, why in [
            (db(t / "in" / "u.vleo", group_id="solar", version="1.2", sealed=""), "not sealed"),
            (db(t / "in" / "x.vleo", group_id="nobody", version="1", sealed="x", fingerprint="f"), "not a group"),
        ]:
            try:
                place_releases(out, [bad_release])
                check("a release that is %s is refused" % why, False)
            except SystemExit as e:
                check("a release that is %s is refused" % why, why in str(e))
        dl = t / "dl"
        dl.mkdir()
        (dl / "DELIVERY.toml").write_text('group = "solar"\nversion = "1.1"\nfingerprint = "%s"\n' % ("ab" * 32))
        (dl / "group-test.csv").write_text("node,held\n")
        placed = place_deliveries(out, [dl])
        home = out / "deliveries" / "solar-1.1"
        check("a delivery goes beside the sealed release it was built from", placed == ["solar-1.1"] and (home / "DELIVERY.toml").is_file() and (home / "group-test.csv").is_file())
        (dl / "DELIVERY.toml").write_text('group = "solar"\nversion = "1.1"\nfingerprint = "%s"\n' % ("cd" * 32))
        try:
            place_deliveries(out, [dl])
            check("a delivery whose sealed release the drive does not hold is refused", False)
        except SystemExit as e:
            check("a delivery whose sealed release the drive does not hold is refused", "--sealed" in str(e))
    with tempfile.TemporaryDirectory() as t:
        import csv
        import sqlite3

        t = Path(t)
        ex, out = t / "export", t / "drive"
        for g, rows in {"solar": [("sw_a", "How hot?"), ("sw_b", "How long?")], "orbit": [("orb_h", "How high?")]}.items():
            (ex / g).mkdir(parents=True)
            (ex / g / "nodes.csv").write_text("id,question,kind,output,unit,lower,upper,value\n" + "".join("%s,%s,computed,x,1,0,1,\n" % r for r in rows))
        (ex / "GROUPS.csv").write_text("group,name,layer,owner,nodes,computed,with_method,files\nsolar,Solar,3,env,2,2,2,9\norbit,Orbit,3,env,1,1,0,4\n")
        (out / "groups").mkdir(parents=True)
        (out / "groups" / "READY.csv").write_text("group,nodes,errors\nsolar,2,3\n")
        (out / "design").mkdir()
        c = sqlite3.connect(out / "design" / "design.vleo")
        c.execute("create table row (id text, label text, kind text, state text)")
        c.execute("create table published (grp text, node text, unit text, version int, crosses_to text, read_by_grp text, read_by text)")
        c.execute("insert into row values ('sw_a', 'Hot level', 'computed', 'published')")
        c.execute("insert into published values ('solar', 'sw_a', '1', 1, '', 'orbit', 'orb_h')")
        c.commit()
        c.close()
        write_readable(out, ex)

        def rows(name):
            with open(out / "readable" / name, newline="", encoding="utf-8-sig") as f:
                return list(csv.DictReader(f))

        n, g, i = rows("Nodes.csv"), rows("Groups.csv"), rows("Interfaces.csv")
        check("readable/Nodes.csv holds every group's nodes, each named with its group", [(r["group"], r["id"]) for r in n] == [("solar", "sw_a"), ("solar", "sw_b"), ("orbit", "orb_h")])
        check("readable/Groups.csv joins each group to what its own checks still ask", [(r["group"], r["errors"]) for r in g] == [("solar", "3"), ("orbit", "")])
        check("readable/Interfaces.csv names who reads each published value", [(r["node"], r["label"], r["read_by_node"]) for r in i] == [("sw_a", "Hot level", "orb_h")])
    print("selftest: %s" % ("all as expected" if not bad else "%d FAILED" % bad))
    return 1 if bad else 0


def main():
    if "--selftest" in sys.argv[1:]:
        sys.exit(selftest())
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    sub = ap.add_subparsers(dest="what", required=True)
    p = sub.add_parser("pack", help="build the drive's folder from the repository")
    p.add_argument("--out", default=str(ROOT / "target" / "drive"))
    p.add_argument("--sealed", nargs="*", default=[], help="sealed releases (.vleo) to put in their groups' releases/")
    p.add_argument("--delivery", nargs="*", default=[], help="folders `xtask group-deliver` wrote, for deliveries/")
    p.add_argument("--zip", action="store_true", help="also write the folder as one zip, to put in the drive by hand")
    p.add_argument("--update", action="store_true", help="leave groups/ out: the groups' own once it is in the drive")
    u = sub.add_parser("upload", help="mirror a packed folder into a Drive folder")
    u.add_argument("local")
    u.add_argument("--folder", required=True, help="the Drive folder's id, from its link")
    a = sub.add_parser("auth", help="once, on the owner's computer: print a refresh token")
    a.add_argument("--client-id", required=True)
    a.add_argument("--client-secret", required=True)
    args = ap.parse_args()
    if args.what == "pack":
        pack(args.out, args.sealed, args.delivery, args.zip, args.update)
    elif args.what == "upload":
        upload(args.local, args.folder)
    else:
        auth(args.client_id, args.client_secret)


if __name__ == "__main__":
    main()
