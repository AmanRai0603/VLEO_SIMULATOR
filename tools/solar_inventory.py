#!/usr/bin/env python3
"""Write docs/SOLAR_INVENTORY.md: every node in the solar chain, in full.

Generated, and for one reason: a reader verifying this subsystem needs the
QUESTION, the RELATION, the INPUTS, the ALGORITHM and the ANSWER of every row in
one place, and a document maintained by hand beside 62 sheets is a document that
disagrees with them. Everything here is read from node.toml and from the running
engine; nothing is transcribed.

Covers layer 3 (the solar subsystem), the crossing, and the layer-2 rows that
read it. Needs the daemon: cargo run --release -p vleo-daemon
"""
import glob, json, os, sys, tomllib, urllib.request

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
BASE = "http://127.0.0.1:7777"


def sheets():
    out = {}
    for f in glob.glob(os.path.join(ROOT, "crates/*/nodes/*/node.toml")):
        d = tomllib.load(open(f, "rb"))
        d["_dir"] = os.path.dirname(f)
        out[d["id"]] = d
    return out


def get(path):
    try:
        return json.loads(urllib.request.urlopen(BASE + path).read())
    except Exception as e:
        return {"ok": False, "why": str(e)}


def panels_citing():
    import re
    txt = open(os.path.join(ROOT, "web/js/solar.js"), encoding="utf-8").read()
    who = {}
    for m in re.finditer(r"id: '([a-z]+)',\n\s*rows: \[([^\]]*)\]", txt):
        for r in re.findall(r"'([a-z0-9_.]+)'", m.group(2)):
            who.setdefault(r, []).append(m.group(1))
    return who


def one(d, val, cite, out):
    nid = d["id"]
    q = d.get("question", {})
    mx = d.get("maths", {})
    o = d.get("output", {})
    out.append("\n### `%s` — %s\n\n" % (nid, d.get("label", "")))
    flags = ["**%s**" % d.get("kind"), d.get("state"), "layer %s" % d.get("layer"),
             "tier %s" % d.get("tier"), d.get("criticality", "")]
    if d.get("crosses_to"):
        flags.append("**crosses to `%s`** (sense `%s`)" % (d["crosses_to"], d.get("sense")))
    out.append("%s\n\n" % " · ".join(x for x in flags if x))
    out.append("**Asks.** %s\n\n" % q.get("text", "—"))
    if q.get("note"):
        out.append("%s\n\n" % q["note"].strip())
    out.append("**Relation.** `%s`\n\n" % mx.get("expression", "—"))
    out.append("Source: `%s`" % mx.get("source", "—"))
    if mx.get("confirmed_by"):
        out.append(" · relation confirmed by %s" % mx["confirmed_by"])
    else:
        out.append(" · **relation not yet read against its source**")
    out.append("\n\n")

    ins = d.get("input", [])
    if ins:
        out.append("**Reads.**\n\n")
        for i in ins:
            out.append("- `%s` as `%s` (%s)\n" % (i["var"], i["binding"], i.get("type", "")))
        out.append("\n")
    else:
        out.append("**Reads nothing** — it is a declared value or a leaf.\n\n")

    steps = d.get("algorithm", {}).get("step", [])
    if steps:
        out.append("**Algorithm.**\n\n")
        for st in steps:
            out.append("%d. %s → `%s` (%s)\n"
                       % (st.get("number", 0), st.get("text", "").strip(),
                          st.get("binds", ""), st.get("type", "")))
        out.append("\n")

    out.append("**Answers** `%s` : %s in `%s`" % (o.get("symbol"), o.get("type"), o.get("unit")))
    if o.get("lower") is not None or o.get("upper") is not None:
        out.append(", refusing outside %s … %s" % (o.get("lower"), o.get("upper")))
    out.append(".")
    if val is None:
        out.append(" The engine did not return it.")
    elif isinstance(val, str):
        out.append(" **Refused:** %s" % val)
    else:
        out.append(" **Now: %.6g**" % val)
    out.append("\n\n")

    pub = d.get("publishes", [])
    if pub:
        out.append("**Publishes %d more variables** (a set row — the declared exception to "
                   "one row, one answer):\n\n" % len(pub))
        for m in pub:
            out.append("- `%s.%s` — %s\n" % (nid, m["id"], m.get("label", "")))
        out.append("\n")

    ev = []
    if os.path.exists(os.path.join(d["_dir"], "parity.csv")):
        ev.append("holds `parity.csv`, the legacy run's own answer")
    fx = os.path.join(d["_dir"], "fixtures.toml")
    if os.path.exists(fx):
        n = tomllib.load(open(fx, "rb")).get("fixture", [])
        ev.append("%d fixture(s)" % len(n))
    ev.append("figure: %s" % (", ".join("`%s`" % p for p in cite.get(nid, [])) or "**none**"))
    out.append("*%s.*\n" % " · ".join(ev))

    for a in d.get("assumption", []):
        out.append("\n> **Assumption.** %s\n>\n> *Fails when:* %s\n"
                   % (a.get("text", ""), a.get("fails_when", "")))


