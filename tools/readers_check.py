#!/usr/bin/env python3
"""The readers' folder, opened from a file, in a real browser.

    cargo run -p xtask -- readers            # builds target/readers
    tools/readers_check.py                   # target/readers
    tools/readers_check.py --folder <dir> --answers <file.tsv>
    tools/readers_check.py --selftest

`xtask readers` writes a folder of pages a team reads with no tool running —
from a shared drive, opened as files. Its own last step proves every link and
asset a page names is in the folder. That is not the same as the pages
WORKING, and the part most worth proving is the engine the page carries
(crates/vleo-kernel-wasm): a lesson's widget there is answered by it, and a
reader cannot tell an engine that answers wrongly from one that answers right.

So this opens the folder the way a reader does — `file://`, in Chromium — and
checks three things:

  1 · the pages open    the index and a sample of row pages load with no page
                        error; a row page's tabs switch; a link to another row
                        goes to a page that is there
  2 · the engine runs   loaded from assets/kernel.js exactly as the page loads
                        it: it asks the page for nothing (no imports), says what
                        it is, answers every computed row it can on the declared
                        values, refuses a supplied computed row, refuses a row
                        that reads reference data AS data-missing, and sweeps
  3 · a widget answers  a copy of a real row page, with a lesson of this
                        check's own placed in it — never in the tree — drawn by
                        the folder's bundled component library: the widget
                        starts where the engine says the row stands, moves when
                        its slider moves, draws the sweep, and names a row that
                        needs reference data with the page's own wording

With --answers, every answer from 2 is written as `<row>\\t<the engine's JSON>`,
and `cargo test` in crates/vleo-kernel-wasm, given the file in
VLEO_BROWSER_ANSWERS, asks the same engine built natively the same questions:
the page's numbers must be the tool's, byte for byte, because the maths is
portable (pmath) and so must not move between the two builds.

Needs Chromium through Playwright, as tools/panel_check.py does.
"""

import argparse
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(Path(__file__).resolve().parent))
from panel_check import chromium_path  # noqa: E402 — one way to find the browser

# The engine loader, as web/js/readers.js has it. The check's own copy, so it
# can ask the engine directly; the widget in check 3 goes through the page's.
ENGINE_JS = r"""
async (rows) => {
  await new Promise((ok, no) => {
    const s = document.createElement('script');
    s.src = 'assets/kernel.js'; s.onload = ok; s.onerror = () => no(new Error('assets/kernel.js did not load'));
    document.head.appendChild(s);
  });
  const unpack = async b64 => {
    const bin = atob(b64 || '');
    const gz = new Uint8Array(bin.length);
    for (let k = 0; k < bin.length; k++) gz[k] = bin.charCodeAt(k);
    return new Uint8Array(await new Response(new Blob([gz]).stream().pipeThrough(new DecompressionStream('gzip'))).arrayBuffer());
  };
  const mod = await WebAssembly.compile(await unpack(window.VLEO_KERNEL));
  const imports = WebAssembly.Module.imports(mod).map(i => i.module + '.' + i.name);
  const v = (await WebAssembly.instantiate(mod, {})).exports;
  const read = at => new TextDecoder().decode(new Uint8Array(v.memory.buffer, at, v.vleo_out_len()));
  const callBytes = (fn, enc) => {
    const p = v.vleo_alloc(enc.length);
    new Uint8Array(v.memory.buffer, p, enc.length).set(enc);
    return read(v[fn](p, enc.length));
  };
  const call = (fn, text) => callBytes(fn, new TextEncoder().encode(text));
  // Before the design is open the engine runs nothing; then the design the
  // folder carries is opened, as the page opens it.
  const before = call('vleo_run', 'node sw_ap_design_long\n');
  const opened = callBytes('vleo_open', await unpack(window.VLEO_DESIGN));
  const runs = {};
  for (const id of rows) runs[id] = call('vleo_run', 'node ' + id + '\n');
  return {
    imports,
    before,
    opened,
    identity: read(v.vleo_identity()),
    runs,
    supplied: call('vleo_run', 'node sw_ap_design_long\nset sw_ap_design_long 3\n'),
    sweep: call('vleo_sweep', 'node sw_ap_design_long\nover sw_ap_central_expectation\nfrom 5\nto 35\npoints 7\n'),
  };
}
"""

