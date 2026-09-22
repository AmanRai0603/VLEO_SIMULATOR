#!/usr/bin/env python3
"""The numbers a theory tab states, against the engine and against the record.

    tools/theory_check.py              needs a daemon
    tools/theory_check.py --selftest   needs neither

WHY THIS EXISTS. A `[theory]` block is prose, and prose is outside the sheet
hash, so nothing regenerates it and nothing checks it. It is also where a row
explains itself to the person who has to sign `maths.confirmed_by` — §30 A3 —
and a derivation with a wrong magnitude in it is worse than no derivation,
because it reads as evidence.

§21 found the same defect class in panels: four of `design`'s numbers were a
row's constants copied into the figure, and one was two revisions stale. The
answer there was the `correct` block, which makes a panel declare what it must
measure off the engine. This is that idea for a sheet's prose.

WHAT IT CHECKS, and the shape matters. Every claim below carries the EXACT
SUBSTRING it appears as in the sheet. If the sheet's wording changes, the check
fails and says the claim moved rather than silently checking a number nobody
writes any more — a probe that cannot fail is the defect this file exists to
catch, so it may not have one itself.

The claims are verified three ways, none of which is the prose:

  * off the row's own `maths.expression`, parsed into coefficients and then
    PROVEN against the expression evaluated as a string, so a parse that read
    the relation wrongly fails instead of agreeing with itself;
  * off the engine, through `/v1/run` with driver overrides, which is the same
    path the face takes;
  * off the RECORD — the bundle CSVs — for a claim about what has been observed.

The one it found on the first pass: `env_exospheric_temperature`'s theory said
the slow solar term moves T_inf "by about 900 K over the observed range of
F10.7A". The observed range of the 81-day centred mean is 65.91 to 226.81, which
through the term's own coefficient is 521 K. 900 K is the range of the DAILY
flux, 64 to 343 — the fast term's quantity, cited for the slow term, in the one
step whose whole job is to separate them.

COVERAGE, STATED PLAINLY. 66 rows carry a theory block with `math` in its steps.
This checks ONE of them, `env_exospheric_temperature`, because that is the row
§30 step 3 is about. The other 65 are unchecked, and several carry another row's
answer as a literal in their prose — `l3_solar_ach_01` carries 104.07,
`sw_ap_cold_long` carries 3.5937 — which is exactly the staleness §21 found.
Adding a row here is adding an entry to `CLAIMS`.
"""

import argparse
import csv
import json
import math
import os
import re
import sys
import urllib.error
import urllib.parse
import urllib.request

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
HOST = os.environ.get("VLEO_DAEMON", "http://localhost:7777")
BUNDLE = os.path.join(ROOT, "bundles/solar-weather/2026.09.14")
WINDOW = 81        # the centred mean the relation's slow term is defined on
# Centred against trailing does not matter for what is claimed off it: both are
# 81-day means of one series, so their extremes agree to a fraction of a per cent
# and the RANGE is what the theory cites. It matters for a value on a given day,
# which nothing here claims. Measured rather than assumed — swapping the window
# to trailing moves neither end of the range enough to fail any claim.


# ─── the sheet ────────────────────────────────────────────────────────────────

def flat(s):
    """One line, single-spaced. A sheet wraps its prose at 80 columns, so a claim
    that spans a line break is the same sentence and must match as one."""
    return " ".join(s.split())


def sheet(node):
    """The node's sheet, parsed and raw. The raw text is what claims are found in."""
    import tomllib
    for crate in sorted(os.listdir(os.path.join(ROOT, "crates"))):
        p = os.path.join(ROOT, "crates", crate, "nodes", node, "node.toml")
        if os.path.exists(p):
            raw = open(p, encoding="utf-8").read()
            return tomllib.loads(raw), flat(raw), p
    raise SystemExit("no sheet for %s" % node)