def main():
    sh = sheets()
    cite = panels_citing()
    solar = sorted([v for v in sh.values() if v.get("subsystem") == "solar"],
                   key=lambda r: r.get("order", 0))
    l2 = sorted([v for v in sh.values()
                 if any(i["var"].startswith("l3_solar_interface") for i in v.get("input", []))],
                key=lambda r: r.get("order", 0))

    vals = {}
    for r in solar + l2:
        d = get("/v1/run?node=" + r["id"])
        if not d.get("ok"):
            vals[r["id"]] = d.get("message") or d.get("fault") or d.get("why") or "refused"
            continue
        for v in d.get("values", []):
            if v["id"] not in vals or v["id"] == r["id"]:
                vals[v["id"]] = v["si"]

    live = [r for r in solar if r["state"] != "deprecated"]
    dead = [r for r in solar if r["state"] == "deprecated"]

    out = ["# The solar chain, node by node\n",
           "\nGenerated by `tools/solar_inventory.py`. Do not edit by hand.\n\n",
           "Every row in the solar-weather subsystem, the one crossing out of it, and the "
           "layer-2 rows that read it. For each: what it asks, the relation, what it reads, "
           "the algorithm the hole implements, the answer the engine gives now, and which "
           "figure draws it.\n\n",
           "| | count |\n|---|---|\n",
           "| solar rows, live | %d |\n" % len(live),
           "| solar rows, deprecated | %d |\n" % len(dead),
           "| layer-2 rows reading the crossing | %d |\n" % len(l2),
           "| variables the crossing publishes | %d |\n"
           % (1 + len(sh["l3_solar_interface"].get("publishes", []))),
           ]

    out.append("\n---\n\n## 1 · Layer 3 — the solar-weather subsystem, live\n")
    for r in live:
        one(r, vals.get(r["id"]), cite, out)

    out.append("\n---\n\n## 2 · Layer 2 — what reads the crossing\n\n")
    out.append("The subsystem is reached through `l3_solar_interface` and never by reaching "
               "into it. These are every row on the other side of that seam.\n")
    for r in l2:
        one(r, vals.get(r["id"]), cite, out)

    out.append("\n---\n\n## 3 · Deprecated, and kept visible\n\n")
    out.append("A deprecated row is not deleted. It stays so that a reader who remembers it "
               "can find out what replaced it and why.\n")
    for r in dead:
        one(r, vals.get(r["id"]), cite, out)

    p = os.path.join(ROOT, "docs/SOLAR_INVENTORY.md")
    open(p, "w", encoding="utf-8").write("".join(out))
    print("solar inventory: %d live + %d deprecated + %d at layer 2 -> %s"
          % (len(live), len(dead), len(l2), p))
    return 0


if __name__ == "__main__":
    sys.exit(main())