# The widget's rows and lesson, the check's own. sw_ap_design_long answers from
# declared values alone; sw_ap_design reads the solar-weather bundle, so the
# page must refuse it and say why in its own words.
PROBE_ROWS = [
    {"id": "sw_ap_central_expectation", "label": "Expected Ap over the mission window", "unit": "", "lo": 1.0, "hi": 40.0, "factor": 1.0},
    {"id": "sw_ap_design_long", "label": "Sustained Ap to design to", "unit": "", "lo": 0.0, "hi": 300.0, "factor": 1.0},
    {"id": "sw_ap_design", "label": "Ap to design to", "unit": "", "lo": 0.0, "hi": 400.0, "factor": 1.0},
]
PROBE_LESSON = {
    "node": "sw_ap_design_long",
    "title": "tools/readers_check.py — a lesson of the check's own, never in the tree",
    "by": "tools/readers_check.py",
    "answer": "Written by the check, to drive the widget; not a lesson.",
    "kind": "explanation",
    "stations": [{"kind": "simply", "title": "", "text": "Test text.", "claim": "illustrative", "source": ""}],
    "equations": [],
    "widgets": [{
        "title": "Try it",
        "inputs": ["sw_ap_central_expectation"],
        "outputs": ["sw_ap_design_long", "sw_ap_design"],
        "sweep": "sw_ap_central_expectation",
    }],
    "checks": [],
    "references": [],
}


def computed_rows():
    path = ROOT / "generated" / "index.json"
    if not path.is_file():
        raise SystemExit("no generated/index.json, which says which rows compute — "
                         "build it with: cargo run -p xtask -- assemble")
    idx = json.loads(path.read_text(encoding="utf-8"))
    return [r["id"] for r in idx["rows"] if r.get("kind") == "computed" and r.get("state") == "published"]


def engine_findings(e, asked):
    """What is wrong with the engine's answers, from check 2."""
    bad = []
    if e.get("imports"):
        bad.append("the engine asks the page for %s — a page opened from a file has nothing to give" % e["imports"])
    try:
        ident = json.loads(e.get("identity") or "")
        if not ident.get("kernel"):
            bad.append("the engine does not say which kernel it is: %s" % e.get("identity"))
        if not ident.get("graph"):
            bad.append("the engine does not say which graph it runs: %s" % e.get("identity"))
    except ValueError:
        bad.append("the engine's identity is not JSON: %r" % (e.get("identity") or "")[:120])
    # Before the design the folder carries is open, the engine runs nothing;
    # then it opens, every row of it.
    try:
        before = json.loads(e.get("before") or "")
        if before.get("ok") or "no design is open" not in (before.get("message") or ""):
            bad.append("the engine ran before a design was open: %s" % (e.get("before") or "")[:200])
    except ValueError:
        bad.append("the engine's answer before a design was open is not JSON")
    try:
        opened = json.loads(e.get("opened") or "")
        if not opened.get("ok") or not opened.get("rows"):
            bad.append("the design the folder carries did not open: %s" % (e.get("opened") or "")[:300])
    except ValueError:
        bad.append("the engine's answer to opening the design is not JSON")
    answered, missing_data = 0, 0
    for rid in asked:
        text = (e.get("runs") or {}).get(rid)
        if text is None:
            bad.append("%s was asked and the engine gave nothing" % rid)
            continue
        try:
            r = json.loads(text)
        except ValueError:
            bad.append("%s: the answer is not JSON" % rid)
            continue
        if not r.get("ok"):
            bad.append("%s: the engine failed rather than answering or refusing: %s" % (rid, r.get("message")))
            continue
        if any(v.get("id") == rid for v in r.get("values", [])):
            answered += 1
        blocked = [b for b in r.get("blocked", []) if b.get("id") == rid]
        if blocked and blocked[0].get("kind") == "data-missing":
            missing_data += 1
    if answered == 0:
        bad.append("no computed row answered in the page")
    if missing_data == 0:
        bad.append("no row refused for want of reference data — the page carries none, so some must")
    try:
        s = json.loads(e.get("supplied") or "")
        if s.get("ok"):
            bad.append("the engine took a value for a computed row: %s" % e.get("supplied"))
    except ValueError:
        bad.append("the refusal of a supplied computed row is not JSON")
    try:
        w = json.loads(e.get("sweep") or "")
        ys = (w.get("figure") or {}).get("series", [{}])[0].get("y", [])
        if not w.get("ok") or len(ys) != 7 or any(y is None for y in ys):
            bad.append("the sweep did not answer 7 of 7 points: %s" % (e.get("sweep") or "")[:200])
        elif len(set(ys)) < 2:
            bad.append("the sweep answered the same value at every point: %s" % ys)
    except ValueError:
        bad.append("the sweep's answer is not JSON")
    return bad, answered