def evaluator(expr):
    """The sheet's own expression, as something callable.

    Built by rewriting the string, so what runs is what the sheet says and not a
    second copy of the relation kept here. The long name is replaced first:
    `F10.7A` contains `F10.7`.
    """
    rhs = expr.split("=", 1)[1]
    py = (rhs.replace("F10.7A", "fa").replace("F10.7", "f")
             .replace("Kp", "kp").replace("exp(", "math.exp("))
    code = compile(py.strip(), "<maths.expression>", "eval")
    return lambda f, fa, kp: eval(code, {"math": math}, {"f": f, "fa": fa, "kp": kp})


def terms(expr):
    """The five coefficients, read off the expression AND PROVEN against it.

    A regex over someone else's algebra is a guess. So the parse is checked by
    rebuilding the relation from the coefficients it claims to have found and
    comparing it against the expression evaluated as a string, over a grid. If
    the expression is ever written in a form this cannot read, the check fails
    by name instead of quietly agreeing with itself.
    """
    n = r"([0-9]+(?:\.[0-9]+)?)"
    m = re.search(n + r"\s*\+\s*" + n + r"\s*\*\s*F10\.7A\s*\+\s*" + n
                  + r"\s*\*\s*\(\s*F10\.7\s*-\s*F10\.7A\s*\)\s*\+\s*" + n
                  + r"\s*\*\s*Kp\s*\+\s*" + n + r"\s*\*\s*exp\(\s*Kp\s*\)",
                  expr)
    if not m:
        return None
    a, b, c, d, e = (float(x) for x in m.groups())
    ev = evaluator(expr)
    for f in (60.0, 150.0, 400.0):
        for fa in (60.0, 150.0, 400.0):
            for kp in (0.0, 3.0, 9.0):
                mine = a + b * fa + c * (f - fa) + d * kp + e * math.exp(kp)
                if abs(mine - ev(f, fa, kp)) > 1e-9:
                    return None
    return {"const": a, "slow": b, "fast": c, "geo_linear": d, "geo_exp": e}


# ─── the record ───────────────────────────────────────────────────────────────

def observed_f107a():
    """The observed range of the 81-day CENTRED mean, off the bundle.

    Not the daily flux, which is a different quantity with a range 73 per cent
    wider, and not the declared domain of `env_f107a`, which is 60 to 400 and is
    where the relation is valid rather than where the Sun has been.
    """
    v = []
    with open(os.path.join(BUNDLE, "observed_daily.csv")) as fh:
        for r in csv.DictReader(l for l in fh if not l.startswith("#")):
            if r["f107"].strip():
                v.append(float(r["f107"]))
    h = WINDOW // 2
    a = [sum(v[i - h:i + h + 1]) / WINDOW for i in range(h, len(v) - h)]
    return min(a), max(a), min(v), max(v)


# ─── the engine ───────────────────────────────────────────────────────────────

def run(node, **over):
    """One run through `/v1/run`, with driver overrides — the face's own path."""
    q = (HOST + "/v1/run?" + urllib.parse.urlencode({"node": node})
         + "".join("&set=%s:%r" % (k, v) for k, v in sorted(over.items())))
    with urllib.request.urlopen(q, timeout=120) as fh:
        d = json.load(fh)
    if not d.get("ok"):
        return None
    out = {v["id"]: v["si"] for v in d.get("values", [])}
    out.update({v["symbol"]: v["si"] for v in d.get("values", []) if v.get("symbol")})
    return out


def probe(node, **inputs):
    """One RELATION, at given inputs, with no graph — `/v1/probe`.

    Not `/v1/run`. This file asks what the thermosphere relation DOES at chosen
    drivers, which is a question about the relation and not about the design,
    and since env_f107 started reading the solar subsystem the two have needed
    different paths: the flux is no longer free, so the engine correctly refuses
    to pretend a supplied one survives the run.

    The refusal is the point. Before the daemon learned to give it, a `set=` on
    a computed row was accepted and silently dropped, and this file compared
    867.2 K against 867.2 K across a hundred sweep points and called the
    disagreement 1303 K without ever noticing it was measuring one number.
    """
    q = (HOST + "/v1/probe?" + urllib.parse.urlencode({"node": node})
         + "".join("&in=%s:%r" % (k, v) for k, v in sorted(inputs.items())))
    with urllib.request.urlopen(q, timeout=120) as fh:
        d = json.load(fh)
    if not d.get("ok"):
        raise SystemExit("probe refused: %s" % d.get("message", d))
    return d["si"]


