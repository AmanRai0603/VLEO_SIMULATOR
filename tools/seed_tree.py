#!/usr/bin/env python3
"""
Seed the node tree.

`xtask seed` in the delivery plan: run once, ever. It writes one folder per
node — the sheet, the initial hole bodies and the fixture table — and after
that the sheets are the source and this script is history. It is kept in the
repository because the alternative is that nobody can tell, in two years,
whether a field was authored or inherited from a seed.

Folder names are frozen here and recorded in the sheet. Editing a label never
moves a directory: renaming a heading would otherwise show up in version
control as hundreds of deletes and adds and would conflict with every open
branch.

Run:  python3 tools/seed_tree.py   — and only ever in an empty tree; see THE REFUSAL below.
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from seed_helpers import *          # noqa: F401,F403
from seed_helpers import NODES, LAYERS, SEEDED, esc, toml_str

# =============================================================================
# THE FOUR LAYERS
#
# Exactly one node crosses between any two layers. A subsystem is reached
# through its interface node and never by reaching into it, which is what makes
# a layer something you can reason about on its own.
# =============================================================================
layer("root", "VLEO multipayload programme", "", "systems", 0, box=True, tone="slate")

# --- LAYERS 1 AND 2 · FROM THE DOCUMENT --------------------------------------
#
# Every row of the management and system layers is CD-06's, read out of
# `cd06/tree.json` rather than written here. 228 rows and 367 rows, with the
# 82, 78, 95 and 245 edges between them. Nothing in this file names a variable
# those layers do not already contain.
#
# The three graphs arrive already separated, which is the thing that makes the
# document usable as a source at all: ED_* is group to group, VE is variable to
# variable, KE is variable to KPI. They are never merged.
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from cd06_rows import (Cd06, install as cd06_install, install_edges as cd06_edges,
                        LAYER3_SOURCE)
import crate_skeleton

CD = Cd06()
cd06_install(CD, layer, S)



# The rows, one module per subsystem. Import order is authoring order,
# which is the order they read in on the tree.
from nodes import (          # noqa: E402,F401
    env,
    orbit,
    aero,
    power,
    thermal,
    gnc,
    comms,
    payload,
    mass,
    mission,
    cost,
    closure,
)

# The specified rows move into layer 3, where the decomposition they belong to
# lives.
#
# This is the correction the document forced. These rows were authored against
# a flat set of headings, then re-parented into a layer 2 that was mine; CD-06's
# layer 2 is coarser than they are. It holds "Drag coefficient" as one row and
# this repository decomposes that into dynamic pressure, speed ratio, frontal
# area and drag force — which is exactly what CD-06 pushes down into layer 3:
# nineteen targets from layer 2 and ninety-five rows to answer them.
#
# So layer 2 becomes the document's 367 rows and nothing else, and every one of
# these 250 becomes what it always was: the beginning of a subsystem layer's own
# working. Nothing is discarded, no relation moves and no fixture changes — the
# derivation graph is untouched. It is a change of parent and of layer.
REPARENT = {
    # heading                 subsystem layer it decomposes
    "prop_intake":            "l3_prop",
    "prop_thruster":          "l3_prop",
    "aero_drag":              "l3_massaero",
    "mass_aero":              "l3_massaero",
    "payload":                "l3_payload",
    "pay_optical":            "l3_payload",
    "pay_rf":                 "l3_payload",
    "power":                  "l3_power",
    "thermal":                "l3_thermal",
    "comms":                  "l3_ttc",
    "gnc":                    "l3_acs",
    # Two layers the document does not have. Orbit geometry, the space
    # environment and mission performance are layer-2 groups in CD-06, not
    # subsystem layers, and this repository decomposes all three; closure and
    # cost are ours entirely. They are kept, in layers named as additions, so
    # that nothing here is mistaken for a row the document specifies.
    "space_env":              "l3_x_envorbit",
    "orbit_geometry":         "l3_x_envorbit",
    "mission":                "l3_x_envorbit",
    "closure":                "l3_x_closure",
    "mgt_cost":               "l3_x_closure",
    "mgt_customer":           "l3_x_closure",
    "applications":           "l3_x_closure",
}

# How many rows each subsystem layer already holds, so the unnamed remainder is
# the document's total less the work that exists rather than on top of it.
SPECIFIED_INTO = {}
for _n in NODES:
    _g = REPARENT.get(_n["parent"])
    if _g:
        SPECIFIED_INTO[_g[3:]] = SPECIFIED_INTO.get(_g[3:], 0) + 1


# The two layers CD-06 does not have, named so that nobody mistakes them for
# rows it does. The user asked for the document's variables; these are the
# ones this repository brought with it, kept rather than dropped.
layer("l3_x_envorbit", "Environment and orbit — addition", "root", "environment", 3,
      box=True, tone="amber",
      relates=[("l3_x_envorbit", CD.nid["orb"],
                "it decomposes orbit geometry and the space environment, which CD-06 holds at layer 2")])
S("l3_x_envorbit_interface", "Environment and orbit — subsystem interface",
  "l3_x_envorbit", "envorbit", "environment", 3, kind="required", crosses=CD.nid["orb"])

layer("l3_x_closure", "Closure and cost — addition", "root", "systems", 3,
      box=True, tone="amber",
      relates=[("l3_x_closure", CD.nid["svc"],
                "required against achieved, for the service the customer buys")])
S("l3_x_closure_interface", "Closure and cost — subsystem interface",
  "l3_x_closure", "closure", "systems", 3, kind="required", crosses=CD.nid["svc"])


# =============================================================================
# LAYER 3 · THE FIFTEEN SUBSYSTEM LAYERS
#
# The document specifies these by shape and never names their rows: which
# layers exist, how many targets each takes from layer 2, and how many rows
# each holds. Three of those four are enough to build the layer honestly.
#
# The targets are derivable, and they are real names rather than placeholders:
# a target equals the parent's variable, one for one, and every one of the
# fifteen target counts in the document equals the variable count of the
# layer-2 group it reports to. Nineteen targets for propulsion because layer 2
# holds nineteen propulsion variables. So each target row is that variable's
# name, mirrored by an achieved row, and the layer closes when achieved meets
# required.
#
# What is left over is the layer's own working — the rows nobody has named. Those
# stay `to be named`, because a plausible label on an unspecified row is the one
# thing that would make this tree lie.
# =============================================================================
for sid, slabel, sowner, n_target, n_total, src_group in LAYER3_SOURCE(CD):
    gid = "l3_%s" % sid
    reports_to = CD.nid[src_group]
    layer(gid, slabel, "root", sowner, 3, box=True, tone="teal",
          relates=[(gid, reports_to,
                    "the subsystem's only route to the system is its interface node")])
    # The one node that crosses. Nothing else in this layer is visible from
    # outside it, and nothing outside it is visible from within.
    S("l3_%s_interface" % sid, "%s — subsystem interface" % slabel, gid, sid, sowner, 3,
      kind="required", crosses=reports_to)
    for i, (cd_id, vlabel) in enumerate(CD.variables_of(src_group), 1):
        S("l3_%s_req_%02d" % (sid, i), vlabel, gid, sid, sowner, 3, kind="required")
        S("l3_%s_ach_%02d" % (sid, i), vlabel, gid, sid, sowner, 3, kind="achieved")
    # The document's total for the layer, less its interface, its targets and
    # its achieved rows, less whatever this repository has already specified
    # into it. A layer already finer than the document's estimate gets none.
    budget = n_total - 1 - 2 * n_target - SPECIFIED_INTO.get(sid, 0)
    if budget > 0:
        unnamed("l3_%s_n" % sid, gid, sid, sowner, 3, budget, "%s · internal" % slabel)





def amend(nid, **kw):
    """Rewire a node after the fact.

    Used only here, and only to close the power-thermal-propulsion loop after
    both of its ends have been declared. Every amendment is visible in this
    file, which is the point: a sheet that was quietly edited by a later pass
    is a sheet nobody can review.
    """
    for n in NODES:
        if n["id"] == nid:
            n.update(kw)
            return
    raise SystemExit("amend: no node %s" % nid)


# The array's end-of-life output now carries the temperature derating, which is
# what turns three independent chains into one system.
amend("pwr_array_power_eol",
      inputs=[("p", "pwr_array_power_bol"), ("f", "pwr_degradation"), ("ft", "pwr_cell_derating")],
      expression="P_eol = P_bol*f_deg*f_T",
      steps=[("apply both the ageing degradation and the temperature derating to the beginning-of-life output",
              "pe", "Power", "power::array_power_eol(p, f) * ft.get()")])

# The bus demand sees what propulsion actually drew, not what it asked for.
amend("pwr_demand",
      inputs=[("pp", "prop_delivered_bus_power"), ("pay", "pay_power"), ("av", "pwr_avionics_load"),
              ("com", "com_tx_power"), ("th", "pwr_thermal_load"), ("lh", "pwr_harness_loss")])

# And the closure compares delivered thrust against drag.
amend("prop_thrust_to_drag",
      inputs=[("t", "prop_delivered_thrust"), ("d", "aero_drag_force")])


# =============================================================================
# KPIs — required against achieved
#
# The closure rule, repeated identically at every boundary. A required row and
# an achieved row, compared the same way whether the boundary is customer to
# system or system to subsystem. That repetition is one of the five ideas here
# that is ours and unproven: the standards do this at review gates, not
# continuously.
# =============================================================================
def KPI(nid, label, req_value, req_unit, req_ty, achieved_node, sense, q, src,
        req_lo, req_hi, req_rlo, req_rhi, note=""):
    D(nid + "_required", label + " — required", "kpi", "mgt_customer", req_ty, req_unit,
      "R_req", req_value, req_lo, req_hi, req_rlo, req_rhi,
      "What does the customer require for " + label.lower() + "?",
      src, "systems", "A. Rai / 2026-09-01", tier="A")
    C(nid, label + " — closure", "kpi", "applications", "Ratio", "One", "M",
      q, "M = (achieved - required)/required, in the sense the requirement is stated",
      src, "systems",
      [("req", nid + "_required"), ("ach", achieved_node)],
      [("compare achieved against required in the declared sense and return the signed fractional margin",
        "m", "Ratio",
        "Ratio::new(mission::closure(req.get(), ach.get(), mission::Sense::%s).margin)" % sense)],
      -100.0, 1000.0,
      "a margin below -10000% means the inputs are not the ones the requirement was written against",
      "a margin above 100000% means the requirement is not the one that binds",
      tier="A", kind="kpi", note=note)


KPI("kpi_revisit", "Mean revisit time", 6.0, "Hour", "Time", "mis_revisit", "AtMost",
    "Does the constellation revisit often enough?", "orbitt_case_c1",
    0.1, 720.0, "below six minutes no constellation in this class achieves it",
    "above 30 days the service is not the one being sold")

KPI("kpi_gsd", "Ground sample distance", 1.0, "Metre", "Length", "pay_gsd", "AtMost",
    "Is the imagery sharp enough?", "orbitt_case_c1",
    0.05, 100.0, "below 5 cm no optical payload in this mass class achieves it",
    "above 100 m the product is not the one being sold")

KPI("kpi_geolocation", "Emitter geolocation accuracy", 500.0, "Metre", "Length",
    "pay_geolocation_error", "AtMost",
    "Are emitters located accurately enough?", "orbitt_case_c2",
    1.0, 100000.0, "below one metre no time-difference system in this class achieves it",
    "above 100 km the product carries no information")

KPI("kpi_latency", "End to end latency", 30.0, "Minute", "Time", "mis_latency", "AtMost",
    "Does the product reach the customer fast enough?", "orbitt_case_c1",
    1.0, 2880.0, "below a minute no ground segment in this design delivers",
    "above two days the product is not the near-real-time one being sold")

KPI("kpi_availability", "Service availability", 0.99, "One", "Ratio", "mis_availability", "AtLeast",
    "Is the service available often enough?", "orbitt_case_c1",
    0.5, 0.99999, "below 50% the service is unavailable more often than not",
    "above five nines no constellation in this class has demonstrated it")

KPI("kpi_data_volume", "Daily downlink volume", 500.0, "Gigabit", "DataVolume",
    "com_daily_volume", "AtLeast",
    "Does enough data reach the ground each day?", "orbitt_case_c1",
    1.0, 1.0e7, "below one gigabit a day the mission returns nothing useful",
    "above 10 petabits a day the ground segment is not the one costed")

KPI("kpi_service_lifetime", "Service lifetime", 5.0, "Year", "Time",
    "orbit_lifetime_uncontrolled", "AtLeast",
    "Does the satellite stay up long enough with no propulsion at all?", "orbitt_case_c1",
    0.1, 25.0, "below five weeks the satellite is not a mission",
    "above 25 years the debris mitigation standard is violated",
    note="Deliberately measured against the uncontrolled lifetime rather than the propelled one, so the answer does not depend on the propulsion system working.")

KPI("kpi_cost_per_year", "Cost per operational year", 45.0, "MillionUsDollar", "Money",
    "cost_per_year", "AtMost",
    "Does a year of service cost what the business case allows?", "orbitt_case_c1",
    0.1, 10000.0, "below 100 thousand a year no programme of this size runs",
    "above 10 billion a year the programme is not the one being designed")

KPI("kpi_thrust_margin", "Thrust to drag closure", 1.0, "One", "Ratio",
    "prop_thrust_to_drag", "AtLeast",
    "Does thrust exceed drag — does the design close at all?", "romano2021",
    0.1, 5.0, "a required ratio below 0.1 is not a closure requirement",
    "above 5 the requirement is not one an air-breathing design is written against",
    note="The one closure the whole programme exists to reach. Everything else is a detail until this is at or above zero margin.")

KPI("kpi_power_margin", "Power margin", 0.20, "One", "Ratio", "pwr_margin", "AtLeast",
    "Is there enough power, with the margin the standard requires?", "ecss_e_st_10_02",
    0.0, 1.0, "a negative required margin is not a requirement",
    "above 100% required margin the design is being asked to double its power for no stated reason")

KPI("kpi_thermal_margin", "Thermal margin", 10.0, "Kelvin", "Temperature", "thm_margin", "AtLeast",
    "Is the spacecraft comfortably inside its temperature limit?", "ecss_e_st_10_02",
    0.0, 100.0, "a negative required margin is not a requirement",
    "above 100 K the requirement is not one this design is written against")

KPI("kpi_mass_margin", "Mass margin", 0.10, "One", "Ratio", "mass_margin", "AtLeast",
    "Does the spacecraft fit its mass allocation with margin?", "ecss_e_st_10_02",
    0.0, 0.9, "a negative required margin is not a requirement",
    "above 90% required margin the allocation is nine times the design")


# =============================================================================
# THE EMITTER
# One folder per node. Everything about a node is in one directory: adding one
# is a copy, deleting one is a remove, its history is the log of a directory,
# and ownership is a path rule.
# =============================================================================
# =============================================================================
# ONE CRATE PER OWNER
#
# A node's folder lives in the crate of the thing that owns it: the management
# layer, the system layer, or one of the subsystem layers. Not its discipline —
# `aero` and `mass` are both answering the mass-and-aero layer's targets, and a
# crate split along discipline puts one layer's rows in three places.
#
# This is the isolation rule made enforceable rather than stated. Inside a
# single crate `use crate::prop::…` from `power` compiles and nothing stops it;
# across crates it is a manifest line, and a sibling a crate did not declare
# will not build. The faces still take one dependency on the facade, so the
# compiler still sees independent units and still builds them in parallel.
#
# The set is derived from the tree rather than listed, and the skeletons, the
# workspace members and the facade's dependencies are all generated from it.
# A hand-maintained list of crates is a list that drifts from the tree it is
# supposed to mirror, silently, until somebody notices a layer whose rows are
# in two places.
# =============================================================================

# The addition layers keep readable crate names; that they are additions is
# said on the tree, where a reader is, rather than encoded in a path.
CRATE_ALIAS = crate_skeleton.ALIAS


def owner_crate(n):
    """The crate a node belongs in, from the layer that owns it."""
    if n["layer"] == 1:
        return "vleo-mod-management"
    if n["layer"] == 2:
        return "vleo-mod-system"
    g = n["parent"]
    seen = set()
    while g and g in LAYERS and not g.startswith("l3_") and g not in seen:
        seen.add(g)
        g = LAYERS[g]["parent"]
    if not g or not g.startswith("l3_"):
        raise SystemExit("%s: layer %s but no subsystem layer above it (parent %s)"
                         % (n["id"], n["layer"], n["parent"]))
    sid = g[3:]
    return "vleo-mod-%s" % CRATE_ALIAS.get(sid, sid)


def folder_for(n):
    """The folder is the identifier. Always, for every row.

    This used to be three rules — seeded rows kept their whole id, KPI rows
    kept theirs, and everything else had its subsystem prefix stripped — and
    the third one collided the moment two disciplines answered the same layer:
    `pwr_margin` and `thm_margin` both wanted `margin`. One rule cannot
    collide, because identifiers cannot.

    It is also what the repository already says everywhere else: one small
    question, one answer, one folder, one row, and the variable id *is* the
    node id. A folder named for a stripped-down version of the id was the one
    place that was not true.
    """
    return n["id"]


def crate_for(n):
    return owner_crate(n)


def write(path, text):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w") as f:
        f.write(text)


def emit_node(n):
    crate = crate_for(n)
    folder = folder_for(n)
    base = os.path.join(ROOT, "crates", crate, "nodes", folder)
    n["_crate"] = crate
    n["_folder"] = folder
    L = []
    a = L.append
    a("# The sheet. This is the only file in this folder that must be written by")
    a("# hand, and every other artefact here is generated from it. A hand edit")
    a("# outside a numbered HOLE block in model.rs fails the regeneration diff.")
    a('id = %s' % toml_str(n["id"]))
    a('label = %s' % toml_str(n["label"]))
    a('folder = %s   # frozen at seed; renaming the label never moves the directory' % toml_str(folder))
    a('subsystem = %s' % toml_str(n["subsystem"]))
    a('parent = %s' % toml_str(n["parent"]))
    a('kind = %s' % toml_str(n["kind"]))
    a('owner = %s' % toml_str(n["owner"]))
    a('tier = %s' % toml_str(n["tier"]))
    a('layer = %d   # 1 management · 2 the system · 3 subsystem · 4 the run' % n["layer"])
    a('# Where this row sits among its siblings, as whoever wrote them ordered')
    a('# them. Folders sort alphabetically and a tree that reads alphabetically')
    a('# is a tree nobody wrote: `Achievable lifetime` before `Specific impulse`')
    a('# reverses the order the source put them in, and the order is part of')
    a('# what the source said.')
    a('order = %d' % n["order"])
    if n["crosses_to"]:
        a('# The single node that crosses between this layer and the one above.')
        a('# A subsystem is reached through its interface node, never by reaching')
        a('# into it, and that is what makes a layer something you can reason')
        a('# about on its own.')
        a('crosses_to = %s' % toml_str(n["crosses_to"]))
    a('state = %s' % toml_str(n["state"]))
    a("")
    if n["state"] == "empty":
        a("# SEEDED — the folder exists and nothing has been specified in it.")
        a("# Every field below is required before anything can be generated. An")
        a("# open field blocks generation, which is the mechanism: ambiguity")
        a("# becomes a blocking item on an engineer's screen rather than")
        a("# something an implementer resolves silently.")
        a("")
    a("[question]")
    a('text = %s' % toml_str(n["question"]))
    if n["note"]:
        a('note = %s' % toml_str(n["note"]))
    a("")
    a("[maths]")
    a('expression = %s' % toml_str(n["expression"]))
    a('source = %s' % toml_str(n["source"]))
    a("")
    for text, fails in n["assumptions"]:
        a("[[assumption]]")
        a('text = %s' % toml_str(text))
        a('fails_when = %s' % toml_str(fails))
        a("")
    a("[output]")
    a("# The node's one answer. The variable id is the node id: one small")
    a("# question, one answer, one folder, one row on the tree.")
    if n["state"] == "empty":
        a("# A unit is a decision, not data. None of these is filled yet.")
    a('symbol = %s' % toml_str(n["symbol"]))
    a('type = %s' % toml_str(n["ty"]))
    a('unit = %s' % toml_str(n["unit"]))
    a('lower = %r' % float(n["lo"]))
    a('upper = %r' % float(n["hi"]))
    a('reason_lower = %s' % toml_str(n["rlo"]))
    a('reason_upper = %s' % toml_str(n["rhi"]))
    a("")
    if n["kind"] == "declared" and n["value"] is not None:
        a("[value]")
        a("# A number a person picked. It carries the same discipline as a")
        a("# fixture, because every margin in the design is built out of these.")
        a('number = %r' % float(n["value"]))
        a('confirmed_by = %s' % toml_str(n["confirmed"]))
        a("")
    for name, var in n["inputs"]:
        a("[[input]]")
        a("# The consumer declares what it expects. Assembly refuses a mismatch")
        a("# against what the producer publishes — which is the only place the")
        a("# whole graph is visible and therefore the only place to ask.")
        a('binding = %s' % toml_str(name))
        a('var = %s' % toml_str(var))
        a('type = %s' % toml_str(BY_ID[var]["ty"]))
        a("")
    for i, (text, binds, ty, _body) in enumerate(n["steps"], 1):
        a("[[algorithm.step]]")
        a('number = %d' % i)
        a('text = %s' % toml_str(text))
        a('binds = %s' % toml_str(binds))
        a('type = %s' % toml_str(ty))
        a("")
    if n["kpis"]:
        a("[contributes]")
        a('kpis = [%s]' % ", ".join(toml_str(k) for k in n["kpis"]))
        a("")
    if n["bundles"]:
        a("[data]")
        a('bundles = [%s]' % ", ".join(toml_str(b) for b in n["bundles"]))
        a("")
    kind, opts = n["view"]
    a("[view]")
    a('kind = %s' % toml_str(kind))
    for k, v in opts.items():
        a('%s = %s' % (k, toml_str(v) if isinstance(v, str) else repr(v)))
    a("")
    write(os.path.join(base, "node.toml"), "\n".join(L) + "\n")

    # The holes: the only lines a person or an agent types into the code.
    if n["steps"]:
        H = ["// Seeded hole bodies. `cargo xtask docs` regenerates everything",
             "// around these markers and splices the bodies back in unchanged.",
             "// A hand edit outside a HOLE block fails the regeneration diff.",
             ""]
        for i, (text, binds, ty, body) in enumerate(n["steps"], 1):
            H.append("// ---- HOLE %d : %s -> %s" % (i, text, ty))
            H.append("let %s: %s = %s;" % (binds, ty, body))
            H.append("// ---- end HOLE %d" % i)
            H.append("")
        write(os.path.join(base, "model.rs"), "\n".join(H) + "\n")

    F = ["# Known-good values, and where each came from.",
         "#",
         "# The one rule: an expected value may never be produced by the code",
         "# under test. The schema refuses a row whose provenance is the",
         "# implementation, because a number worked out by the thing being",
         "# checked proves nothing.",
         ""]
    for label, inputs, expect, tol, prov, src in n["fixtures"]:
        F.append("[[fixture]]")
        F.append('label = %s' % toml_str(label))
        F.append('expect = %r' % float(expect))
        F.append('tolerance = %r' % float(tol))
        F.append('provenance = %s' % toml_str(prov))
        F.append('source = %s' % toml_str(src))
        F.append("inputs = { %s }" % ", ".join("%s = %r" % (k, float(v)) for k, v in inputs.items()))
        F.append("")
    write(os.path.join(base, "fixtures.toml"), "\n".join(F) + "\n")


SOURCES = [
    ("jacchia1971", "Jacchia, L. G. — Revised static models of the thermosphere and exosphere with empirical temperature profiles", "SAO Special Report 332, 1971", "current",
     "The temperature profile, the exospheric temperature relation and the diffusive-equilibrium structure the atmosphere node implements."),
    ("us_std_1976", "U.S. Standard Atmosphere, 1976", "NOAA-S/T 76-1562", "current",
     "The 120 km base state the diffusive-equilibrium integration starts from, and the collision cross-section."),
    ("sentman1961", "Sentman, L. H. — Free molecule flow theory and its application to the determination of aerodynamic forces", "Lockheed LMSC-448514, 1961", "current",
     "The free-molecular drag and lift coefficients for a surface element."),
    ("doornbos2011", "Doornbos, E. — Thermospheric density and wind determination from satellite dynamics", "TU Delft, 2011", "current",
     "The form of Sentman's relations used here, and the published residual of empirical thermosphere models against measured drag."),
    ("moe2005", "Moe, K. and Moe, M. M. — Gas-surface interactions and satellite drag coefficients", "Planetary and Space Science 53, 2005", "current",
     "The Langmuir adsorption model for energy accommodation, and why accommodation moves with solar activity."),
    ("romano2021", "Romano, F. et al. — Intake design for an atmosphere-breathing electric propulsion system", "Acta Astronautica, 2021", "current",
     "The intake flux balance, the compression ratio and collection efficiency definitions, and the thruster efficiency chain."),
    ("vallado2013", "Vallado, D. A. — Fundamentals of Astrodynamics and Applications, 4th edition", "Microcosm Press, 2013", "current",
     "The two-body and J2 secular relations, the decay relation and the rocket equation."),
    ("larson_wertz", "Wertz, J. R., Everett, D. F. and Puschell, J. J. — Space Mission Engineering: The New SMAD", "Microcosm Press, 2011", "current",
     "The access geometry, the link budget, the power and thermal budgets, the disturbance torques and the payload relations."),
    ("wgs84", "NIMA TR8350.2 — World Geodetic System 1984", "3rd edition, 2000", "current",
     "The Earth equatorial radius, flattening, gravitational parameter and J2."),
    ("codata2018", "CODATA 2018 recommended values of the fundamental physical constants", "NIST, 2019", "current",
     "Boltzmann's constant, the gas constant, Avogadro's number, the elementary charge and the atomic mass unit."),
    ("igrf2020", "International Geomagnetic Reference Field, 13th generation", "IAGA, 2020", "current",
     "The dipole moment used in the magnetic field approximation."),
    ("iso24113", "ISO 24113 — Space systems: space debris mitigation requirements", "ISO, 2019", "current",
     "The disposal requirement that makes deorbit delta-v a design variable rather than an afterthought."),
    ("ecss_e_st_10_02", "ECSS-E-ST-10-02C — Verification", "ESA-ESTEC, 2018", "current",
     "The margin philosophy, the mass growth allowances and the required-against-achieved closure convention."),
    ("ecss_e_st_50", "ECSS-E-ST-50-05C — Radio frequency and modulation", "ESA-ESTEC, 2011", "current",
     "The link margin convention."),
    ("ccsds131", "CCSDS 131.0-B — TM synchronization and channel coding", "CCSDS, 2021", "current",
     "The required energy per bit for the assumed coded modulation."),
    ("ccsds122", "CCSDS 122.0-B — Image data compression", "CCSDS, 2017", "current",
     "The compression ratio range the imagery budget is written against."),
    ("itu_p618", "ITU-R P.618 — Propagation data and prediction methods for earth-space telecommunication systems", "ITU, 2023", "current",
     "The atmospheric and rain attenuation the link budget carries."),
    ("itu_sa1862", "ITU-R SA.1862 — Frequency bands for Earth exploration-satellite service downlinks", "ITU, 2010", "current",
     "The X-band downlink allocation the frequency is chosen inside."),
    ("noaa_swpc", "NOAA Space Weather Prediction Center — solar and geomagnetic indices", "NOAA, retrieved 2026-09-01", "current",
     "The F10.7 and Kp drivers, distributed as the solar-drivers bundle."),
    ("nist_asd", "NIST Atomic Spectra Database, ionisation energies", "NIST, 2023", "current",
     "The first ionisation energies that set the floor on the energy an ABEP thruster spends per ion."),
    ("ciaaw2021", "CIAAW — Standard atomic weights", "IUPAC, 2021", "current",
     "The molar masses of the atmospheric species."),
    ("nasa_cer", "NASA Cost Estimating Handbook, version 4.0", "NASA, 2015", "current",
     "The form of the cost estimating relations and the learning curve convention."),
    ("modtran_ref", "At-aperture radiance reference case, mid-latitude summer, 30 degree solar zenith", "MODTRAN 6 run, 2026-08-14", "current",
     "The scene radiance the optical signal-to-noise chain is evaluated at."),
    ("orbitt_case_c1", "Orbitt Space — VLEO multipayload reference case C1", "internal, 2026-09-01", "current",
     "Every declared value that is a programme decision rather than a physical constant."),
    ("orbitt_case_c2", "Orbitt Space — VLEO multipayload reference case C2", "internal, 2026-09-01", "current",
     "The comms and PNT case's declared values."),
    ("matlab_legacy", "Orbitt Space — legacy MATLAB feasibility tool, 364 files", "internal, 2025", "current",
     "An independent implementation, used as a second opinion. Its outputs are a parity grid, never a fixture: an implementation cannot supply its own expected values, and this one is an implementation."),
]

CASES = [
    ("nominal", "Nominal — case C1 at the reference altitude",
     "The design point every dossier in this suite is checked against. Case A of "
     "the two reference cases: 250 km, integrated ISR/EO and IoT comms "
     "multipayload, global service, constellation concept.",
     {}),
    ("low_altitude", "Case A — 200 km, drag dominated",
     "Drag-dominated and air-breathing propulsion in play. The propulsion layer "
     "carries thrust-to-drag as its binding target.",
     {"orbit_altitude": 200000.0}),
    ("high_altitude", "Case B — 350 km, downlink dominated",
     "Drag is reduced and both chemical and air-breathing propulsion close. The "
     "propulsion targets relax and downlink and revisit dominate.",
     {"orbit_altitude": 350000.0}),
    ("solar_max", "Solar maximum, storm conditions",
     "The activity level the design must survive rather than the one it is sized "
     "at. Density at 250 km is roughly five times the quiet value.",
     {"env_f107": 250.0, "env_f107a": 250.0, "env_kp": 7.0}),
    ("solar_min", "Solar minimum, quiet conditions",
     "The activity level at which the air-breathing concept is hardest, because "
     "there is least air to breathe.",
     {"env_f107": 70.0, "env_f107a": 70.0, "env_kp": 1.0}),
]

CYCLE = dict(
    nodes=["pwr_cell_derating", "pwr_array_power_eol", "pwr_available", "prop_throttle",
           "prop_delivered_bus_power", "pwr_demand", "thm_dissipation",
           "thm_equilibrium_temperature"],
    converge_on="pwr_demand", tolerance=1e-6, max_iter=80,
    seeds={"thm_equilibrium_temperature": 300.0},
)


def emit_supporting():
    # ---- sources -----------------------------------------------------------
    S = ["# Sources are objects, not strings.",
         "#",
         "# A fixture or a declared value references an identifier here, never a",
         "# free-text citation. Marking one superseded then lists every node that",
         "# depends on it — one query rather than a search.",
         ""]
    for sid, title, where, status, used in SOURCES:
        S += ["[[source]]", "id = %s" % toml_str(sid), "title = %s" % toml_str(title),
              "where = %s" % toml_str(where), "status = %s" % toml_str(status),
              "used_for = %s" % toml_str(used), ""]
    write(os.path.join(ROOT, "sources", "sources.toml"), "\n".join(S) + "\n")

    # ---- layers ------------------------------------------------------------
    tops = {}
    for gid, g in LAYERS.items():
        top = gid
        seen = set()
        while LAYERS.get(top, {}).get("parent") and top not in seen:
            seen.add(top)
            top = LAYERS[top]["parent"]
        tops.setdefault(top if top != "root" else gid, []).append(gid)
    for top, members in sorted(tops.items()):
        L = ["# A layer file: the rows in the tree that are not nodes.",
             "#",
             "# Headings, their parents, the relation edges between groups, and",
             "# subsystem ownership — which CODEOWNERS is generated from. Changed",
             "# rarely and reviewed by two people, because the tree is the",
             "# decomposition and moving a branch moves everyone's work.",
             ""]
        for gid in sorted(members):
            g = LAYERS[gid]
            L += ["[[group]]", "id = %s" % toml_str(g["id"]), "label = %s" % toml_str(g["label"]),
                  "parent = %s" % toml_str(g["parent"]), "owner = %s" % toml_str(g["owner"]),
                  "layer = %d" % g["layer"],
                  "order = %d" % g["order"],
                  "box = %s   # drawn as a nested box on the diagonal" % ("true" if g["box"] else "false"),
                  "tone = %s" % toml_str(g["tone"]),
                  "cases = [%s]   # empty means every case" %
                  ", ".join(toml_str(c) for c in g["cases"]), ""]
        for gid in sorted(members):
            for frm, to, why in LAYERS[gid]["relates"]:
                L += ["[[relates]]", "from = %s" % toml_str(frm), "to = %s" % toml_str(to),
                      "why = %s" % toml_str(why), ""]
        write(os.path.join(ROOT, "layers", "%s.toml" % top), "\n".join(L) + "\n")

    # ---- cases -------------------------------------------------------------
    for cid, label, note, supplied in CASES:
        L = ["# A case: per-customer values against one shared architecture.",
             "#",
             "# The architecture is never copied. What the customer did not buy is",
             "# simply not in play, and n copies would mean n fixes and silent",
             "# drift — the anti-clone-and-own rule, ISO/IEC 26580.",
             "",
             "id = %s" % toml_str(cid), "label = %s" % toml_str(label),
             "note = %s" % toml_str(note), ""]
        if supplied:
            L.append("[supply]")
            L.append("# Overrides on top of each declared node's own value.")
            for k, v in sorted(supplied.items()):
                L.append("%s = %r" % (k, float(v)))
            L.append("")
        L += ["[[iterate]]",
              "# A declared cycle. Power becomes heat, heat sets array temperature,",
              "# array temperature sets cell efficiency, cell efficiency sets available",
              "# power, and available power throttles the largest load on the bus.",
              "# A cycle is a property of the design, so it is declared where design",
              "# decisions live. An undeclared one is a named error, not a hung run.",
              "nodes = [%s]" % ", ".join(toml_str(x) for x in CYCLE["nodes"]),
              "converge_on = %s" % toml_str(CYCLE["converge_on"]),
              "tolerance = %r" % CYCLE["tolerance"],
              "max_iter = %d" % CYCLE["max_iter"], ""]
        for k, v in CYCLE["seeds"].items():
            L += ["[[iterate.seed]]", "var = %s" % toml_str(k), "value = %r" % float(v),
                  "source = \"orbitt_case_c1\"",
                  "note = \"A seed decides which fixed point the iteration reaches, so it carries a source like every other declared number.\"", ""]
        write(os.path.join(ROOT, "cases", "%s.toml" % cid), "\n".join(L) + "\n")


BY_ID = {}


# =============================================================================
# THE REFUSAL
#
# The docstring above says "run once, ever". Nothing enforced that, and on
# 2026-09-14 this script was run a second time by a sweep that passed it a flag
# it does not take. It ignored the flag, reseeded the tree, and in one write
# reverted 138 hand-filled `model.rs` bodies to their seed hole bodies, dropped
# a `sense` line from twelve KPI rows, reset a written sheet to
# `state = "empty"`, deleted a source that a node still cites, and unregistered
# an owner crate from `vleo-modules` — because the crate list it writes is the
# crate list it knows, and a crate added after the seed is not in it.
#
# Nothing was lost, because everything was committed or recoverable. That was
# luck, not design. A rule that only exists in a docstring is a rule that gets
# broken by a loop, so it is checked here instead: a stated intention with no
# mechanism behind it is not a safeguard.


def _refuse_unless_the_tree_is_unwritten(argv):
    """Stop, unless this really is a bootstrap of an empty tree.

    Two refusals, because two different mistakes reach this line. An argument
    this script does not take means the caller thinks it is some other tool,
    and every argument is unknown because it takes none. A tree that already
    holds written sheets means the seed has run, and running it again is not a
    no-op: it overwrites by content, so it un-writes whatever a person wrote.
    """
    if argv:
        raise SystemExit(
            "seed_tree.py takes no arguments and was given %s.\n"
            "It is not the tool you are looking for: it seeds the tree, it "
            "does not test, lint or report on it. Every other script in "
            "tools/ answers --selftest; this one answers nothing, because it "
            "is a bootstrap." % " ".join(argv))

    written = []
    for crate in sorted(os.listdir(os.path.join(ROOT, "crates"))):
        nodes = os.path.join(ROOT, "crates", crate, "nodes")
        if not os.path.isdir(nodes):
            continue
        for folder in sorted(os.listdir(nodes)):
            sheet = os.path.join(nodes, folder, "node.toml")
            if not os.path.isfile(sheet):
                continue
            with open(sheet, encoding="utf-8") as f:
                if 'state = "published"' in f.read():
                    written.append("%s/%s" % (crate, folder))
    if written:
        shown = ", ".join(written[:3])
        raise SystemExit(
            "refusing to seed: %d sheet(s) in this tree are published "
            "(%s%s).\n"
            "The seed writes every sheet and every hole body from its own "
            "tables, so running it here would overwrite work a person did by "
            "hand. This script ran once, when the tree was empty, and the "
            "sheets have been the source ever since.\n"
            "If you genuinely mean to bootstrap an empty tree, do it in an "
            "empty tree." % (len(written), shown,
                             ", ..." if len(written) > 3 else ""))


def main():
    _refuse_unless_the_tree_is_unwritten(sys.argv[1:])
    # Creation order is authoring order: the document's own order for the rows
    # that came from it, and the order they were written in for the rest.
    for i, n in enumerate(NODES):
        n["order"] = i
    for n in NODES:
        if n["parent"] in REPARENT:
            n["parent"] = REPARENT[n["parent"]]
            n["layer"] = 3
        BY_ID[n["id"]] = n
    edges = cd06_edges(CD, LAYERS, BY_ID)
    print("cd06 edges: %(relation)d relation, %(derivation)d derivation, "
          "%(contribution)d contribution, %(skipped)d skipped" % edges)
    seen = {}
    for n in NODES:
        key = (crate_for(n), folder_for(n))
        if key in seen:
            raise SystemExit("folder collision: %s and %s both resolve to %s/nodes/%s"
                             % (seen[key], n["id"], key[0], key[1]))
        seen[key] = n["id"]
    for n in NODES:
        emit_node(n)
    emit_supporting()
    crates = sorted({crate_for(n) for n in NODES})
    crate_skeleton.emit(crates, LAYERS, ROOT, write,
                        open(os.path.join(ROOT, "tools", "node_crate_build.rs"),
                             encoding="utf-8").read())
    print("seeded %d node folders across %d owner crates" % (len(NODES), len(crates)))
    for c in crates:
        k = len([n for n in NODES if crate_for(n) == c])
        print("  %-22s %3d nodes" % (c, k))
    print("%d layer groups, %d sources, %d cases" % (len(LAYERS), len(SOURCES), len(CASES)))


if __name__ == "__main__":
    main()