def probe_page(folder):
    """A copy of a real row page with the check's lesson placed in it, beside
    the others so every relative link stays as it is. Returns its path."""
    src = folder / "rows" / "sw_ap_design_long.html"
    page = src.read_text(encoding="utf-8")
    seg = ('<section class="seg" data-seg="lesson"><h3 class="seg-h">the lesson</h3>'
           '<div id="row-lesson"></div></section>\n</main>')
    blocks = ('<script type="application/json" id="vleo-rows">%s</script>\n'
              '<script type="application/json" id="vleo-lesson-json">%s</script>\n'
              '<script src="../assets/kernel.js"></script>\n'
              '<script src="../assets/vleo.js"></script>') % (
        json.dumps(PROBE_ROWS).replace("<", "\\u003c"), json.dumps(PROBE_LESSON).replace("<", "\\u003c"))
    if "</main>" not in page or '<script src="../assets/vleo.js"></script>' not in page:
        raise SystemExit("rows/sw_ap_design_long.html is not the page this check knows how to extend")
    page = page.replace("</main>", seg, 1).replace('<script src="../assets/vleo.js"></script>', blocks, 1)
    out = folder / "rows" / "_readers_check.html"
    out.write_text(page, encoding="utf-8")
    return out


def run(folder, answers_out, chromium):
    from playwright.sync_api import sync_playwright

    found = []
    rows = computed_rows()
    probe = probe_page(folder)
    try:
        with sync_playwright() as pw:
            kw = {"executable_path": chromium} if chromium else {}
            b = pw.chromium.launch(**kw)
            pg = b.new_page(viewport={"width": 1200, "height": 1000})
            errs = []
            pg.on("pageerror", lambda e: errs.append(str(e)))

            # 1 · the pages open
            pg.goto((folder / "index.html").as_uri())
            listed = pg.locator(".rd-rows a").count()
            on_disk = len([p for p in (folder / "rows").glob("*.html") if not p.name.startswith("_")])
            if listed != on_disk:
                found.append(("1 pages", "the index lists %d rows and the folder holds %d" % (listed, on_disk)))
            sample = sorted(p.name for p in (folder / "rows").glob("*.html") if not p.name.startswith("_"))
            sample = sample[:: max(1, len(sample) // 25)] + ["sw_ap_design.html"]
            for name in sample:
                pg.goto((folder / "rows" / name).as_uri())
                if not pg.locator("article.node").count():
                    found.append(("1 pages", "%s does not carry the row's generated page" % name))
                tabs = pg.locator(".tabs .tab")
                if tabs.count() > 1:
                    tabs.nth(1).click()
                    if "sel" not in (tabs.nth(1).get_attribute("class") or ""):
                        found.append(("1 pages", "%s: a tab did not switch" % name))
            go = pg.locator("[data-goto]:visible")
            if go.count():
                target = go.first.get_attribute("data-goto")
                go.first.click()
                pg.wait_for_load_state()
                if not pg.url.endswith("/rows/%s.html" % target):
                    found.append(("1 pages", "a link to %s went to %s" % (target, pg.url)))
            if errs:
                found.append(("1 pages", "page errors: %s" % errs[:3]))
                errs.clear()

            # 2 · the engine runs
            pg.goto((folder / "index.html").as_uri())
            e = pg.evaluate(ENGINE_JS, rows)
            bad, answered = engine_findings(e, rows)
            found += [("2 engine", x) for x in bad]
            if answers_out:
                with open(answers_out, "w", encoding="utf-8") as f:
                    for rid in rows:
                        f.write("%s\t%s\n" % (rid, e["runs"][rid]))
                    f.write("%s\t%s\n" % ("@supplied", e["supplied"]))
                    f.write("%s\t%s\n" % ("@sweep", e["sweep"]))

            # 3 · a widget answers
            pg.goto(probe.as_uri())
            try:
                pg.wait_for_function(
                    "() => /[0-9]/.test((document.querySelector('.ls-out')||{}).innerText||'')", timeout=20000)
            except Exception:
                found.append(("3 widget", "the widget never showed an answer; page errors: %s" % errs[:3]))
                b.close()
                return found, answered, len(rows), len(sample)
            out0 = pg.locator(".ls-out").inner_text()
            v0 = pg.locator(".ls-slider .ls-v").inner_text()
            if abs(float(v0) / 22.095389 - 1) > 1e-3:
                found.append(("3 widget", "the slider starts at %s, not where the engine says the row stands" % v0))
            if "this page does not carry" not in out0:
                found.append(("3 widget", "sw_ap_design is not refused in the page's words: %s" % out0))
            s = pg.locator(".ls-slider input").first
            s.evaluate("e => { e.value = e.max; e.dispatchEvent(new Event('input')); }")
            try:
                pg.wait_for_function(
                    "(was) => (document.querySelector('.ls-out')||{}).innerText !== was", arg=out0, timeout=20000)
            except Exception:
                found.append(("3 widget", "the answer did not move when the slider did"))
            try:
                pg.wait_for_selector(".ls-fig canvas", timeout=20000)
            except Exception:
                found.append(("3 widget", "the sweep was not drawn: %s" % pg.locator(".ls-fig").inner_text()))
            if errs:
                found.append(("3 widget", "page errors: %s" % errs[:3]))
            b.close()
    finally:
        probe.unlink(missing_ok=True)
    return found, answered, len(rows), len(sample)


def selftest():
    good = {
        "imports": [],
        "identity": json.dumps({"kernel": "abc", "graph": "def", "rows": 2}),
        "before": json.dumps({"ok": False, "message": "no design is open: the page opens the design it carries before it runs"}),
        "opened": json.dumps({"ok": True, "rows": 2, "graph": "def"}),
        "runs": {
            "a": json.dumps({"ok": True, "values": [{"id": "a", "si": 1.0}], "blocked": []}),
            "b": json.dumps({"ok": True, "values": [], "blocked": [{"id": "b", "kind": "data-missing"}]}),
        },
        "supplied": json.dumps({"ok": False, "message": "a computed row takes no value"}),
        "sweep": json.dumps({"ok": True, "figure": {"series": [{"y": [1, 2, 3, 4, 5, 6, 7]}]}}),
    }
    assert engine_findings(good, ["a", "b"])[0] == [], engine_findings(good, ["a", "b"])

    def broken(**kw):
        e = dict(good)
        e.update(kw)
        return engine_findings(e, ["a", "b"])[0]

    assert any("asks the page" in x for x in broken(imports=["env.f"]))
    assert any("identity" in x for x in broken(identity="nope"))
    assert any("which graph" in x for x in broken(identity=json.dumps({"kernel": "abc", "graph": None})))
    assert any("ran before a design was open" in x for x in broken(before=good["runs"]["a"]))
    assert any("did not open" in x for x in broken(opened=json.dumps({"ok": False, "message": "does not read"})))
    assert any("took a value" in x for x in broken(supplied=json.dumps({"ok": True})))
    assert any("7 of 7" in x for x in broken(sweep=json.dumps({"ok": True, "figure": {"series": [{"y": [1, None]}]}})))
    assert any("same value" in x for x in broken(sweep=json.dumps({"ok": True, "figure": {"series": [{"y": [2] * 7}]}})))
    none_answer = dict(good["runs"], a=json.dumps({"ok": True, "values": [], "blocked": []}))
    assert any("no computed row answered" in x for x in broken(runs=none_answer))
    no_data = dict(good["runs"], b=json.dumps({"ok": True, "values": [], "blocked": [{"id": "b", "kind": "undefined"}]}))
    assert any("reference data" in x for x in broken(runs=no_data))
    assert any("gave nothing" in x for x in broken(runs={"a": good["runs"]["a"]}))
    print("readers_check selftest: every broken engine answer refused")


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--folder", default=str(ROOT / "target" / "readers"))
    ap.add_argument("--answers", help="write every engine answer here, for crates/vleo-kernel-wasm's comparison")
    ap.add_argument("--chromium", default=chromium_path(),
                    help="the browser to drive; found as tools/panel_check.py finds it (VLEO_CHROMIUM)")
    ap.add_argument("--selftest", action="store_true")
    a = ap.parse_args()
    if a.selftest:
        selftest()
        return 0
    folder = Path(a.folder)
    if not (folder / "index.html").is_file() or not (folder / "assets" / "kernel.js").is_file():
        print("no readers' folder at %s — build it with: cargo run -p xtask -- readers" % folder)
        return 2
    found, answered, asked, sampled = run(folder, a.answers, a.chromium)
    print("readers_check: %d pages opened, %d of %d computed rows answered in the page, the rest refused by name"
          % (sampled + 1, answered, asked))
    for check, what in found:
        print("  %-10s %s" % (check, what))
    print("%d findings" % len(found))
    return 1 if found else 0


if __name__ == "__main__":
    sys.exit(main())
