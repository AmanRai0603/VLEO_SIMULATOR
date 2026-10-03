#!/usr/bin/env python3
"""The mock engine: the page, served against recorded answers, with no Rust.

A frontend developer works on web/ — pages, charts, animation, 3D — against
the contract (contract/README.md), not against a build of the engine. This
serves exactly what the real engine serves, from files:

  * the page itself — web/index.html, web/app.css, web/js/*, web/fonts/* — as it is on disk,
    so an edit shows on reload;
  * every /v1 JSON answer from the example contract/routes.toml records for it
    (contract/examples/*.json), picked by the request's parameters: the example
    whose parameters the request carries, or the route's first when none match;
  * node pages, reference-data files and the MATLAB parity files straight from
    the repository, where the engine reads them too.

What it cannot do, it says: a request with no recorded answer is a 404 naming
the route and how to record one. It computes nothing — a run on another node
answers with the recorded node's numbers — so every answer carries an
`X-VLEO-Mock` header naming the example it came from, /v1/version says
`"endpoint": "mock-engine"`, and the page shows a banner saying so.

    python3 tools/mock_engine.py [--port 7780]
    python3 tools/mock_engine.py --selftest

Standard library only.
"""
import glob
import json
import os
import sys
import tomllib
import urllib.parse
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CONTRACT = os.path.join(ROOT, "contract")

TYPES = {
    ".html": "text/html; charset=utf-8",
    ".css": "text/css; charset=utf-8",
    ".js": "text/javascript; charset=utf-8",
    ".csv": "text/csv; charset=utf-8",
    ".json": "application/json; charset=utf-8",
    ".gz": "application/gzip",
    ".woff2": "font/woff2",
}


def load_routes(root=CONTRACT):
    """Every route and its examples, as contract/routes.toml writes them."""
    with open(os.path.join(root, "routes.toml"), "rb") as f:
        doc = tomllib.load(f)
    out = []
    for r in doc.get("route", []):
        examples = []
        for e in r.get("example", []):
            examples.append({
                "name": e["name"],
                "path": e.get("path", r["path"]),
                "params": [tuple(p) for p in e.get("params", [])],
            })
        out.append({**r, "examples": examples})
    return out


def pick(routes, method, path, params):
    """The recorded example that answers this request, and why, or None.

    The example whose parameters the request carries wins — the one matching
    most of them; a placeholder (`@saved`, `@get:`) matches anything. With none
    matching, the route's first example answers: the page still draws, and the
    header says which answer it was given.
    """
    # GET and POST of one path answer the same question (the page POSTs its
    # runs, the recorded runs are mostly GETs), so both methods' examples are
    # candidates — this method's first.
    best = None
    for r in sorted(routes, key=lambda r: r["method"] != method):
        if r.get("answer") != "json":
            continue
        for e in r["examples"]:
            if e["path"] != path:
                continue
            score = 0
            ok = True
            for k, v in e["params"]:
                if v.startswith("@"):
                    continue
                if (k, v) in params:
                    score += 1
                elif any(pk == k for pk, _ in params):
                    ok = False
            if ok and (best is None or score > best[0]) and (r["method"] == method or score > 0):
                best = (score, e["name"])
    if best is not None:
        return best[1], "matched"
    for r in routes:
        if r["method"] == method and r["path"] == path and r["examples"]:
            return r["examples"][0]["name"], "nearest"
    return None


def node_page(node_id):
    """A node's page, as the real engine rendered it for the contract's example
    node. The engine renders a page from its sheet when it is opened; the mock
    reads no sheets, so it has the recorded one and no other."""
    if not node_id.replace("_", "").isalnum():
        return None
    p = os.path.join(ROOT, "contract", "examples", "fragment-%s.html" % node_id)
    return p if os.path.isfile(p) else None

def plain(name):
    return bool(name) and all(c.isalnum() or c in "_-." for c in name) and ".." not in name


def static(path):
    """A file the engine serves from disk, for a path — or None."""
    if path == "/":
        return os.path.join(ROOT, "web", "index.html")
    if path == "/app.css":
        return os.path.join(ROOT, "web", "app.css")
    if path.startswith("/js/") and plain(path[4:]):
        return os.path.join(ROOT, "web", "js", path[4:])
    if path.startswith("/fonts/") and plain(path[7:]):
        return os.path.join(ROOT, "web", "fonts", path[7:])
    if path.startswith("/v1/fragment/"):
        return node_page(path[len("/v1/fragment/"):])
    if path.startswith("/v1/parity/") and plain(path[len("/v1/parity/"):]):
        return os.path.join(ROOT, "matlab", "reference", path[len("/v1/parity/"):])
    if path.startswith("/v1/bundle/"):
        parts = path[len("/v1/bundle/"):].split("/", 1)
        if len(parts) == 2 and plain(parts[0]) and plain(parts[1]):
            versions = sorted(glob.glob(os.path.join(ROOT, "bundles", parts[0], "*")))
            if versions:
                return os.path.join(versions[-1], parts[1])
    if path in ("/method.wasm.gz",) or path.startswith("/web/"):
        name = path.rsplit("/", 1)[-1]
        if plain(name):
            return os.path.join(ROOT, "web", name)
    return None


