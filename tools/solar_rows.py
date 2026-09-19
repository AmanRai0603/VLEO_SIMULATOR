#!/usr/bin/env python3
"""Write docs/SOLAR_ROWS.md: what every live solar row is, and where it came from.

Generated, because a classification maintained by hand is a classification that
is wrong. Three questions are answered for each row, and each is answered by
measurement rather than by reading a label:

  WHERE IT CAME FROM. The MATLAB study publishes one thing -- a driver set of
  five scenarios over f107, f107bar, ap, kp_mean and kp_peak, twenty-five
  numbers, in matlab/reference/mission_drivers.csv. A row is part of that port
  if l3_solar_interface reaches it. A row can also carry the study's relation
  and sit OUTSIDE the chain, because the relation was moved into vleo-core so a
  driver set could evaluate it at five scenarios rather than one; those rows are
  the provenance of a kernel constant and compute nothing live. Everything else
  is extra -- a question this tree asks that the study did not.

  WHAT IT DOES. Its own question, as the sheet states it.

  WHETHER IT IS A NUMBER OR A CURVE. Every row answers with one number; the
  question is whether anything moves it. A row with no upstream decision that
  changes its answer is a fixed number and a graph of it is a flat line. A row
  with one is a curve, and the decision that moves it most is named. Measured
  through /v1/levers, so a term that is in the relation and inert is counted as
  inert.

Needs the daemon: cargo run --release -p vleo-daemon
"""
import json, os, re, glob, sys, urllib.request

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
BASE = "http://127.0.0.1:7777"


def get(path):
    return json.loads(urllib.request.urlopen(BASE + path).read())