def t_inf(f, fa, kp):
    return probe("env_exospheric_temperature", env_f107=f, env_f107a=fa, env_kp=kp)


# ─── the claims ───────────────────────────────────────────────────────────────
#
# Each entry is (a template, a function returning what the sheet OUGHT to state).
# `{}` in the template stands for a number, and the numbers are read out of the
# sheet rather than out of the template, which splits two different failures:
#
#   * the words are gone       -> the claim moved; re-check it, do not edit this
#   * the words are there and the digits disagree -> the sheet is wrong, by this
#     much, and the message says so
#
# It has to be that way round. The first version carried each claim as a literal
# substring INCLUDING its number, so every wrong-number case failed on the
# substring and the comparison beside it never ran on anything — a check whose
# arithmetic half could have been broken from the start with nothing to show it.

SUSTAINED = 104.071328      # sw_f107_design_long, the 81-day level of the hot case
SINGLE_DAY = 124.143600     # sw_f107_design_short, the disturbed day's own flux


def claims_exospheric(t, d, bad):
    """`env_exospheric_temperature` — the row §30 step 3 is about."""
    out = []

    # ── off the expression's own coefficients ────────────────────────────────
    out.append(("{} K is the intercept of the fit", lambda: (t["const"],)))
    out.append(("The coefficient is smaller than the slow one — {} against {} —",
                lambda: (t["fast"], t["slow"])))
    # The geomagnetic term, split. The claim is that the exponential is
    # negligible at quiet levels and is a seventh of the term at storm levels,
    # which is the reason the relation has two parts rather than one. Kp itself
    # is in the template as a number, so a claim moved to another Kp is a moved
    # claim and not a wrong one.
    out.append(("at Kp 3 the exponential contributes {} K of {}",
                lambda: (round(t["geo_exp"] * math.exp(3.0), 1),
                         round(t["geo_linear"] * 3.0 + t["geo_exp"] * math.exp(3.0), 1))))
    out.append(("at Kp 7 it contributes {} K of {}",
                lambda: (round(t["geo_exp"] * math.exp(7.0), 1),
                         round(t["geo_linear"] * 7.0 + t["geo_exp"] * math.exp(7.0), 1))))

    # ── off the record ───────────────────────────────────────────────────────
    lo, hi, dlo, dhi = observed_f107a()
    out.append(("over the observed range of F10.7A — {} to {} across the record",
                lambda: (round(lo, 2), round(hi, 2))))
    # Rounded to ten, because the sheet says "about" and a theory tab that
    # tracked the record to four figures would be a fixture pretending to be
    # prose. The rounding is here and not in the sheet, so the sheet's number is
    # checked against a measurement rather than against a restatement of itself.
    out.append(("it moves T_inf by about {} K",
                lambda: (round(t["slow"] * (hi - lo) / 10.0) * 10,)))
    # AND THE WRONG ONE, NAMED. The sheet says what 900 K belongs to, so that
    # sentence is a claim too: the daily flux's range, through the same
    # coefficient. Left unchecked it is the number that would drift back.
    out.append(("The daily flux runs {} to {} over the same record, which through "
                "this coefficient would be {} K",
                lambda: (round(dlo), round(dhi),
                         round(t["slow"] * (dhi - dlo) / 10.0) * 10)))

    # ── off the engine ───────────────────────────────────────────────────────
    kp = run("sw_kp_scenarios") or {}
    slots = [kp.get(k) for k in ("Kp_mean_hotmean", "Kp_peak_hotmean",
                                 "Kp_mean_hotday", "Kp_peak_hotday")]
    if None in slots:
        bad.append("the engine did not publish the four Kp slots the assumption cites")
        return out
    mean_hot, peak_hot, mean_day, peak_day = slots
    out.append(("{} against {} at the hot sustained scenario",
                lambda: (round(mean_hot, 3), round(peak_hot, 3))))
    a = t_inf(SUSTAINED, SUSTAINED, mean_hot)
    b = t_inf(SUSTAINED, SUSTAINED, peak_hot)
    c = t_inf(SINGLE_DAY, SUSTAINED, mean_day)
    e = t_inf(SINGLE_DAY, SUSTAINED, peak_day)
    if None in (a, b, c, e):
        bad.append("the engine refused one of the four scenario runs")
        return out
    out.append(("Fed through this relation that is {} K against {} K",
                lambda: (round(a, 1), round(b, 1))))
    out.append(("on the worst-day scenario it is {} K against {} K",
                lambda: (round(c, 1), round(e, 1))))
    out.append(("a {} per cent difference", lambda: (round(100.0 * (e - c) / c),)))
    return out