def answer(routes, method, raw_path, body):
    """(status, content type, bytes, example or None) for one request."""
    parsed = urllib.parse.urlsplit(raw_path)
    path = urllib.parse.unquote(parsed.path)
    params = urllib.parse.parse_qsl(parsed.query if method == "GET" else body, keep_blank_values=True)
    if method == "GET":
        f = static(path)
        if f:
            if not os.path.isfile(f):
                return 404, TYPES[".html"], b"<p>not on disk</p>", None
            with open(f, "rb") as fh:
                return 200, TYPES.get(os.path.splitext(f)[1], "application/octet-stream"), fh.read(), None
    found = pick(routes, method, path, params)
    if found is None:
        msg = {
            "ok": False,
            "message": f"the mock engine has no recorded answer for {method} {path}. Add an example "
                       "to contract/routes.toml and record it: VLEO_CONTRACT_RECORD=1 cargo test -p "
                       "vleo-server --test the_contract_holds — or run the real engine.",
        }
        return 404, TYPES[".json"], json.dumps(msg).encode(), None
    name, how = found
    with open(os.path.join(CONTRACT, "examples", name + ".json"), encoding="utf-8") as f:
        data = json.load(f)
    if path == "/v1/version":
        # Said by the answer itself, so the page can show a banner: every
        # number it draws is a recorded example, not the design.
        data["endpoint"] = "mock-engine"
    return 200, TYPES[".json"], json.dumps(data, separators=(",", ":"), ensure_ascii=False).encode(), f"{name} ({how})"


class Handler(BaseHTTPRequestHandler):
    routes = []

    def _go(self, method):
        n = int(self.headers.get("Content-Length") or 0)
        body = self.rfile.read(n).decode("utf-8", "replace") if n else ""
        status, ctype, data, example = answer(self.routes, method, self.path, body)
        self.send_response(status)
        self.send_header("Content-Type", ctype)
        self.send_header("Content-Length", str(len(data)))
        self.send_header("Cache-Control", "no-store")
        if example:
            self.send_header("X-VLEO-Mock", example)
        self.end_headers()
        self.wfile.write(data)

    def do_GET(self):
        self._go("GET")

    def do_POST(self):
        self._go("POST")

    def log_message(self, *_):
        pass


def serve(port=7780, background=False):
    Handler.routes = load_routes()
    for p in range(port, port + 16):
        try:
            srv = ThreadingHTTPServer(("127.0.0.1", p), Handler)
            break
        except OSError:
            continue
    else:
        raise SystemExit(f"nothing in {port}..{port + 16} was free on 127.0.0.1")
    if background:
        import threading
        threading.Thread(target=srv.serve_forever, daemon=True).start()
        return srv, p
    print("the mock engine — recorded answers from contract/examples, no Rust", flush=True)
    print(f"  http://127.0.0.1:{p}", flush=True)
    try:
        srv.serve_forever()
    except KeyboardInterrupt:
        pass
    return srv, p


def selftest():
    routes = load_routes()
    fails = []

    def expect(cond, what):
        if not cond:
            fails.append(what)

    # Every JSON example the contract names is on disk, and parses.
    for r in routes:
        for e in r["examples"]:
            if r.get("answer") == "json":
                p = os.path.join(CONTRACT, "examples", e["name"] + ".json")
                try:
                    with open(p, encoding="utf-8") as f:
                        json.load(f)
                except (OSError, ValueError) as err:
                    fails.append(f"{p}: {err}")
    st, _, body, ex = answer(routes, "GET", "/v1/version", "")
    expect(st == 200 and json.loads(body)["endpoint"] == "mock-engine", "/v1/version does not say mock-engine")
    st, _, body, ex = answer(routes, "GET", "/v1/run?node=sw_ap_design&mode=branch&reuse=1", "")
    expect(st == 200 and ex.startswith("run-sw_ap_design"), f"a run of sw_ap_design picked {ex}")
    st, _, _, ex = answer(routes, "GET", "/v1/run?node=sw_ap_design&set=sw_ap_design%3A100", "")
    expect(ex and ex.startswith("run-refused"), f"the refused run picked {ex}")
    st, _, _, ex = answer(routes, "GET", "/v1/run?node=some_other_row", "")
    expect(st == 200 and ex.endswith("(nearest)"), f"a run of another row was not marked nearest: {ex}")
    st, _, _, ex = answer(routes, "POST", "/v1/run", "node=l3_solar_ach_01&mode=branch")
    expect(ex and ex.startswith("run-closure"), f"POST /v1/run picked {ex}")
    st, _, _, ex = answer(routes, "POST", "/v1/run", "node=sw_ap_design&mode=branch&reuse=1")
    expect(ex and ex.startswith("run-sw_ap_design"), f"a POSTed run of sw_ap_design picked {ex}")
    st, _, _, _ = answer(routes, "GET", "/v1/nothing-here", "")
    expect(st == 404, "an unknown route was answered")
    st, ctype, body, _ = answer(routes, "GET", "/v1/fragment/sw_ap_design", "")
    expect(st == 200 and b"sw_ap_design" in body and "html" in ctype, "the node page was not served")
    st, _, _, _ = answer(routes, "GET", "/v1/fragment/..%2F..%2Fetc", "")
    expect(st != 200, "a path out of the repository was served")
    st, _, body, _ = answer(routes, "GET", "/", "")
    expect(st == 200 and b"<script" in body, "the page itself was not served")
    st, _, _, _ = answer(routes, "GET", "/js/..%2F..%2FCargo.toml", "")
    expect(st != 200, "a module path out of web/js was served")
    if fails:
        print("mock_engine selftest FAILED:\n  " + "\n  ".join(fails))
        return 1
    print(f"mock_engine selftest: ok — {sum(len(r['examples']) for r in routes)} recorded example(s)")
    return 0


if __name__ == "__main__":
    args = sys.argv[1:]
    if "--selftest" in args:
        sys.exit(selftest())
    port = int(args[args.index("--port") + 1]) if "--port" in args else int(os.environ.get("VLEO_PORT", "7780"))
    serve(port)
