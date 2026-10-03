#!/usr/bin/env python3
"""The team's shared drive, kept by the pipeline rather than by hand.

    tools/drive.py pack --out target/drive          # the folder, built here
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
    groups/<group>/                      every group, as group-export writes it
      <group>.vgroup, nodes/*.vnode, releases/*.vleo
    groups/READY.csv                     what each group's own checks still ask
    groups/l3_solar/                     the solar worked example (groups/solar),
                                         in place of solar's plain export

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


def pack(out):
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
    files = [p for p in out.rglob("*") if p.is_file()]
    size = sum(p.stat().st_size for p in files)
    print("packed %d files, %.0f MB, at %s" % (len(files), size / 1e6, out))
    return out


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
    print("selftest: %s" % ("all as expected" if not bad else "%d FAILED" % bad))
    return 1 if bad else 0


def main():
    if "--selftest" in sys.argv[1:]:
        sys.exit(selftest())
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    sub = ap.add_subparsers(dest="what", required=True)
    p = sub.add_parser("pack", help="build the drive's folder from the repository")
    p.add_argument("--out", default=str(ROOT / "target" / "drive"))
    u = sub.add_parser("upload", help="mirror a packed folder into a Drive folder")
    u.add_argument("local")
    u.add_argument("--folder", required=True, help="the Drive folder's id, from its link")
    a = sub.add_parser("auth", help="once, on the owner's computer: print a refresh token")
    a.add_argument("--client-id", required=True)
    a.add_argument("--client-secret", required=True)
    args = ap.parse_args()
    if args.what == "pack":
        pack(args.out)
    elif args.what == "upload":
        upload(args.local, args.folder)
    else:
        auth(args.client_id, args.client_secret)


if __name__ == "__main__":
    main()