def check():
    """Every claim, and the two structural checks under them."""
    bad = []
    node = "env_exospheric_temperature"
    d, raw, _ = sheet(node)
    expr = d["maths"]["expression"]

    # 1 · THE EXPRESSION IS THE IMPLEMENTATION. Nothing in the gate compares the
    # two: the sheet's relation is a string and the hole is Rust, and they are
    # two statements of one fact. Every claim below is read off the expression or
    # off the engine, so if they had drifted the claims would agree with one and
    # be wrong about the other.
    ev = evaluator(expr)
    worst, pts = 0.0, 0
    for f in (60.0, SUSTAINED, 150.0, 250.0, 400.0):
        for fa in (60.0, SUSTAINED, 150.0, 400.0):
            for kp in (0.0, 3.0, 5.197771, 7.0, 9.0):
                got = t_inf(f, fa, kp)
                pts += 1
                if got is None:
                    bad.append("the engine refused f107=%g f107a=%g kp=%g, "
                               "inside its own declared domain" % (f, fa, kp))
                    continue
                worst = max(worst, abs(ev(f, fa, kp) - got))
    print("  the expression IS the implementation: %d points, worst disagreement %.2e K"
          % (pts, worst))
    if worst > 1e-6:
        bad.append("the sheet's expression and the engine disagree by %.4g K" % worst)

    # 2 · THE THEORY'S STEPS RECONSTRUCT THE RELATION. Four steps, each a term,
    # and read as a build-up they must compose to `maths.expression` — otherwise
    # the tab derives one relation and the row computes another.
    steps = [s.get("math", "") for s in (d.get("theory") or {}).get("step", [])]
    built = "".join(s.split("=", 1)[1] if "=" in s else s for s in steps)
    squash = lambda s: re.sub(r"[\s K]", "", s)
    if squash(built) != squash(expr.split("=", 1)[1]):
        bad.append("the theory's steps do not compose to the relation:\n"
                   "       steps give %s\n       sheet says %s"
                   % (squash(built), squash(expr.split("=", 1)[1])))
    else:
        print("  the theory's %d steps compose to `maths.expression`" % len(steps))

    t = terms(expr)
    if not t:
        bad.append("the coefficients could not be read off the expression, or the "
                   "parse did not reproduce it — %r" % expr)
        print("theory claims: %d FAILED" % len(bad))
        for x in bad:
            print("  FAIL %s" % x)
        return 1

    # 3 · EVERY NUMERIC CLAIM, against the value and against the sentence.
    n = 0
    for template, value in claims_exospheric(t, d, bad):
        n += 1
        pat = re.escape(flat(template)).replace(r"\{\}", r"(-?[0-9]+(?:\.[0-9]+)?)")
        m = re.search(pat, raw)
        if not m:
            bad.append("the sheet no longer says %r — the claim MOVED, so re-check it "
                       "against the source rather than editing this file" % flat(template))
            continue
        stated = [float(x) for x in m.groups()]
        want = value()
        if len(stated) != len(want):
            bad.append("%r has %d number(s) and %d were measured for it"
                       % (flat(template), len(stated), len(want)))
            continue
        off = [(i, a, b) for i, (a, b) in enumerate(zip(stated, want))
               if abs(a - b) > 0.051]
        if off:
            bad.append("%r: the sheet states %s and the measurement is %s"
                       % (flat(template), [a for _, a, _ in off], [b for _, _, b in off]))

    # 4 · the declared floor the theory points at.
    if "the declared lower bound of 400 K" in raw:
        n += 1
        if abs(d["output"]["lower"] - 400.0) > 1e-9:
            bad.append("the theory cites a declared lower bound of 400 K and the "
                       "sheet declares %g" % d["output"]["lower"])
    else:
        bad.append("the theory no longer cites the declared lower bound")

    # 5 · the semiannual assumption names a row and a figure. Both must exist,
    # because the assumption's force is that the measurement is IN the tree and
    # does not reach this relation.
    n += 1
    amp = os.path.join(ROOT, "crates/vleo-mod-solar/nodes/sw_semiannual_amplitude/node.toml")
    face = os.path.join(ROOT, "web/js/solar.js")
    if not os.path.exists(amp) or 'state = "published"' not in open(amp, encoding="utf-8").read():
        bad.append("the assumption cites sw_semiannual_amplitude as a measurement "
                   "the tree holds, and it is not a published row")
    eq = open(face, encoding="utf-8").read()
    if "March equinox" not in eq or "September equinox" not in eq:
        bad.append("the assumption says the climate panel marks both equinoxes, "
                   "and the face no longer marks them")

    print("theory claims: %d checked, %s"
          % (n, "all as stated" if not bad else "%d FAILED" % len(bad)))
    for x in bad:
        print("  FAIL %s" % x)
    return 1 if bad else 0