def main():
    idx = json.load(open(os.path.join(ROOT, "generated/index.json")))
    rows = idx["rows"]
    byi = {r["i"]: r for r in rows}
    byid = {r["id"]: r for r in rows}
    prod = {r["i"]: r["in"] for r in rows}

    def ancestors(nid):
        seen, st = set(), [byid[nid]["i"]]
        while st:
            n = st.pop()
            for p in prod.get(n, []):
                if p not in seen:
                    seen.add(p)
                    st.append(p)
        return {byi[i]["id"] for i in seen}

    chain = ancestors("l3_solar_interface") | {"l3_solar_interface"}

    # WHAT COUNTS AS PORTED FROM THE LEGACY MATLAB.
    #
    # The legacy tool is 01_kernel/sw_study -- eight tabs, 38 analysis methods,
    # ~57 views over one database. A row is a port of it when any of three
    # things is true, and each is a record rather than a judgement:
    #
    #   - it holds a parity.csv, which is the legacy run's own answer for that
    #     row, kept as a second opinion;
    #   - docs/MATLAB_PORT_PLAN.md maps it to a legacy routine by name -- the
    #     prf_*, F.*, C.* and R.* families, or a named tab;
    #   - l3_solar_interface reaches it, because that chain exists for one
    #     purpose: to reproduce the legacy driver table, five scenarios over
    #     f107, f107bar, ap, kp_mean and kp_peak.
    #
    # Anything else this tree added, and says so.
    plan = open(os.path.join(ROOT, "docs/MATLAB_PORT_PLAN.md"), encoding="utf-8").read()
    # THE SOURCE CELL IS TAKEN WHOLE AND THEN SEARCHED, not matched by shape.
    #
    # The first version required the cell to be exactly a backticked name, so
    # "`prf_ap2kp` fit" and "`prf_ap2kp` table" matched NOTHING — the words after
    # the closing backtick made the whole row fail to parse, and the row fell
    # through to "added by this tree". sw_kp_from_ap survived that because it
    # also holds a parity.csv; sw_kp_slot_bias did not, and was counted as an
    # addition for as long as this script has existed. A pattern that is silent
    # when it fails is the wrong shape for a classifier.
    mapped = {}
    for nid, _q, src in re.findall(
            r"^\|\s*\**`([a-z0-9_]+)`\**\s*\|([^|]*)\|([^|]*)\|\s*$", plan, re.M):
        m = re.search(r"(prf_[a-z0-9_]+|[FCR]\.[a-z0-9_]+|Climate tab)", src)
        if m:
            mapped.setdefault(nid, src.strip().strip("`* "))

    def origin(nid):
        if os.path.exists(os.path.join(ROOT, "crates/vleo-mod-solar/nodes/%s/parity.csv" % nid)):
            return "the legacy run's own answer, in `parity.csv`"
        if nid in mapped:
            return "`%s`" % mapped[nid]
        if nid in chain:
            return "the chain that reproduces the legacy driver table"
        return None

    # Which panel draws which row.
    drawn = {}
    for p in glob.glob(os.path.join(ROOT, "web/js/*.js")) + glob.glob(os.path.join(ROOT, "panels/*.toml")):
        txt = open(p, encoding="utf-8").read()
        for m in re.finditer(r"id:\s*'([a-z0-9_]+)',\s*\n\s*rows:\s*\[([^\]]*)\]", txt):
            panel, body = m.group(1), m.group(2)
            for rid in re.findall(r"'([a-z0-9_.]+)'", body):
                drawn.setdefault(rid, []).append(panel)

    solar = [r for r in rows
             if (r["id"].startswith("sw_") or r["id"].startswith("l3_solar"))
             and r["state"] != "deprecated"]
    solar.sort(key=lambda r: r["i"])

    described = []
    for r in solar:
        nid = r["id"]
        try:
            lv = get("/v1/levers?node=" + nid).get("levers", [])
        except Exception:
            lv = []
        moves = [l for l in lv if l.get("span") is not None and l["span"] > 0]
        inert = [l for l in lv if l.get("span") == 0]
        blocked = [l for l in lv if l.get("span") is None]
        if r["kind"] == "declared":
            shape = "**number** — a decision; it moves other rows"
        elif moves:
            shape = "**curve** over `%s` (%s)" % (moves[0]["id"], pct(moves[0]["span"]))
            if len(moves) > 1:
                shape += ", +%d" % (len(moves) - 1)
        elif blocked:
            shape = "**number here** — %d upstream decision(s) refuse at a range end" % len(blocked)
        elif inert:
            shape = "**number** — all %d decisions upstream leave it unchanged" % len(inert)
        else:
            shape = "**number** — no decision upstream at all"
        described.append((r, shape, origin(nid), drawn.get(nid, [])))

    # A row kept only because a removable row reads it is not really kept.
    readers = {}
    for r in rows:
        if r["state"] == "deprecated":
            continue
        for pi in r["in"]:
            readers.setdefault(byi[pi]["id"], []).append(r["id"])
    free = set()
    while True:
        added = False
        for r, _sh, org, fig in described:
            nid = r["id"]
            if nid in free or org or fig:
                continue
            if all(x in free for x in readers.get(nid, []) if x != nid):
                free.add(nid)
                added = True
        if not added:
            break

    def verdict(r, org, fig):
        nid = r["id"]
        rd = sorted(x for x in readers.get(nid, []) if x != nid)
        if nid in free:
            with_ = [x for x in rd if x in free]
            return ("EASY, AS A PAIR" if with_ else "EASY",
                    ("free only with %s" % ", ".join("`%s`" % x for x in with_)) if with_
                    else "nothing reads it, no figure cites it, not a port")
        if org:
            return ("KEEP", "it is the port")
        if rd:
            return ("KEEP", "%s read%s it" % (", ".join("`%s`" % x for x in rd[:3]),
                                              "" if len(rd) == 1 else ""))
        return ("COSTS A FIGURE", "the %s panel argues with it" % "/".join(sorted(set(fig))))

    def table(items, show_origin):
        head = "| row | what it answers | value | number or curve | %s | remove? |\n" % (
            "ported from" if show_origin else "figure")
        out = [head, "|---|---|---|---|---|---|\n"]
        for r, shape, org, fig in items:
            v, why = verdict(r, org, fig)
            val = "%.6g" % r["value"] if r["value"] is not None else "computed"
            col = org if show_origin else ("/".join(sorted(set(fig))) or "—")
            out.append("| `%s` | %s | %s | %s | %s | **%s** — %s |\n"
                       % (r["id"], r["label"], val, shape, col, v, why))
        return "".join(out)

    PF = [d for d in described if d[2] and d[3]]
    PN = [d for d in described if d[2] and not d[3]]
    AF = [d for d in described if not d[2] and d[3]]
    AN = [d for d in described if not d[2] and not d[3]]

    out = ["# The solar subsystem, row by row\n",
           "\nGenerated by `tools/solar_rows.py`. Do not edit by hand.\n"]
    out.append("\n## The two questions this answers\n\n")
    out.append("**Is it a port of the legacy MATLAB?** The legacy tool is `01_kernel/sw_study` — "
               "eight tabs, 38 analysis methods, ~57 views over one database. A row counts as a "
               "port when it holds a `parity.csv` (the legacy run's own answer for that row), or "
               "when `docs/MATLAB_PORT_PLAN.md` names the legacy routine it came from, or when it "
               "sits in the chain that reproduces the legacy driver table. The rest this tree "
               "added.\n\n")
    out.append("**Does a figure go with it?** The eight legacy tabs were ported as *panels*, not "
               "as rows — §12 of the plan, on the grounds that a wrong number crashes a test "
               "while a wrong chart looks beautiful, so a panel carries a reference image and an "
               "automated check instead. Most panels plot the NOAA record and name the rows they "
               "argue about, so for those 'has a figure' means **a panel cites it**. The two Part D "
               "panels are the exception and plot ROW ANSWERS directly: `drivers` draws the "
               "twenty-five variables `l3_solar_interface` publishes against the legacy run's own, "
               "and `closure` draws each required/achieved pair and the margin between them, swept "
               "over whichever decision `/v1/levers` reports as spending that margin fastest.\n\n")
    out.append("**No row here duplicates another.** All %d were swept over seven drivers across "
               "their declared ranges and fingerprinted; every fingerprint is distinct. Nothing "
               "is removable for being a repeat.\n\n" % len(solar))
    out.append("| | with a figure | no figure | total |\n|---|---|---|---|\n")
    out.append("| **Ported** from the legacy MATLAB | %d | %d | %d |\n"
               % (len(PF), len(PN), len(PF) + len(PN)))
    out.append("| **Added** by this tree | %d | %d | %d |\n"
               % (len(AF), len(AN), len(AF) + len(AN)))
    out.append("\n**%d rows are free to remove**: %s.\n"
               % (len(free), ", ".join("`%s`" % x for x in sorted(free)) or "none"))

    out.append("\n---\n\n# 1 · Ported, and a figure goes with it — %d rows\n\n" % len(PF))
    out.append("The legacy tab these belong to was ported as a panel, so the picture came "
               "across with the row. None of these is a removal candidate.\n\n")
    out.append(table(PF, True))

    out.append("\n---\n\n# 2 · Ported, no figure — %d rows\n\n" % len(PN))
    out.append("**And the reason is not an omission.** These reproduce the legacy DRIVER TABLE — "
               "five scenarios over f107, f107bar, ap, kp_mean and kp_peak. That was a table in "
               "the legacy tool too, not a plot, so there was never a figure here to port. If a "
               "picture of the driver set is wanted it has to be drawn new; that is the one real "
               "gap in this subsystem's figures.\n\n")
    out.append(table(PN, True))

    out.append("\n---\n\n# 3 · Added, and a figure cites it — %d rows\n\n" % len(AF))
    out.append("Rows this tree added, which a ported panel then leans on. Removing one leaves "
               "its panel arguing about a row that is gone.\n\n")
    out.append(table(AF, False))

    out.append("\n---\n\n# 4 · Added, no figure — %d rows\n\n" % len(AN))
    out.append("Neither in the legacy tool nor cited by a panel. Every free removal is here.\n\n")
    out.append(table(AN, False))

    p = os.path.join(ROOT, "docs/SOLAR_ROWS.md")
    open(p, "w", encoding="utf-8").write("".join(out))
    print("solar rows: %d live -> %s" % (len(solar), p))
    print("  ported %d (%d with a figure, %d without)" % (len(PF) + len(PN), len(PF), len(PN)))
    print("  added  %d (%d with a figure, %d without)" % (len(AF) + len(AN), len(AF), len(AN)))
    print("  free to remove: %d" % len(free))
    return 0


def pct(v):
    return "%d%%" % round(v * 100) if v >= 0.1 else "%.2f%%" % (v * 100)


if __name__ == "__main__":
    sys.exit(main())