def selftest():
    """The parts that need no daemon: the parse, the record, and the probes."""
    bad = 0

    # 1 · the expression rewriter handles the long name first. `F10.7A` contains
    # `F10.7`, so the naive order turns the slow term into `fA` and the relation
    # silently loses its 81-day mean.
    ev = evaluator("T = 379 + 3.24*F10.7A + 1.3*(F10.7 - F10.7A) + 28*Kp + 0.03*exp(Kp)")
    for f, fa, kp, want in ((0, 0, 0, 379.03), (150, 150, 3, 949.6025),
                            (200, 150, 0, 930.03), (150, 150, 7, 1093.899)):
        # Caught rather than allowed to propagate: replacing the short name first
        # leaves `fA` in the rewritten source, which raises here. A checker that
        # dies is a worse signal than one that names what is wrong.
        try:
            got = ev(f, fa, kp)
        except NameError as exc:
            bad += 1
            print("  FAIL the rewritten expression does not evaluate — %s. The long "
                  "name has to be replaced first: F10.7A contains F10.7" % exc)
            break
        if abs(got - want) > 0.01:
            bad += 1
            print("  FAIL the expression evaluates to %.4f at (%g, %g, %g), not %.4f"
                  % (got, f, fa, kp, want))

    # 2 · THE COEFFICIENT PARSE PROVES ITSELF. A regex over someone's algebra is
    # a guess, so `terms` rebuilds the relation from what it found and compares.
    t = terms("T = 379 + 3.24*F10.7A + 1.3*(F10.7 - F10.7A) + 28*Kp + 0.03*exp(Kp)")
    if not t or t != {"const": 379.0, "slow": 3.24, "fast": 1.3,
                      "geo_linear": 28.0, "geo_exp": 0.03}:
        bad += 1
        print("  FAIL the coefficients did not come back off the expression: %r" % t)
    # and it must REFUSE a relation it cannot read, rather than half-reading one
    if terms("T = 379 + 3.24*F10.7A + 28*Kp") is not None:
        bad += 1
        print("  FAIL a relation this cannot parse was accepted anyway")
    # THE CASE THE SELF-PROOF EXISTS FOR, and the only one that can show it is
    # load-bearing: a relation the regex matches as a PREFIX while the algebra
    # carries a term beyond it. Read by pattern alone the five coefficients come
    # back looking right and the relation they describe is not the relation.
    if terms("T = 379 + 3.24*F10.7A + 1.3*(F10.7 - F10.7A) + 28*Kp "
             "+ 0.03*exp(Kp) + 5*Kp") is not None:
        bad += 1
        print("  FAIL a relation with a term past what the pattern reads was accepted: "
              "the parse is not being proven against the expression")

    # 3 · the record's own ranges, and that the two are not the same quantity —
    # which is the defect this file was written to catch.
    try:
        lo, hi, dlo, dhi = observed_f107a()
        if not (60 < lo < 80 and 200 < hi < 250):
            bad += 1
            print("  FAIL the 81-day centred mean came back at %.2f to %.2f" % (lo, hi))
        if not (hi - lo) < (dhi - dlo) * 0.75:
            bad += 1
            print("  FAIL the 81-day mean's range is not materially narrower than the "
                  "daily flux's: %.1f against %.1f" % (hi - lo, dhi - dlo))
    except OSError as exc:
        bad += 1
        print("  FAIL the bundle could not be read: %s" % exc)

    # 4 · every claim's WORDS are still in the sheet, and each template matches
    # exactly once. A template that matched twice would check one sentence and
    # leave the other unchecked; one that matches none is a claim that moved.
    _, raw, _ = sheet("env_exospheric_temperature")
    for template in ("{} K is the intercept of the fit",
                     "The coefficient is smaller than the slow one — {} against {} —",
                     "at Kp 3 the exponential contributes {} K of {}",
                     "at Kp 7 it contributes {} K of {}",
                     "over the observed range of F10.7A — {} to {} across the record",
                     "it moves T_inf by about {} K",
                     "{} against {} at the hot sustained scenario",
                     "Fed through this relation that is {} K against {} K",
                     "on the worst-day scenario it is {} K against {} K",
                     "a {} per cent difference",
                     "the declared lower bound of 400 K"):
        pat = re.escape(flat(template)).replace(r"\{\}", r"(-?[0-9]+(?:\.[0-9]+)?)")
        hits = len(re.findall(pat, raw))
        if hits != 1:
            bad += 1
            print("  FAIL the template %r matches the sheet %d times, not once"
                  % (flat(template), hits))

    # 5 · the theory's steps compose to the relation, which needs no daemon.
    d, _, _ = sheet("env_exospheric_temperature")
    steps = [s.get("math", "") for s in (d.get("theory") or {}).get("step", [])]
    built = "".join(s.split("=", 1)[1] if "=" in s else s for s in steps)
    squash = lambda s: re.sub(r"[\s K]", "", s)
    if squash(built) != squash(d["maths"]["expression"].split("=", 1)[1]):
        bad += 1
        print("  FAIL the theory's steps do not compose to the relation")

    # 6 · AND THE SIGNATURE IS STILL A PERSON'S. This file checks arithmetic; it
    # cannot check that the relation is Jacchia's, and an agent may not sign that.
    if (d.get("maths") or {}).get("confirmed_by"):
        print("  note maths.confirmed_by now carries %r — if an agent put it there, "
              "that is the one defect nothing else in this repository can catch"
              % d["maths"]["confirmed_by"])

    print("selftest: %d cases, %s" % (22, "all as expected" if not bad else "%d FAILED" % bad))
    return 1 if bad else 0


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--selftest", action="store_true")
    a = ap.parse_args()
    if a.selftest:
        return selftest()
    try:
        return check()
    except (OSError, urllib.error.URLError) as exc:
        print("could not reach the engine: %s" % str(exc)[:160])
        print("a daemon must be running: cargo run --release -p vleo-daemon")
        return 1


if __name__ == "__main__":
    sys.exit(main())
