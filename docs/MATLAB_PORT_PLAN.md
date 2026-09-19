# Bringing the MATLAB tool across

Written after reading the uploaded `VLEO_Tool` end to end — every contract,
every area README, and the purpose line of every model function. It plans the
work; it does not do any of it.

---

## 1 · What the uploaded tool actually is

430 `.m` files in fifteen top-level areas. The first useful fact is that most
of them are not physics:

| | files | what they are |
|---|---|---|
| `model/` + `legacy/` | **114** | the physics. The only candidates for porting. |
| `views/` | 168 | drawing. Each area's README says a view **never computes**. |
| everything else | 148 | vendored kernel, bus, data paths, UI helpers, audit tools |

So the porting surface is 114 files, not 430 — and of those, `50_platform` and
`70_constellation` are largely geometry and drawing support.

### The chain, from the contracts

Every area declares its boundary in a `*_contract.m`, and those eight files are
the dependency graph without having to infer it:

```
env.{from, life_d, conf}          ← the mission window: the single input
  └─ solar    f107, f107bar, ap, kp, scen
      └─ atmos    density.rho, density.n, density.dh        (+ des.rho_scale)
          ├─ drag     Drag, Drag_mN, N_avail, Drag0_mN      (+ des.h, des.A)
          │   └─ abep    T_mN, N_ion, N_req, R, CR_total, CR_active, pump_kg
          └─ platform geom, area_panel, faces, power
constellation  planes, sats, total, coverage, revisit        (independent thread)
  └─ closure  margin, sizing, summary                        (reads req.* + abep + power)
```

This matters because `docs/RUNBOOK.md` already says to **order by thread, not
by file**. The contracts give the thread.

### Two things worth knowing before planning anything

**The original studies are kept verbatim.** `99_legacy_source/` holds
`VLEO_ABEP_Feasibility_Study_ves1.m` (329 KB) and
`VLEO_Constellation_Feasibility_Study_ves1.m` (484 KB), under checksum guard.
Everything in the tool is a refactoring of those two files, and the refactor
claims line-by-line fidelity in places (`30_drag/README.md`: *"vleo_abep_model.m
holds sections 1-5 of the original WHOLE, deliberately: splitting it would break
the line-by-line fidelity claim."*). So there are two candidate references for
any relation: the tool, and the study it came from.

**The tool already has this repository's habits.** It has contracts that fail
the build on an undeclared cross-folder call, a duplicate-name checker, a
vendored read-only kernel, self-tests inside the functions, and comments that
record why a thing is the way it is. That is a considerable advantage: the
reasons are written down, so the sheets can be filled from something better
than a reading of the arithmetic.

---

## 2 · How the breakdown will work, node by node

The procedure is already in `docs/RUNBOOK.md` under *Bringing the legacy MATLAB
across*, and the machinery for it is already in the tree: `migrated_from`,
`parity.csv`, the generated parity test, and gate check 7c. Nothing new has to
be invented; this plan is how that procedure gets applied to 114 files.

### Per node, five steps

1. **Read the MATLAB and find the real source.** The sheet is filled from the
   paper, textbook or measurement the MATLAB was built from — **never from the
   MATLAB**. The headers usually name it (`prf_ap2kp` cites the Bartels ap/Kp
   scale, IAGA; `vleo_atmosphere` cites Bruinsma & Boniface 2021). Where no
   source is named, that is a finding to raise, not a gap to paper over.
2. **Write or amend the sheet** — question, relation, symbol, type, unit,
   declared range with a reason for each bound, assumptions with what each one
   fails on. Set `migrated_from` to the function and line.
3. **Export the MATLAB's numbers over a grid into `parity.csv`** beside the
   node. A second opinion, never a fixture: an implementation cannot supply its
   own expected values and the MATLAB is an implementation.
4. **Fill the holes**, then `xtask gate <node>` and `xtask ready <node>`.
5. **A disagreement is a finding**, about one of the two implementations. Not
   something to settle by adjusting a tolerance.

### What comes out of each node

| artefact | where it comes from |
|---|---|
| the sheet | the cited source, read independently |
| `fixtures.toml` | the source — a worked example, a published table, a measurement |
| `parity.csv` | the MATLAB, run over a grid — **but see below** |
| `[maths] confirmed_by` | a person, via `xtask confirm`. Not me, ever. |

**There is no MATLAB in this environment, and the plan above assumed there was.**
Step 3 got its grid anyway, because `prf_ap2kp`'s table half is two literal
arrays and a clip, so its answer at each of its own points is readable from the
source without running it. That was luck, not method, and it is why step 3 was
the right first node.

It does not generalise. For a node whose relation is arithmetic rather than a
table, there are three options and they are not equally good:

1. **Run the MATLAB** on a machine that has it and commit the exported grid.
   This is the only one that produces what `parity.csv` claims to be, and it
   needs a person with a licence, once per node.
2. **Reimplement the MATLAB** in a third language to generate the grid. This is
   the tempting one and it is close to worthless: a grid produced by a
   reimplementation is not the prior implementation's opinion, it is a second
   copy of my own reading of it, and the two will agree for exactly the reasons
   that make the comparison uninformative.
3. **Leave `migrated_from` empty** and carry no grid, so the node stands on its
   fixtures alone. Honest, and it gives up the strongest verification asset the
   programme has — eighteen months of working code — on every node it is used
   for.

The machinery already refuses to let (3) be silent: `migrated_from` with no
`parity.csv` fails, and a `parity.csv` with no `migrated_from` fails, so a node
either claims a prior implementation and shows the comparison or claims neither.
What it cannot catch is (2), because a reimplemented grid looks exactly like an
exported one. **Which of the three applies is a decision to take before starting
a node, and to record in the node's own comments.**

### Order of attack, and why

Ordered by the contract chain rather than by file count:

| # | area | model files | lands on | note |
|---|---|---|---|---|
| 1 | `10_solar` | 3 | `env_f107`, `env_f107a`, `env_kp` | head of the chain. All three are **declared constants** in the tree today and **computed** in the MATLAB — a `kind` change, not a hole-fill. |
| 2 | `20_density` | 6 | `env_mass_density`, `env_number_density`, `env_scale_height`, … | the DTM2020 kernel is vendored Fortran-in-MATLAB; see the open question below. |
| 3 | `30_drag` | 3 | `aero_drag_force`, `aero_dynamic_pressure`, `aero_drag_coefficient`, … | smallest self-contained physics. Functions carry SelfTests. |
| 4 | `40_abep` | 7 | the 28 written `prop_*` nodes | what the tool exists for. |
| 5 | `50_platform` | 13 | `pwr_*`, some `aero_*` geometry | mostly geometry helpers; low physics density. |
| 6 | `80_closure` | 5 | `kpi_*`, the twelve closures | judgement layer, reads everything else. |
| 7 | `70_constellation` | 62 | `mis_*` | a separate thread. Deliberately last. |

Work stops after each node for review. That is the agreed cadence and it is
also what the repository's own rules want — one node per pull request.

### The first real decision, already surfaced

`env_kp` is `kind = "declared"`, `Kp = 3`, confirmed by a person. The MATLAB
computes it from daily planetary Ap. Porting it changes the node's kind, deletes
a confirmed declared value, introduces an input with no layer-3 row
(`sys_space_environment_ap` exists at **layer 2**, seeded), and changes what
`env_exospheric_temperature` depends on.

The same question lands on `env_f107` and `env_f107a`. **This is a design
authority question, not a porting one**, and it is the first thing to settle
because it sets the pattern for the whole solar area.

---

## 3 · The data plan

### What the data actually is

`vleo_data.m` is the one path resolver, and it creates exactly the tree in your
screenshot beside the tool, not inside it:

```
VLEO_Data/
  00_input/{solar,goce,indices}/   prf_study.mat, solar_weather_tables.mat
  10_cache/                        keyed cache objects
  20_runs/                         timestamped run folders
  30_exports/
  90_reference/
```

Its reasoning is the same as this repository's: *"Code is versioned, small and
shipped. Data is large, private and machine-specific."*

`prf_study.mat` is **not primary data**. `prf_build` constructs it, and
`prf_indices_fetch` downloads what it is built from — daily F10.7, SESC sunspot
number, planetary Ap and 3-hourly Kp — from
`https://www.ngdc.noaa.gov/stp/space-weather/swpc-products/`. It is a derived
cache of a published record.

### Which means this repository already has the answer written down

`bundles/solar-drivers/2026.09.04/manifest.toml` has provenance `noaa_swpc`,
holds **monthly means**, and its own note says:

> The daily record is a larger bundle with the same name and a later version,
> and syncing it changes nothing in the tool but the numbers.

That is the same source, at the resolution the MATLAB uses. The plan was
written before the MATLAB arrived; it just has not been executed.

### So: the `.mat` never enters the repository

| | |
|---|---|
| **not** committed | `prf_study.mat` is derived, large, and machine-specific |
| **not** a fixture source | it is an implementation's cache |
| **is** the input to a new bundle | `solar-drivers` at a later version, daily resolution, provenance `noaa_swpc`, licence and staleness declared, content hash by `xtask bundle publish` |

Three ways it could reach the bundle. The first was tested and is closed:

1. ~~**Fetch the published record directly.**~~ **Tested, and not available.**
   All three primary hosts are blocked by this environment's egress proxy:
   `ngdc.noaa.gov`, `services.swpc.noaa.gov` and `kp.gfz-potsdam.de` all return
   nothing. So the route where the provenance claim would be exactly true — the
   bundle built here from the published record — is closed, and route 2 is the
   best available.
2. **You export the series from MATLAB to CSV** and attach it — F10.7, F10.7bar,
   Ap, and the eight 3-hourly Kp, one row per day. The bundle then cites NOAA
   as provenance with a note saying the transcription was via `prf_study.mat`,
   which is honest and slightly weaker.
3. **You attach `prf_study.mat` itself.** Reading a v7.3 `.mat` outside MATLAB
   needs HDF5 tooling; possible, and the least attractive, because the bundle
   would then be a copy of a copy.

**Nothing about the location on your machine matters to this repository.** The
tool finds its data through `VLEO_DATA_ROOT` or `vleo_data('setroot', …)`, and
this repository finds its data through `vleo data sync` into its own store.
They never need to be the same folder, and neither needs to know where the
other is.

### The one thing the data changes about the nodes

The bias-correction half of `prf_ap2kp` is **fitted on the record**: nine Ap
bins, at least 200 days, at least 20 per bin, median residuals, nearest-bin
fill. That is not a relation and cannot become an `expression` in a sheet. It
is a derived table with a provenance — reference data, in this tool's
vocabulary — and it is the first case where the port produces a bundle rather
than a node.

---

## 4 · Open questions, in the order they block work

1. **Do the solar drivers become computed nodes, or stay declared design
   values?** Blocks node 1 and sets the pattern for the area.
2. **Where does Ap live** — the seeded layer-2 `sys_space_environment_ap`, or a
   new layer-3 row, which is a change to the tree's shape and goes through
   `layers/`?
3. **DTM2020.** `01_kernel/dtm2020_oper` is four files of vendored coefficient
   tables and a density evaluator. It is a *model* with published coefficients,
   not a relation anybody re-derives. Does it become a node, a bundle of
   coefficients plus a node, or an adopted dependency recorded in
   `ADOPTION.lock`?
4. **Which reference wins when the tool and the original study differ** — the
   refactored `model/` file, or `99_legacy_source/`? The refactor claims
   fidelity in places, which means it can be checked rather than assumed.
5. ~~**Can this environment reach NGDC?**~~ **Answered: no.** All three primary
   hosts are blocked here, so the data has to come from your machine. Route 2 —
   a CSV you export — is the plan unless you would rather I work from
   `prf_study.mat` directly.

---

# Part II · The solar-weather tab, and where it lands

Added after exploring `01_kernel/sw_study` in full — eight tabs, 38 analysis
methods, ~57 views, all reading one database.

## 6 · The shape problem, stated exactly

The tree links layer 3 to layer 2 through **one interface node per subsystem**,
carrying `crosses_to`. Group parentage does not do it:

```
l3_x_envorbit                     parent = "root"          ← correct by design
  └─ l3_x_envorbit_interface      crosses_to = "sys_orbit_and_environment"
```

`sys_orbit_and_environment` is the **parent** of `sys_space_environment`. So the
existing layer-3 group crosses to the level above space environment, and
**nothing in the tree crosses to `sys_space_environment` itself.** That is why
there is no layer-3 environment under it: not an oversight in the seed, but a
subsystem that was never given one.

The six layer-2 rows under `sys_space_environment` — solar flux, F10.7, Ap,
atmospheric density, thermospheric wind, atomic oxygen fluence — are all
`state = "empty"`. They are the answers a layer-3 group is supposed to supply,
and no group supplies them.

## 7 · What actually computes a number in that tab

38 analysis methods, but most produce *understanding* — verification, scoring,
segmentation, storm classification. The chain that produces the numbers a design
consumes is narrow:

```
prf_study.mat            10,319 days, 1997-2025, F10.7 · Ap · 8×Kp
  └─ prf_cycles          cycle boundaries, the mean cycle, 27-day recurrence
      └─ prf_design      THE SYNTHESIS — "what F10.7 and Ap do I design to,
         │                N days ahead, at what confidence"
         │   F10.7  central = persistence(today) → mean cycle,
         │           blended w = exp(−lead/27)
         │           design  = central + q-percentile of the L-day change
         │                     (empirical structure function, so the band
         │                      widens with lead as knowledge fades)
         │   Ap     not forecastable long-term → STORM RETURN PERIOD from
         │           the empirical exceedance curve (1 / 10 / 100 years)
         ├─ prf_ap2kp    Ap → Kp: Bartels/IAGA table + a bias measured on the
         │                record, because a daily Ap has to serve two senses
         │                DTM2020 wants and the table was built for neither
         └─ prf_drivers  the formal export: per-day F10.7, Ap, Kp, plus
                          .f107a_81 (81-day CENTRED mean) and ap_3h [N×8]
```

`prf_drivers`' header states the interface better than a summary could:

> The orbit propagator, the decay study and the density model all need the same
> three numbers per day — F10.7, Ap and Kp — and they need to know **which
> version** they are being given: the truth, the expected level, or a design
> percentile.

That is exactly a node's contract: one answer, and the credibility that travels
with it.

## 8 · The proposal — everything under solar flux

A new layer-3 group for solar weather, crossing to `sys_space_environment`, with
`sys_space_environment_solar_flux` as the layer-2 row it rolls up into. F10.7 and
Ap then flow **out** of it, which is where they are actually computed.

```
sys_space_environment              (layer 2 group)
  ├─ sys_space_environment_solar_flux   ← the tab's answer lands here
  ├─ sys_space_environment_f10_7        ← flows from solar flux
  ├─ sys_space_environment_ap           ← flows from solar flux
  └─ …

l3_solar_weather                   (layer 3 group, NEW, owner: environment)
  └─ l3_solar_weather_interface    crosses_to = "sys_space_environment"
```

Nodes inside it, one question each:

| node | the question it answers | from |
|---|---|---|
| `sw_cycle_phase` | Where in the solar cycle is this date? | `prf_cycles` |
| `sw_mean_cycle_level` | What F10.7 does the mean cycle expect at that phase? | `prf_cycles` |
| `sw_central_expectation` | Persistence decaying into the mean cycle, `w = exp(−L/27)` | `prf_design` |
| `sw_uncertainty_growth` | How far does F10.7 move over L days, at percentile q? | `prf_design` |
| `sw_f107_design` | **The F10.7 to design to**, at a lead and a confidence | `prf_design` |
| `sw_f107_81day` | The 81-day centred mean | `prf_drivers` |
| `sw_ap_return_level` | **The Ap that recurs once per N years** | `prf_design` |
| `sw_ap_to_kp` | Kp from daily Ap, published table | `prf_ap2kp` |

`sw_f107_design` and `sw_ap_return_level` are the two that cross upward. The rest
are the working that gets them there, and each is small enough to review.

### The one that is not a node

`prf_ap2kp`'s **bias correction** is fitted on the record — nine Ap bins, ≥200
days, ≥20 per bin, median residuals, nearest-bin fill. That is not a relation and
cannot be an `expression`. It is a derived table with a provenance: reference
data, published as part of the bundle, not computed by a node. The node
`sw_ap_to_kp` carries the published table; the offset arrives as data.

## 9 · What this does to the three declared constants

`env_f107 = 150`, `env_f107a = 150`, `env_kp = 3` stay exactly as they are for
now. They are the design point — one assumed sky — and the layer-3 solar-weather
group is what makes a *window-derived* sky available beside it. Whether layer 3's
`env_*` rows later read from layer 2 is a separate decision, taken once there is
something to read.

This ordering matters: it means the whole tab can be brought across without
touching a written node or deleting a confirmed value, and the first thing that
changes for anybody is that six empty layer-2 rows start answering.

## 10 · The data, settled

`prf_study.mat` was downloaded from Drive and opened here: **MATLAB v5**, so
`scipy.io.loadmat` reads it with no MATLAB and no HDF5 tooling.

```
PRF.meta.source     NGDC DSD/DGD annual summaries
PRF.meta.tool       prf_indices_fetch.m
span                1997-01-01 … 2025-12-31   10,319 gap-free days
PRF.observed        f107 (64…343) · ap_planetary (0…273) · kp_planetary [N×8]
```

Its own metadata carries the caveat that belongs in the bundle note: *"ap_planetary
is SWPC estimated planetary A, not GFZ definitive."*

So: a new version of the existing `solar-drivers` bundle, daily resolution,
provenance `noaa_swpc`, that caveat recorded, hashed by `xtask bundle publish`.
The `.mat` itself never enters the repository, and nothing depends on where it
sits on anybody's machine.

---

# Part III · The shape that keeps the node count down

Part II proposed eight nodes. That was the wrong instinct: it decomposed the
*working* into rows, when the working is not what the design consumes.

## 11 · Only four quantities leave the study

38 analysis methods, ~57 views, eight tabs — and the whole of it exists to
produce four numbers that anything downstream actually reads:

| what leaves | to whom |
|---|---|
| **F10.7 at a lead and a confidence** | `sys_space_environment_f10_7`, then density |
| **Ap at a storm return period** | `sys_space_environment_ap`, then Kp |
| **Kp** | what DTM2020 is actually given |
| **the 81-day centred mean** | DTM2020 carries most of its solar response on it |

Everything else — repeatability, recurrence, segmentation, forecast skill, storm
climatology — is not a separate answer. It is **the evidence that those four can
be trusted**, which in this tree has a first-class home already: the credibility
vector, and a node's own evidence page.

So: **four nodes, not eight**, and the study's substance goes where it is
checked rather than into rows that publish nothing.

## 12 · The tabs are panels, not nodes

`panels/README.md` opens with the reason this is the right home:

> A wrong number crashes a test. A wrong chart looks beautiful.

A panel declares `draws`, `reads`, `correct`, and carries a reference image with
a tolerance. `tools/panel_check.py` then drives a real browser against the real
daemon and checks three things with no person involved: **it renders**, **it
moves when each input it claims to read is changed**, and **it matches the
reference**.

That is a stronger contract than a node page would give these figures, and check
two is the one that matters here — a tab wired to nothing renders perfectly and
matches yesterday's reference every time.

One panel per tab, eight of them:

| panel | draws | the number it is evidence for |
|---|---|---|
| `sw_repeatability` | mean cycle against every cycle, Ap by cycle, storm scale | how much of F10.7 the cycle explains |
| `sw_pattern` | 27-day recurrence lag and decay, spike classification and timing | the rotation term in the central expectation |
| `sw_segmentation` | cycle phases, activity bands | which regime a date sits in |
| `sw_predict` | 27-day prediction for F10.7 and Ap, significance, by cycle | where skill stops |
| `sw_forecast` | issued-window verification, rolling windows, daily skill, issue age | how much the outlook beats persistence |
| `sw_design` | **the design window for F10.7 and for Ap** | the two that flow out |
| `sw_density` | profile, spread, sensitivity, by driver, by altitude | what the drivers are worth as density |
| `sw_climate` | yearly F10.7 and Ap, Kp–ap, F10.7A, semiannual | the long-run context |

## 13 · The group

```
l3_solar                          layer 3, owner environment, parent root
  ├─ l3_solar_interface           kind = "required"
  │                               crosses_to = "sys_space_environment"   ← the missing link
  ├─ sw_ap_design                 Ap at a storm return period
  ├─ sw_kp_from_ap                Kp from daily Ap
  ├─ sw_f107_81day                the 81-day centred mean
  └─ sw_f107_design               F10.7 at a lead and a confidence
```

Five rows, one of which is the interface the convention requires. `env_f107`,
`env_f107a` and `env_kp` are not touched — the window-derived sky becomes
available beside the design point, not instead of it.

## 14 · How this executes, in order

Each step stops for review before the next.

**Step 0 — the data, before anything claims to be evidenced.**
Extract the daily series from `prf_study.mat` to CSV, write the manifest with
provenance `noaa_swpc` and the `ap_planetary is SWPC estimated, not GFZ
definitive` caveat, then `xtask bundle publish` and `xtask bundle verify`.
Nothing downstream can be honestly evidenced until this exists.

**Step 1 — the tree's shape.** `layers/l3_solar.toml`: the group, the interface
node with `crosses_to`, and the relation edges. A change to the decomposition, so
it is reviewed by two people, and `xtask codeowners` is regenerated after.

**Step 2 — `sw_ap_design`.** First node, and deliberately not the most
interesting one: an empirical exceedance curve to a return level is the smallest
honest relation in the study, it needs only Ap, and it proves the whole loop —
sheet from the source, fixtures from the record, `parity.csv` from the MATLAB,
hole, gate, ready — on the smallest surface that can carry it.

**Step 3 — `sw_kp_from_ap`.** The published Bartels/IAGA table. Fixtures from
the published scale; `parity.csv` from `prf_ap2kp`'s own self-test anchors, which
are already known: ap = 0, 4, 7, 15, 27, 48, 80, 132, 207, 400 → Kp = 0…9.

**Step 4 — `sw_f107_81day`.** A centred mean, where the whole content is the edge
handling and what happens at the ends of the record.

**Step 5 — `sw_f107_design`.** The largest: persistence decaying into the mean
cycle, plus the percentile growth of the L-day change. Needs `prf_cycles`
understood first, which is why it is last of the four.

**Step 6 — wire upward.** `sys_space_environment_f10_7` and
`sys_space_environment_ap` stop being empty and start reading from the group.

**Step 7 — the panels**, one at a time, each with its reference image recorded
and its three checks passing.

## 15 · What is settled, and what is not

Settled: four nodes, eight panels, one bundle, the interface that was missing,
and an order that proves the loop on the smallest node first.

Not settled, and not blocking step 0 or step 1: whether `env_f107`, `env_f107a`
and `env_kp` eventually read from layer 2 instead of holding their declared
constants. That decision is better taken when there is something real to read.

---

# Part IV · The subsystem, and what the system sees

Part III got the count right and the *architecture* wrong. It treated the study
as four outputs plus pictures. The tree's own shape says otherwise:

> **Layer 3 is the subsystem — where a person works, and where everything is a
> row. Layer 2 is the system — which sees only what crosses.**

So solar weather becomes a subsystem with everything in it as a node, and the
system layer receives the conclusions, not the working.

## 16 · Two findings that change how step 1 is done

**Not one interface node has ever been written.** All seventeen `l3_*_interface`
rows are `state = "empty"`. The layer-3-to-layer-2 seam exists in the seed and
has never been exercised.

**`kind = "required"` has no published instance either** — 213 rows carry it,
none is written. The generator maps it (`"required" => "Kind::Required"`), but
`is_declared()` is true only for `"declared"`, so the gap pass treats a
`required` row like a computed one and will demand inputs, fixtures and holes of
it. Whether that is right for an interface row is an open question, and it is
the first thing step 1 has to answer.

This means writing `l3_solar_interface` is not routine work: **it establishes the
pattern the other sixteen subsystems will copy.** It should be done deliberately
and reviewed as such.

## 17 · The subsystem: every row a person works on

`l3_solar` — "Solar weather", layer 3, owner `environment`. Derived from the
eight tabs and the methods' own returned fields, one question per row.

### The record and its structure — *Repeatability, Pattern*

| node | the question | from |
|---|---|---|
| `sw_cycle_number` | Which solar cycle is this date in? | `C.cycle_no` |
| `sw_cycle_phase` | How far through that cycle? | `C.member` |
| `sw_mean_cycle_level` | What F10.7 does the mean cycle hold at that phase? | `C.superposed` |
| `sw_cycle_repeatability` | How much does one cycle repeat the last? | `C.repeat` |
| `sw_recurrence_lag` | At what lag does the rotation signal peak? | `C.rot_peak_lag` |
| `sw_recurrence_strength` | How strong is that recurrence? | `C.rot_peak_r` |
| `sw_spike_threshold` | What counts as a spike? | `prf_spikes` |
| `sw_event_duration` | How long does a burst last? | `prf_events` |

### Where the record sits today — *Segmentation*

| node | the question | from |
|---|---|---|
| `sw_regime` | Quiet, active or storm, for this day? | `prf_cluster` |
| `sw_activity_band` | Which flux band? | `prf_segment` |

### How far ahead it is knowable — *Predict, Forecast*

| node | the question | from |
|---|---|---|
| `sw_uncertainty_growth` | How far does F10.7 move over L days, at percentile q? | `prf_design` |
| `sw_horizon_climatology` | Beyond how many days is climatology as good? | `F.horizon_clim` |
| `sw_horizon_persistence` | Beyond how many days is persistence as good? | `F.horizon_persist` |
| `sw_forecast_skill` | How much does the issued outlook beat the baseline? | `F.skill_clim` |
| `sw_forecast_bias` | Does it run high or low, and by how much? | `F.bias_fc` |
| `sw_band_coverage` | Does the stated band actually contain the truth? | `F.band_cov` |

### Storms

| node | the question | from |
|---|---|---|
| `sw_storm_rate` | How many storms a year, at each level? | `R.per_year` |
| `sw_storm_return_level` | What Ap recurs once per N years? | `prf_design` |
| `sw_kp_from_ap` | What Kp does this daily Ap mean? | `prf_ap2kp` table |
| `sw_kp_slot_bias` | By how much does the table miss the mean and the peak? | `prf_ap2kp` fit |

### What the design is told — *Design, Climate*

| node | the question | from |
|---|---|---|
| `sw_central_expectation` | Persistence decaying into the mean cycle | `prf_design` |
| `sw_f107_81day` | The 81-day centred mean | `prf_drivers` |
| `sw_semiannual_amplitude` | How large is the semiannual variation? | Climate tab |
| **`sw_f107_design`** | **The F10.7 to design to, at a lead and a confidence** | `prf_design` |
| **`sw_ap_design`** | **The Ap to design to, at a return period** | `prf_design` |
**Twenty-five nodes**, plus the interface and the three target/achieved pairs —
thirty-two rows in the group. That is the subsystem: everything the study
establishes, each as one row a person can open, review and own. The count was
written as twenty-four here while the tables listed twenty-five; it is the tables
that are right.

## 18 · What crosses, and what the system sees

`l3_solar_interface` is the single seam — the convention is one crossing node per
subsystem, and this repository has seventeen of them and no exceptions.

Everything above it is the subsystem's business. What crosses is the conclusion:
**the design drivers, at a stated lead and confidence, with the credibility that
travelled with them.**

At layer 2, `sys_space_environment` then stops being empty:

| layer-2 row | receives |
|---|---|
| `sys_space_environment_solar_flux` | the subsystem's headline — the drivers and what they are worth |
| `sys_space_environment_f10_7` | `sw_f107_design` |
| `sys_space_environment_ap` | `sw_ap_design` |

A person working at system level opens `sys_space_environment_solar_flux` and
sees a number, a confidence and a source. A person working the subsystem opens
`l3_solar` and sees all twenty-four rows. Neither has to read the other's layer,
which is the entire point of having two.

The eight tabs remain **panels** (Part III, §12) — pictures with a declared
`draws`, `reads`, `correct` and a reference, checked by `panel_check.py`. A
figure is not a row, and this repository already has the better home for one.

## 19 · The steps, in order, each stopping for review

| # | step | what it produces | why here |
|---|---|---|---|
| **1** | **The seam** | `layers/l3_solar.toml` — the group; `l3_solar_interface` written | Nothing else can hang anywhere until the subsystem exists. First interface in the repo, so it also settles what `kind = "required"` demands. |
| **2** | **The data** | `solar-drivers/<version>` bundle, daily, from `prf_study.mat` | Nothing below can be evidenced without it |
| **3** | `sw_kp_from_ap` | first node | Smallest honest relation: a published table. Parity anchors already in hand (ap 0,4,7,…,400 → Kp 0…9). Proves sheet → fixtures → parity → hole → gate → ready. |
| **4** | `sw_storm_return_level` | | Needs only Ap and the bundle. One empirical curve. |
| **5** | `sw_f107_81day` | | A centred mean; the content is the edges |
| **6** | `sw_cycle_number`, `sw_cycle_phase`, `sw_mean_cycle_level` | three nodes | The cycle structure everything else leans on |
| **7** | `sw_uncertainty_growth`, `sw_central_expectation` | | The two halves of the design value |
| **8** | **`sw_f107_design`**, **`sw_ap_design`** | the conclusions | Everything they need now exists |
| **9** | **The crossing** | `sys_space_environment_f10_7`, `_ap`, `_solar_flux` written | The system layer starts answering |
| **10** | The remaining subsystem rows | skill, bias, coverage, regime, storms, spikes, semiannual | Evidence and context; none blocks the crossing |
| **11** | The eight panels | one at a time, each with its reference | The visuals, where visuals are checked |

Steps 1 and 2 are the only ones that must happen in that order. From step 3 the
nodes are independent enough to reorder if something proves harder than it looks.

### daily_regime.csv cannot be trusted, and the mechanism is understood

The derived regime column in `bundles/solar-weather` labels every day quiet,
active or storm. **171 of the 827 days it calls "storm" — 20.7% of the positive
class — have a daily Ap of 6 or less**, which is inside the range its own
"quiet" label covers exclusively. 2005-12-07 is labelled storm with confidence
0.992 at Ap 0, kp_max 0 and no flares. The mean confidence of those 171 is
0.774, *higher* than the 0.705 mean of storm labels overall: the quietest days in
the record are being called storms, confidently.

The `.mat` records the classifier and it explains itself. A three-component
Gaussian mixture on one feature, `ap`, transformed `log10(x+1)`. Component means
0.767, 1.024, 1.039 — Ap 4.85, 9.56, 9.95 — so components 2 and 3 sit at
essentially the same place and differ only in width, standard deviations 0.235
and 0.385. **Component 3 is not the high-activity component; it is the broad
one**, and a mixture assigns by posterior, so it wins at both extremes. Ap 0 and
Ap 1 map to 0.000 and 0.301 in `log10(x+1)` space, further from component 1's
mean than most real storms are, and are swept into the broad component — the one
named "storm". The name describes a width, not a level. It means "unusual", and
the quietest possible day is unusual.

This is the source study's defect and not a transcription error: 0 of 10,299 CSV
rows differ from the `.mat`'s own regime index to `regime_name` mapping.

`sw_regime` is therefore **deliberately left seeded**, with all of the above
written into its sheet so a node author meets it before starting. Making it
writeable needs either a classifier ordered by level rather than by fit order,
or an honest threshold on Ap — `prf_segment` already publishes standard ones
(quiet below 8, unsettled 8 to 15, active 16 to 29, G1 30 to 49, G2+ 50 and
above) which separate cleanly and need no clustering at all. The second is
probably the right row, and it is a modelling decision rather than a port.

The bundle's own `INDEX.md` cannot carry the warning without publishing a new
version: the manifest hashes every payload file, so editing the index changes
the content hash and a synced copy of 2026.09.14 stops verifying. Sixteen files
reference that version and the payload is 3.8 MB. Whether to spend a version on
a prose caveat is the data owner's decision.

### Two rows the tree cannot carry yet, and nine nodes waiting on them

Building the group turned up the same wall four times, so it is written here
once rather than discovered a fifth time. **Nine of the sixteen unwritten rows
are blocked on two declared values that do not exist**, and each is a single
number a person has to confirm.

**A mission epoch — a date.** `sys_mission_requirements_mission_epoch` exists at
layer 2 and is seeded, and no derivation edge in this repository crosses a layer,
so a layer-3 row cannot read it even once it is written. Without a date the
subsystem cannot compute a cycle number, a cycle phase, a mean-cycle level, an
81-day centred mean, a semiannual amplitude, or `prf_apdesign`'s dated model —
and `sw_central_expectation` already carries the consequence as a declared
limitation: it uses the record's unconditional mean, so it cannot tell solar
maximum from solar minimum on a record running 64 to 343 sfu.

**A forecast lead — days.** ~~There is no row for a lead and inventing one needs
a confirmed value.~~ RESOLVED. `sw_outlook_lead` now declares it, at layer 3
rather than layer 2 because it is a property of the published product and the
record that verifies it, not something the mission negotiated. `sw_forecast_skill`
and `sw_forecast_bias` are written and read it. `sw_band_coverage` still needs a
confidence and remains blocked on that, not on the lead.

What is NOT blocked, and was written instead: every row whose input is an
existing declared value or another written row. That turned out to be ten of the
twenty-six plus the interface.

### The issued 27-day outlook, measured — and two figures below were wrong

`forecast_issued.csv` was verified against the observed record. The first pass
recorded two findings that a second pass, taken while writing `sw_forecast_skill`
and `sw_forecast_bias`, contradicted. Both corrections are kept visible rather
than overwritten, because the wrong versions are the more tempting answers and
somebody will reach them again.

**The outlook is biased LOW at every lead.** This one stands. Measured over
1997-2025 the mean signed error, forecast minus observed, is negative at all 26
verifiable leads: -0.593 sfu at lead 1, deepening to an interior minimum of
-2.748 at lead 9, recovering to -1.848 at lead 13, then deepening again to
-3.968 at lead 26. For a drag design that is the unsafe direction —
under-predicted flux gives under-predicted density and under-sized drag — and no
symmetric uncertainty band removes an offset.

**It does NOT lose to persistence at short leads.** The first pass reported a
skill of -0.52 at lead 1 and concluded the outlook was worse than doing nothing
through lead 4. That was an artefact of the baseline. Persistence had been taken
as the observed F10.7 on the issue date, and 719 of the 1281 issues index their
rows from lead 0, so the issue date is itself a forecast target for most of the
record: the baseline was being handed an observation the forecaster did not have.
With persistence taken as the last observation strictly BEFORE the issue date,
the outlook beats it from lead 1 onward — +0.069 at lead 1, peaking at +0.438 at
lead 9, still +0.018 at lead 23 — and goes negative only at leads 24, 25 and 26
(-0.036, -0.032, -0.022). The outlook's own RMS error is unchanged by the choice,
10.33 sfu at lead 1; what changed is persistence's, from 6.79 to 10.70. One
choice of baseline, and the sign of the short-lead conclusion flips. The written
rows declare the baseline for exactly this reason.

**`lead_days` is indexed two ways in the same column.** The first pass said every
issue carries exactly 27 rows. It does not: of 1281 issues, 900 carry 27 rows and
349 carry 14 — the older short-format bulletin — with a dozen other shapes making
up the rest. More importantly, among the 899 issues that span exactly 27 days,
719 index their rows as leads 0 to 26 and only 175 as leads 1 to 27. So
`lead_days == 27` does not select the far edge of the window; it selects the
minority that used the 1-based convention, 192 rows against the 868 that lead 26
draws from both. It shows in the answer: lead 27 reports a skill of +0.043, a
different sign from each of its three neighbours, and a bias of -5.958 sfu, half
again as large as lead 26's. Neither figure is a property of the forecast at 27
days.

That is why `sw_outlook_lead` declares 26 and not 27. 27 is the length of the
product and is recorded in that row's assumptions; 26 is the last lead this
record can verify.

The column also runs from -26 to 392. 740 issues contain a lead of zero or
negative — the bulletin published after its own window had begun — and 96 rows
across 10 issues sit above lead 27, one of them reaching 392. Any use of this
table must filter to leads 1 to 26; the measurements above do.

### What step 3 corrected about this table: steps 3 and 4 are the wrong way round

The claim above that the nodes are independent from step 3 is wrong. A computed
row needs at least one declared input, the input must name a row that is
**written**, and no row anywhere in the tree published an Ap. So `sw_kp_from_ap`,
the smallest relation in the plan, was the one row that could not be written
first: binding it to a seeded `sw_ap_design` or `sw_storm_return_level` fails the
contract check, because a seeded row publishes no type.

**The order the tree already encoded was the right one.** `sw_storm_return_level`
sits at order 1353 and `sw_kp_from_ap` at 1354 — the argument before the
function — and that is the dependency: the storm return level publishes the Ap,
and the conversion reads it. The table above had them as 4 then 3. They are now
written in tree order, and the chain runs
`orbit_mission_duration → sw_storm_return_level → sw_kp_from_ap`, three rows deep
from a declared value that already existed.

Two things made that possible without inventing a declared row. First,
`sw_storm_return_level`'s natural input is the **mission length** — the design
storm is the worst one the mission is likely to meet — and `orbit_mission_duration`
is already declared and confirmed. Second, a declared row cannot be invented by an
agent anyway: `[value] confirmed_by` needs a person, and the gate refuses without
one. Between them those two facts point at the same answer, which is a good sign
the decomposition is sound.

The general rule this leaves for the remaining nodes: **before picking the next
step, check that every input it needs is a written row, and prefer the input the
tree's own `order` already implies.** Three of the twenty-five read the record
rather than another row (`sw_f107_81day`, `sw_kp_slot_bias`, `sw_storm_rate`), and
nodes cannot read bundles. `sw_storm_return_level` shows the way round that: the
statistic is fitted on the record **outside** the kernel, and the fit's
coefficients are declared in the sheet where a reviewer can see them, with the
residual against the empirical curve declared as an assumption. What must never
happen is a coefficient appearing only in a hole body.

## 20 · The correction: the driver product, long and short term

§18 above describes a seam that was built differently and a layer 2 that was
never written. This section records what is actually there, what was decided on
2026-09-16, and the order the correction goes in. It supersedes §18's account of
what crosses.

### 20.1 · What is actually there

| | measured state |
|---|---|
| layer 2, all six `sys_space_environment_*` rows | `computed` and **`empty`** — nothing receives anything |
| `l3_solar_interface` | `F107_crossing = F107_design` — **F10.7 only**; Ap never crosses |
| `l3_solar_req_01` / `ach_01` | `F107_req = 250` ← `sw_f107_design` |
| `l3_solar_req_02` / `ach_02` | `F107_req = 250 at 95%` ← `sw_f107_design` — **the same quantity again** |
| `l3_solar_req_03` / `ach_03` | `Ap_req = 150` ← `sw_storm_return_level` |
| long term / short term | **no rows at all** |
| `sys_mission_requirements_mission_duration` | `declared` and **`empty`**, while `orbit_mission_duration` is published at layer 3 |

So the subsystem has 35 study rows and two distinct conclusions, one of which
crosses; the far side of the seam is unwritten; and the split this section
exists to add does not exist.

### 20.2 · What long and short term are, in the study's own numbers

`matlab/reference/mission_drivers.csv` is the tool's saved output for one run —
window opening 2027-06-26, 365 days, 95 per cent, Kp slot `mean`:

| scenario | f107 | f107bar | ap | kp_mean | kp_peak |
|---|---|---|---|---|---|
| nominal | 158.3304 | 158.3304 | 22.0925 | 3.5481 | 4.7953 |
| hotmean | 175.6064 | 175.6064 | 26.6833 | 3.8311 | 5.1968 |
| coldmean | 141.0544 | 141.0544 | 17.5017 | 3.1669 | 4.2780 |
| hotday | 209.7546 | 175.6064 | 41.5352 | 4.5009 | 6.1367 |
| coldday | 109.7211 | 141.0544 | 6.8721 | 1.8533 | 2.8116 |

The three `*mean` scenarios have `f107 == f107bar`: they are the **window mean**,
what the mission sustains — the LONG TERM. The two `*day` scenarios have
`f107 != f107bar`: a single day riding on the mean beneath it — the SHORT TERM.
A design sizes its array and its propellant against different ones of these, and
a tool that publishes only one has decided for the reader which.

`tools/mat_parity.py` already carries this as an open finding in its own words:
*"MATLAB turns (date, duration, confidence) into five driver sets. The port has
no such row."*

### 20.3 · The decision about crossing

A subsystem publishes exactly one crossing row; there are seventeen and no
exceptions, and §18 left "how a subsystem with more than one conclusion crosses"
unsettled for all twenty-one interfaces. It is settled here:

> **`l3_solar_interface` publishes the driver set as one conclusion.** Not four
> crossings, and not one number chosen as governing. The study's product is the
> set; splitting it at the seam would mean layer 2 reassembling something layer
> 3 already had.

What that costs is a change to what a row may publish. The kernel does not need
one: `call(inputs: &[f64], outputs: &mut [f64])` already takes a slice, and the
bus already carries `Vec<ValueOut>`. Only the generator assumes one output —
`crates/vleo-sheet/src/emit.rs` writes `OUTPUT_VARS` with a single name. The
sheet gains repeatable `[[publishes]]` blocks and `emit.rs` writes the list.
Every existing sheet declares one and is unaffected.

### 20.4 · The rows to add

Long and short term as separate published rows, because a design reads one or
the other and a reader must see which:

| row | kind | what it answers |
|---|---|---|
| `sw_mean_band_spread` | computed | the ±1.28σ that makes hotmean and coldmean from the centre |
| `sw_daily_band_spread` | computed | the within-rotation percentile a single day adds on top |
| `sw_f107_design_long` | computed | the sustained F10.7 to design to — the mean band |
| `sw_f107_design_short` | computed | the single-day F10.7 to survive — the daily band |
| `sw_ap_design_long` | computed | the sustained Ap |
| `sw_ap_design_short` | computed | the single-day Ap |
| `sw_driver_set` | computed | the five scenarios assembled, which the interface publishes |

The Kp columns need no new rows: `sw_kp_from_ap`, `sw_kp_slot_bias` and
`sw_kp_mean_bias` already produce both slots.

Then four required/achieved pairs, one per design output, replacing the three
that exist — `req_01`/`ach_01` and `req_02`/`ach_02` answer the same quantity
today and collapse into one.

### 20.5 · Layer 2, and the duration

`sys_space_environment_solar_flux` receives the driver set — it is the headline
and the other two read from it. `_f10_7` and `_ap` receive their design values.
`_atmospheric_density`, `_thermospheric_wind` and `_atomic_oxygen_fluence` stay
seeded: no subsystem computes them, and filling a row from nothing is the defect
the density panel exists to point at.

`sys_mission_requirements_mission_duration` is filled and becomes the one
declared duration. `orbit_mission_duration` reads it rather than declaring its
own; two rows declaring a mission length is two rows that can disagree about how
long the mission is. That is a change in another subsystem and goes through its
owner.

### 20.6 · What may not be a fixture

**The five driver sets are parity data, not fixtures.** `model.migrated_from`
says it plainly: an implementation cannot supply its own expected values. Every
new row above needs expected values derived independently — from the paper, from
the record, or by a person working the arithmetic — and `mission_drivers.csv`
goes in `parity.csv` beside the node, where a disagreement is a finding about
one of the two rather than a check either has passed.

That is the expensive part of this section and it is not optional.

### 20.7 · The order

1. The generator: `[[publishes]]` in the sheet, `OUTPUT_VARS` as a list. No node
   changes; the regeneration diff must come back empty for all 1371.
2. `sys_mission_requirements_mission_duration` filled, `orbit_mission_duration`
   routed to read it.
3. The two spread rows, then the four design rows, each with independently
   derived fixtures.
4. `sw_driver_set`, then `l3_solar_interface` re-specified to publish it.
5. The four required/achieved pairs, replacing the three.
6. Layer 2: three rows written, three left seeded and saying why.

Each step is a pull request that stops for review, as §19 requires. A step that
cannot get its expected values from outside the code does not proceed to the
next.

### 20.8 · What steps 3 and 4 actually did, and where they departed

Written after the fact, against what is in the tree. Four departures from §20.4
and §20.7, each with its reason.

**Six rows were added that §20.4 did not list.** The four design rows publish the
HOT side only — `sw_f107_design_long`'s own third theory step says "one-sided,
because a design is sized against the high side; the cold case is its own
scenario and not this row" — so the five scenarios need a cold side that did not
exist. Two measured rows for the low tail of the within-rotation sample
(`sw_daily_band_drop` 31.2722, `sw_ap_daily_band_drop` 10.5889), because the two
tails are not each other's negation: the F10.7 tails differ by nine per cent and
the Ap tails by forty-two, both because the quantities are bounded below and not
above. Then four relations (`sw_f107_cold_long`, `sw_f107_cold_short`,
`sw_ap_cold_long`, `sw_ap_cold_short`). Composing the cold side inside the
interface instead would have put the 1.28 in a hole body, which is what the
gate's portable-maths check exists to refuse.

**`sw_driver_set` was NOT built, and the interface publishes the set directly.**
§20.3 decided that `l3_solar_interface` publishes the driver set as one
conclusion, and §20.4 then listed a separate row to assemble it for the interface
to republish. That is thirty variables where fifteen do, each of the fifteen
equal to one of the other fifteen, and a second place for the scenario-to-value
mapping to be decided. The assembly is selection rather than arithmetic — no
value is combined with another — and §20.3's rule is that a crossing relays. So
the interface reads the ten producing rows and publishes fifteen members.

**The Kp columns do not cross.** The study's driver set has five columns; this
one has three. `sw_kp_from_ap`, `sw_kp_mean_bias` and `sw_kp_slot_bias` all
exist, and none can be used here, because the bus passes a node's VALUE and not
its RELATION: each answers at one Ap and a driver set needs them at five. Three
ways out, none free — those three rows each publish a set of five keyed to the
scenarios; or the ap-to-Kp scale and both bias tables move into `vleo-core` as
named functions, at the cost of putting measured data in the kernel; or ten more
rows exist, one per scenario per slot. Until one is chosen a consumer needing Kp
must convert it itself, which is the duplication the crossing exists to prevent.
This is the one part of §20.4 that is not done rather than done differently.

**The seam refuses, and that is the state to review.** `sw_f107_cold_short` is
blocked on the nominal case: `sw_central_expectation` reads the cycle analogue
and gives 86.8497 sfu at the declared epoch rather than the study's 158.33, and a
symmetric 1.28-sigma band with a 5th-percentile daily drop stacked on it reaches
38.36, below the floor of 60 that every relation reading F10.7 declares. The
guard is right — 38 sfu has never been observed — and the construction is what is
wrong at a centre that low. All three candidate resolutions are upstream: a
phase-conditioned sigma rather than one pooled across the cycle; empirical
percentiles of the residuals rather than a normal multiplier on a skewed sample;
and not stacking a day's departure on a rotation already at its band's low edge.
The crossing is the AND of ten rows, so it refuses with it. Nothing reads the
crossing yet, so the refusal is contained to that row — but step 6 writes layer 2
against it, and one of the three has to be settled first.

### 20.9 · The order, revised

1. ~~The generator: `[[publishes]]` in the sheet, `OUTPUT_VARS` as a list.~~ Done,
   and finished afterwards: the sheet could declare a set and the contract still
   wrote one slot, so the members were never published.
2. ~~`sys_mission_requirements_mission_duration` filled, `orbit_mission_duration`
   routed to read it.~~ Done.
3. ~~The two spread rows, then the four design rows.~~ Done, plus the six cold-side
   rows 20.4 did not foresee.
4. ~~`sw_driver_set`, then `l3_solar_interface` re-specified.~~ Done as the
   interface alone. The Kp columns are outstanding.
5. ~~The four required/achieved pairs, replacing the three.~~ Done as FIVE pairs,
   for the reason in 20.10 below.
6. ~~Layer 2: three rows written, three left seeded and saying why.~~ Done; see
   20.13.

### 20.10 · What step 5 did, and where it departed

**Five pairs, not four.** §20.4 said "four required/achieved pairs, one per
design output, replacing the three that exist". Replacing all three would have
dropped `req_03`/`ach_03`, and that pair asks a question none of the four design
outputs asks: `sw_storm_return_level` is an extreme-value return level, the one
storm expected in the whole mission, while `sw_ap_design_short` is the top of an
ordinary design band. At the declared window they are 158.38 and 41.70 — a
factor of nearly four. A single ceiling covering both would have to be the storm
one, and the design would then be claiming to operate through a G3 storm. So
`req_03`/`ach_03` is untouched and there are five pairs:

| pair | question | required | achieved |
|---|---|---|---|
| 01 | F10.7, sustained | 250 sfu | 104.07 |
| 02 | F10.7, single day | 350 sfu | 138.30 |
| 04 | Ap, sustained | 48 | 26.70 |
| 05 | Ap, single day | 80 | 41.70 |
| 03 | Ap, the one storm | 150 | **158.38 — does not close** |

**Four requirement rows lost their confirmation, and two of them kept their
number.** `req_01` still says 250 and `req_03` still says 150. But `req_01`'s
question narrowed from "what solar flux must this design survive" to "what
SUSTAINED F10.7 must it operate in", and the question is inside the sheet hash,
so the name against it was against a sheet that no longer exists. `req_02`'s
number moved as well, 250 to 350. `req_04` and `req_05` are new. All four are
gate-red on `declared-value` and each says on its own sheet exactly what a
person would be agreeing to.

**Two requirement numbers now have derivations and two still do not.** `req_04`
and `req_05` are the published ap equivalent amplitudes at Kp 5 and Kp 6, the G1
and G2 thresholds, so a reader can check them against a scale. `req_02`'s 350 is
anchored at the record's largest daily F10.7, 343 sfu on 2023-02-17, rounded up.
`req_01`'s 250 is still what it always was — the next round number above what
the chain achieved when it was written — and its headroom has drifted from 9.6
per cent to 140, because the achieved side went from 228.14 to 104.07 across two
separate changes. A ceiling the environment cannot approach is a closure that
cannot fail, which is the one thing a requirement must be able to do. Re-deriving
it the way `req_04` and `req_05` are derived is the obvious move and it is a
decision rather than an arithmetic consequence.

**`sw_f107_design` now has no consumers.** It was the subsystem's headline and
all three of its readers — both closures and the interface — moved to the design
rows in steps 4 and 5. It is not wrong and it was not replaced: it answers a
different question, a one-sided 95th-percentile persistence drift from today's
value rather than the width of the window. But a published number nothing reads
is a number nobody checks. Give it a consumer, deprecate it, or leave it
published with the paragraph now on its sheet; retiring a row is a decision
about the tree.

**Nothing in the tree evaluates a closure.** `sense` is declared on each
requirement row and the gate checks only that it is present; the pairing is a
convention the matrix draws. So `req_03` failing at 158.38 against 150 is visible
to a reader of the matrix and to nothing else, and has been since it was written.
That is the largest finding of this step and it is not a solar-weather problem —
it is true of every closure in the tree.

### 20.11 · The daily band is a curve, not a number

The refusal 20.8 recorded is settled, and settling it changed four published
design levels. This is the largest correction the port has made and it is a
departure from the source rather than a reproduction of it.

**What was wrong.** `designWindow_` reads one percentile of the within-rotation
departure over the 2001 days before the window and applies it wherever it is
needed. The departure is not one number. Measured over the whole record and
conditioned on the level of the rotation each day sits in, the 95th percentile
runs from 4.63 sfu at a rotation of 70 to 46.93 at 210 — a factor of ten — and on
Ap from 6.02 at Ap 4 to 63.85 at Ap 26, a factor of eleven. A fixed-window
percentile is that statistic MIXED over whatever levels fell in the window, so it
is right near the mean level of its own sample and wrong everywhere else.

The study never met the failure because its centre was high: it holds its last
rotation forecast forward and gets 158.33 sfu, close to the 136.63 its sample
averaged. This port reads the cycle analogue and gets 86.85, so the old drop of
31.27 was being applied at a rotation of 69.63 and gave 38.36 sfu — below the
floor of 60 and below anything ever observed. `sw_f107_cold_short` refused and
the seam refused with it.

**What replaced it.** The four daily rows are `computed` instead of `declared`.
Each reads the sustained level it applies at — the hot day rides on the hot mean
and the cold day on the cold mean, so each row is evaluated at exactly the one
point its consumer needs, which is why a level-conditioned table can live on a
row at all given the bus passes values and not relations. The table in each hole
is measured by one stated rule: a geometric ladder of knots, each bin every day
whose rotation sits within 15 per cent of the knot, a knot kept while its bin
holds at least 200 days and both tails stay monotone, the ladder stopping at the
last knot that passes. Seven knots for F10.7 from 70 to 210, eight for Ap from 4
to 26.

**What moved.**

| | was | is |
|---|---|---|
| `sw_f107_design_short` | 138.30 | 124.14 |
| `sw_f107_cold_short` | refused | 65.24 |
| `sw_ap_design_short` | 41.70 | **90.55** |
| `sw_ap_cold_short` | 6.91 | 4.93 |

The Ap correction is the one to read. The declared window's sustained Ap is
26.70 against a sample mean of 11.64, so the old single number understated the
band by more than four times: the previous Ap single-day design level was less
than half what the record supports for the subsystem's own hot scenario. **A
design sized on 41.70 was under-designed by a factor of two.**

**What it broke.** `l3_solar_req_05` commits the design to one day at the G2
threshold, Ap 80, and the achieved side is now 90.55. That closure fails, and the
requirement has NOT been moved to make it pass. Two of the five closures in this
group now fail and neither is visible to any machine check in this repository.

**What the four rows lost.** `migrated_from` and their parity grids. They no
longer answer the question `prf_density.m:221` answers, so a grid against it
would compare two answers to two different questions. For the record, at the mean
rotation level of the study's own 2001-day sample the F10.7 table gives 32.10
against the study's 34.15 — the level-mixed statistic is about six per cent
wider, which is what mixing distributions of different widths does to a tail.

**What is still open.** Two of the three resolutions 20.8 named are untouched and
neither would have been enough on its own: the daily drop alone took 86.85 below
60, so no change to sigma could have saved it. A sigma measured for the window's
own phase rather than pooled across the cycle, and empirical percentiles of the
residuals in place of a normal multiplier on a skewed sample, both remain worth
doing. And the table conditions on LEVEL but not on cycle PHASE: a rotation at
100 sfu on a rising cycle and one at 100 sfu on a declining cycle get the same
band, and the record does not say they should.

### 20.12 · req_01, re-derived

`l3_solar_req_01` committed the design to 250 sfu because 250 was the next round
number above what the chain achieved when the row was written. That number
tracked the design rather than the sky: two later changes moved the achieved side
to 104.07 and the same 250 became 140 per cent of headroom — a ceiling the
environment cannot approach, which is a closure that cannot fail.

It is now 260 sfu, anchored the way `req_02`, `req_04` and `req_05` are anchored:
in something a reader can check. The largest 27-day mean F10.7 in 28.2 years is
252.67, centred 2024-08-13, and the 27-day mean is the right timescale because
the achieved side is a band on rotation-mean forecasts. Rounded up to the next
ten, the commitment is that the design operates through any rotation the record
has ever shown. `confirmed_by` is empty: the anchor is a fact about the record,
but the commitment is a claim about a spacecraft.

All four of this group's F10.7 and Ap ceilings now have derivations. None of them
has a name against it.

### 20.13 · What step 6 did

Three layer-2 rows written, three left seeded. `sys_space_environment` had six
empty rows and no layer-3 group to answer them — which is the gap
`layers/l3_solar.toml` opens by describing, and which the solar-weather subsystem
was added to fill. These are the far side of the crossing, and the first layer-2
rows in the repository to receive anything.

| row | receives | at the declared window |
|---|---|---|
| `_solar_flux` | `l3_solar_interface` (primary) | 104.07 sustained |
| `_f10_7` | `l3_solar_interface.f107_hotday` | 124.14 single day |
| `_ap` | `l3_solar_interface.ap_hotday` | 90.55 single day |

`vleo run sys_space_environment_ap` walks 22 nodes with none blocked, from the
pinned record through the subsystem, across the seam, to layer 2.

**The convention these set, since sixteen more subsystems will copy it.** A
layer-2 row reads its subsystem's interface node and NOTHING else inside that
subsystem. It computes nothing — it is an identity, and its declared range is the
crossing's own restated, because a row that narrowed the range it received would
be changing the answer while appearing to relay it. Where the crossing publishes
a set, the layer-2 row names the member it is about; five of the fifteen members
are fluxes in the same range with the same unit and the same declared domain, so
naming the wrong one changes the number and nothing in the tree notices.

**Two gate holes this found.** Teaching the node-level contract check about
`<node>.<member>` inputs in step 4 was not enough — two other places resolved a
variable by node id alone. The assembly edge check called every member input
dangling, which is a loud failure. The CYCLE DETECTOR silently skipped them,
which is not: an edge it cannot follow is a loop it cannot find, and a cycle the
resolver cannot see is a run that does not terminate rather than a gate failure.
Both now resolve through a shared helper.

**The three left seeded, and what each is waiting on.** No subsystem computes
them, and filling a row from nothing is the defect this heading's empty rows
exist to point at. Each now carries a comment block naming what writing it would
need, because five layer-2 rows are waiting on these three:

- `_atmospheric_density` is the single row between the solar record and the drag
  budget; `sys_mass_and_aero_drag_acceleration`, `sys_propulsion_decay_rate` and
  `sys_thermal_free_molecular_heat_flux` all read it. A density model needs
  (altitude, F10.7 daily, F10.7 81-day, Kp). The daily flux now arrives; **the
  81-day mean does not** — the crossing publishes `f107bar` for all five
  scenarios and no layer-2 row receives it, which is a row rather than a
  redesign — and **Kp does not cross at all**, which is §20.8's open finding.
- `_thermospheric_wind` needs a horizontal wind model. The drivers it would take
  are now here; the model is a subsystem's worth of work.
- `_atomic_oxygen_fluence` follows from density, so it is waiting on a row that
  is waiting on a subsystem that does not exist.

**§20 is now complete as a plan.** What remains from it is not a step but three
named findings: the Kp columns, the 81-day mean, and the two closures that do not
hold. Two of those are dealt with in 20.14; the closures are decisions.

### 20.14 · Two of the three findings, closed

**The 81-day mean now reaches layer 2.** `sys_space_environment_f10_7_81day`
receives `l3_solar_interface.f107bar_hotday`. It was one row, as 20.13 said it
would be. A density model takes the daily value and the 81-day mean together
because the two carry different physics — the day's EUV heating and the
background state the atmosphere has settled into — and it now has both.

The member's name is the trap and the sheet says so: `f107bar_hotday` is the hot
MEAN, 104.07, not the 81-day mean of the hot day. A `*day` scenario is a single
day riding on the sustained level beneath it, so its 81-day companion is that
sustained level, and only the three `*mean` scenarios have the two equal. The
design pair is (124.14 daily, 104.07 background).

**The ap-to-Kp table was written twice and the two copies disagreed.**
`vleo_core::physics::env::kp_from_ap` held the 28 published pairs with Kp
tabulated as decimals — 0.33, 0.67 — while `sw_kp_from_ap`'s hole held the same
pairs in exact thirds, which is how IAGA defines the index. They differed by up
to 0.0033 Kp everywhere between the anchors.

Nothing caught it for as long as both existed, and WHY is the part worth keeping.
That node's eleven fixtures are `published-source`, drawn from the table's own
anchor points, and the anchors are precisely where two transcriptions of one
table agree. A fixture set drawn only from a source's tabulated points cannot see
a transcription error in what lies between them. The kernel's own comment on
`SOLAR_CYCLE_SHAPE` had already stated the principle — "a hand-copied table
drifts from its original without anything noticing" — and cited `kp_from_ap` as
the example of doing it right, while `kp_from_ap` was the thing that had drifted.

The kernel's copy is now in thirds and is the only one; the node calls it. No
published value moved, because the node was already using the correct
transcription. This also makes the relation callable from any hole, which is the
second of §20.8's three ways out for the Kp columns.

**What is left of §20 is three decisions, none of them mine.** The Kp columns
still do not cross. `l3_solar_req_05` fails at 90.55 against 80 and
`l3_solar_req_03` at 158.38 against 150. And seven rows carry a measured number
or a design commitment with no name against them.

### 20.15 · The Kp columns cross, and three things that assumed one row means one variable

The driver set now carries all five of the study's columns. `sw_kp_scenarios`
reads the five scenario `Ap` values and publishes ten `Kp` — two slots across
five scenarios — and `l3_solar_interface` relays them, taking the crossing from
fifteen members to twenty-five. `sys_space_environment_kp` receives the one a
design reads, the peak slot of the disturbed day.

**Fed the study's own five `Ap`, this chain reproduces all ten of its `Kp`
numbers to the four decimals it printed.** For three relations and two measured
tables composed at five points that is as close as the published precision
allows, and it is what the parity grid holds.

The resolution taken is §20.8's second: the published scale and both measured
slot-bias tables live in `vleo-core` as named functions, so one row can evaluate
all three at five points. The arithmetic is a subsystem row and not the crossing,
because §20.3's rule is that a crossing relays — putting `kp_from_ap(ap) + bias`
in the interface's hole would be subsystem work done where no subsystem reviewer
reads it.

The kernel now holds three pieces of measured data where its own comment used to
say `SOLAR_CYCLE_SHAPE` was the only one. That claim is corrected rather than
quietly falsified, and the rule it rests on is the reason both tables moved:
more than one caller reads each, and a hand-copied table drifts.

**THREE PLACES ASSUMED A ROW PUBLISHES EXACTLY ONE VARIABLE.** All three were
written long before `[[publishes]]` existed, none was wrong until a set row
existed, and only one failed loudly.

- `Scratch.slots` was sized at `NODE_COUNT`. A slot is one VARIABLE. It panicked
  on the first set row — the right failure, found in §20.11's probe.
- `MAX_INPUTS` was 16, hand-written. The crossing now declares 20. `eval` sliced
  to the cap, so the node silently received four fewer inputs, and what surfaced
  was the generated length guard refusing the short slice — reported as a node
  "blocked on an input that has never run", which is not what had happened. Both
  caps are now EMITTED FROM THE TREE by the generator, measured as the actual
  maxima, so neither can be outgrown by a sheet again.
- The daemon's `/v1/index` emitted VARIABLE indices in each row's `in` array, and
  the face reads that array as ROW indices. Identical numbers while the two were
  1:1. A member's variable index sits past the end of the row list, so
  `state.js` indexed off the end of an array, threw inside `ingest()`, and every
  panel then drew from broken state: the matrix rendered nothing at all, the
  design panel differed from its reference in 100 per cent of pixels, density in
  4.6 and climate in 2.9.

**That last one shipped.** It went in with §20.13's layer-2 rows, which were the
first member-variable edges in the tree, and it was not caught because
`tools/panel_check.py` was read as passing when it had in fact crashed before
printing a verdict. The gate was green throughout — nothing in the KERNEL was
wrong — and the whole failure lived in the one place the gate does not look. It
is the argument for `panel_check` being in CI, which it is, and against reading
a tail of its output as a result.

### 20.16 · The gate is green, and four decisions that made it so

`xtask gate` reports **1393 nodes, 0 node check failures, 0 assembly failures**
for the first time on this branch. Everything that closed it was a person's
decision on 2026-09-16, because everything that was open needed one.

**Seven rows confirmed.** Three measured — `sw_mean_band_spread` 13.454382,
`sw_ap_mean_band_spread` 3.593688, `sw_ap_central_expectation` 22.095389 — and
four design commitments. Each sheet records what was agreed to alongside the
name, so a later reader sees the claim and not only the signature.

**Two closures raised, and neither to make the arithmetic pass.**

`l3_solar_req_05` went from the G2 threshold, 80, to G3, 132. The level-
conditioned band took `sw_ap_design_short` from 41.70 to 90.55, and a one-day
commitment at G2 is simply below what this window presents — 90.55 is a G3-G4
day. It now closes with 46 per cent of room.

`l3_solar_req_03` went from 150 to 207, the G4 threshold, and the framing
changed with it. The old sheet REJECTED exactly this number: "a margin chosen so
the closure passes would have to exceed 158.4, which means about 200 — and 200
sits in G4 territory, committing the vehicle on paper to a level it is not built
for." That objection holds for an OPERATING commitment, and it is why
`sw_storm_design_level` is untouched at G3 and the four exceedance rows do not
move. It does not hold for a SURVIVAL one, which is what this row asks: what
must not DESTROY the mission. Its own `reason_upper` already said G4 is
"handled by operating through the event rather than by building for it".

So 158.38 now sits BETWEEN the two levels — above the 132 the vehicle operates
through, below the 207 it must survive — and that is more informative than
either verdict alone: the mission will meet a storm it cannot work through and
will not be destroyed by it. The three exceedance rows still measure the
operating violation: 1.42 days over five years, about 1.24 events, longest run
2 days in 29 years.

All five requirements in this group are now anchored on something a reader can
check — two on the record's own extremes, three on published G thresholds. None
can pass merely because its number was picked above whatever the chain gave,
which was true of exactly one of them before.

**Layer 2 defaults to the sustained scenario.** The crossing carries five; every
row under `sys_space_environment` now takes the sustained one, because that is
what the system designs to by default and the two `*day` scenarios are spikes.
`_ap` moved from 90.55 to 26.70 and `_kp` from 8.00 to 5.20.

A consequence worth seeing rather than hiding: on a sustained scenario the daily
value and its 81-day mean ARE the same number — a `*mean` scenario is its own
81-day mean, which is why the study's driver set has `f107 == f107bar` on
exactly those three rows. So `_solar_flux`, `_f10_7` and `_f10_7_81day` all read
104.07. Three rows, one number, and it is the physics rather than a duplication:
they diverge the moment a spike scenario is read, where the day is 124.14 and
the mean beneath it is still 104.07. A row wanting a spike names the member; none
exists because nothing at layer 2 has asked for one.

**`sw_f107_design` is deprecated.** All three of its readers moved to the band-
width rows across steps 4 and 5. The gate stops treating it as live and the
contract check refuses it as a NEW dependency, so nothing picks it up by
accident. What it answers — a one-sided persistence drift from today's value —
is still a real question, and reviving it is now a deliberate act.

### 20.17 · Three more places that assumed one row means one variable

20.15 named three: `Scratch.slots`, `MAX_INPUTS`, and the daemon's index. That
list was a description of the kernel, and the same assumption was in the FACE.
A set row reached the run correctly and was then described wrongly by everything
that reads the tree.

**The interface table showed one output of twenty-five.** Its own opening line
is *"Every input and output, typed, with its unit. If it is not here it is not
an interface"* — so on `l3_solar_interface` that sentence was false about 24
variables. The table now emits one row per published member, showing the full
dotted variable, its symbol, type and unit, and says how many the set holds. A
member sits indented behind the primary, so the table reads as one answer plus
its set rather than twenty-five equal outputs: the first is the answer the node
is NAMED for, and that distinction is what `kind` and the bus both act on.

**An input on a member printed `?` for its unit.** The unit was looked up by the
whole dotted name in the sheet map, which holds nodes. A member's unit is
declared on the producing node's `[[publishes]]` entry and is read from there
now; the cross-reference opens the producing node, because that is the page
which answers it.

**The dependency edges into both set rows did not exist.** The consumers line,
and the `in` list in the index the shell loads, matched inputs against node ids.
`sys_space_environment_ap` reads `l3_solar_interface.ap_hotmean`, so it was
counted as reading nothing and the interface reported three readers where it has
five. In the index this was the SAME mismatch as the matrix defect in `83ea5910`
— variable names where row indices were meant — and silent for the same reason:
`filter_map` drops what does not resolve. Both now map a variable to the node
that answers it, and the edge list is de-duplicated, because a node reading
twenty members of one interface is one edge and not twenty.

**And `docs/VARIABLES.md` was wrong about 33 variables.** It says it holds every
variable in the tree, its unit, its range, the reason for each bound, and what
reads it; it held one of the interface's twenty-five and one of
`sw_kp_scenarios`' ten. Each member now gets its own entry under the row that
publishes it. That file is generated precisely so a register maintained by hand
cannot drift, which is what makes being wrong in it worse than being wrong in
prose: nobody was going to check it.

The shape of all six is the same, and worth stating once for the next set row: a
NODE and a VARIABLE stopped being the same thing when `[[publishes]]` was added,
and every place that had conflated them was silent about it. Three failed in the
kernel — one loudly, two quietly — and three in the face, where nothing fails at
all; a page simply says something untrue. The generators are the defence, so
what they emit is now derived from the variable list rather than the row list.
Resolving a variable to the node that answers it is the operation that was
missing everywhere, and it is now written four times — `producer_of` in
`gate.rs` and in `page.rs`, the `producer` field on `VARS` in the daemon, and
the reader test in the register generator. That is three too many, and worth
collapsing into `vleo-sheet` the next time one of them needs a change.

## 21 · The figures do not read the engine, and that is why rows have none

A question worth asking, asked by the tool's owner: the legacy study is 38
analysis methods and about 57 views — *every analysis had a picture*. So a
ported analysis row with no figure does not mean there was nothing to draw. It
means the view was not brought across. And the required/achieved pairs, which
are this tree's own idea rather than the study's, are exactly the kind of thing
a design tool draws: a bound, a value, and the distance between them.

Checking that turned up something larger than a missing picture.

### 21.1 · What is actually wrong

**No panel reads the engine. Not one.** `grep -c 'v1/run\|v1/sweep' web/js/solar.js`
returns zero. Every panel fetches a reference bundle — the NOAA record — and
plots it. Where a panel needs a number this tree computes, the number was copied
into the panel as a literal at the time the panel was written.

Three of those literals are now wrong:

| panel | literal | what the row says now |
|---|---|---|
| `design` → `f107Window` | `CENTRAL = 114.8437` | `sw_central_expectation` = **86.85** |
| `design` | requirement options `150 / 132 / 200` | `l3_solar_req_03` = **207** |
| `design` | `A = 92.515531, B = 40.926516` | `sw_storm_return_level`'s fit constants, copied |

The first is two revisions stale: §20 re-specified `sw_central_expectation` on the
cycle analogue and its own sheet records the move — "this row now answers 97.0
sfu for the same window and used to answer 114.8". The design panel still draws
the F10.7 design window around 114.8. The second offers the reader a requirement
of 150 or 200, when the requirement a person settled on 2026-09-16 is 207.

**And every check stayed green throughout.** `panel_check`'s second check is the
good one — a panel must MOVE when each input it claims to read is changed — but a
panel declares the *bundle* as what it reads. Change a row and nothing moves,
because nothing was ever connected. The check is sound; it was verifying a
contract that does not mention the engine.

**The design panel also does not cite the rows that now do its job.** Its own
label is "the design window for F10.7 and for Ap". §20 built exactly that as
`sw_f107_design_long`, `sw_f107_design_short`, `sw_ap_design_long`,
`sw_ap_design_short` and the two cold rows. None of the six is in the panel's
row list. It cites `l3_solar_ach_03`, which since the closure rework publishes a
margin rather than an Ap — so that citation is stale in meaning as well.

### 21.2 · Where the classification went wrong

`docs/SOLAR_ROWS.md` reported seventeen ported rows with no figure and said of
them that there was never a figure to port, the driver set having been a table
in the legacy tool too. That is right about the *table* and wrong about the rest:
the design window is a legacy view, its panel is here, and the rows that compute
it are in that seventeen. The report described a symptom as though it were a
design decision, because it measured which rows a panel *cites* and took citation
for the whole relationship. The relationship it should have measured — does a
figure read this row — does not exist anywhere in the tool.

### 21.3 · The plan, in order

**Step 1 — let a panel read a row.** A `engine` list on the panel definition, and
a helper that fetches `/v1/run` and `/v1/sweep` for it. Panels keep reading the
bundle for the record; the engine supplies the numbers this tree computes.

**Step 2 — make the check bite.** `panels/*.toml` gains `engine = [...]` beside
`reads`, and `panel_check`'s second check extends to it: a panel naming an engine
row must move when that row's value moves. This is what would have caught all
three stale literals, and it is the step that makes every later one safe.

**Step 3 — delete the three literals.** `CENTRAL` from `sw_central_expectation`,
the fit constants from `sw_storm_return_level`, the requirement options from
`l3_solar_req_03`. Re-cite the design panel on the six design-window rows.

**Step 4 — draw the driver set.** `l3_solar_interface` publishes twenty-five
numbers and nothing shows them. Five scenarios against f107, f107bar, ap, kp_mean
and kp_peak, with the legacy run's own values from
`matlab/reference/mission_drivers.csv` drawn alongside. That is the missing
picture and a standing parity check in one figure, and it is the honest answer to
"the driver set was a table" — it was a table because nobody drew it.

**Step 5 — draw the closures.** One figure per required/achieved pair: the bound,
the achieved value, the margin, and how the margin moves across the decision that
drives it. The study had no closures, so it had no such view; the port introduced
them and therefore owes them one. `/v1/levers` already names the decision worth
putting on the x axis.

**Step 6 — re-run the classification.** After 4 and 5, "no figure" should survive
only where a row genuinely has nothing to show, and that list is then a real
finding rather than an artefact of never having connected the two halves.

Steps 1 and 2 are the load-bearing ones. Everything after them is drawing, and
drawing without the check is how three wrong numbers sat in a panel through
green runs.

## 22 · The views, counted — what to cut, what to replace, what to draw

§21 established that no panel reads the engine. This is the other half: what the
panels actually offer, how much of it is the same picture twice, and which of
this tree's own analyses have overtaken the legacy method they were ported from.

### 22.1 · View-wise — 162 combinations, 119 pictures

Eight panels, and the product of each one's controls:

| panel | combinations | distinct pictures | repeats | why |
|---|---|---|---|---|
| `pattern` | 36 | 19 | **17** | `view=spikes` ignores `v`, `w` and `lag` |
| `predict` | 48 | 48 | 0 | every control reaches every branch |
| `repeatability` | 18 | 10 | **8** | `view=storm` ignores `v` and `bins` |
| `design` | 18 | 10 | **8** | `v=f107` ignores `g` and `req` |
| `forecast` | 18 | 13 | **5** | `view=age` ignores `m` and `base` |
| `segmentation` | 8 | 5 | **3** | `view=phase` ignores `v` and `scale` |
| `climate` | 15 | 13 | **2** | `by=kpap` ignores `v` |
| `density` | 1 | 1 | 0 | no controls; it draws the gap on purpose |
| | **162** | **119** | **43** | |

**43 of the 162 are the same picture reached twice.** Not a rendering bug — the
control stays on screen offering a choice that changes nothing, which is the
same defect as the sweep offering `env_f107`: a reader turns the knob, sees no
change, and stops trusting the knob. The fix is per-branch control visibility,
one rule in `panelBody`: a control whose branch does not read it is not drawn.
That is a dozen lines and it removes all 43 at once.

`predict` at 48 combinations is the other end and worth a second look: four
controls multiply faster than a reader can hold, and 48 pictures of one analysis
is not obviously more useful than twelve.

### 22.2 · Where this tree's analysis has overtaken the legacy method

Two of them, and both are already the row's answer while the panel still draws
the legacy way. These are replacements, not additions:

**The centre of the design window.** `prf_design` freezes the last 27-day
rotation forecast and holds it flat, giving 158.33 sfu for its own 2027 window.
`sw_central_expectation` runs the amplitude-scaled cycle analogue over the window
and gives 86.85. Over the same span past maximum, the two completed cycles ran at
91 sfu scaled onto cycle 25's amplitude — so the record puts the legacy method
**74% high**, and this row within a few per cent of it. The row's own sheet
records the disagreement and `tools/mat_parity.py` keeps measuring it. The design
panel still hard-codes the legacy centre.

**The within-rotation spread.** `designWindow_` applies one spread at every
level. §20 conditioned it on the rotation level it applies at, which moved
`sw_ap_design_short` from 41.70 to 90.55 — the previous value was less than half
what the record supports for the subsystem's own hot scenario. The design panel
draws neither.

So the design panel is not merely stale (§21.1); it draws a method this tree has
measured as wrong. Step 3 of §21.3 is therefore a replacement rather than a
refresh, and the panel should say on its face which method it is drawing.

### 22.3 · Node-wise — what each row without a view needs

Eighteen rows carry no figure. They are three groups, not one, and each wants a
different kind of picture.

**Group A — the driver set, 12 rows.** `sw_f107_design_long`, `_short`,
`sw_f107_cold_long`, `_short`, the four Ap equivalents, `sw_kp_scenarios`,
`sw_mean_band_spread`, `sw_ap_mean_band_spread`, `sw_ap_central_expectation`.
One figure serves all twelve: **five scenarios against f107, f107bar, ap, kp_mean
and kp_peak**, with the legacy run's own values from
`matlab/reference/mission_drivers.csv` drawn beside each. That is the picture the
legacy tool never had — it printed the table — and it is a standing parity check
in the same frame. One new panel, `drivers`.

**Group B — the closures, 10 rows.** Five required and five achieved. One figure
per pair: the bound as a line, the achieved value as a point, the margin as the
distance, and the whole thing swept over the decision `/v1/levers` names as
moving it most. The legacy tool had no closures, so there is no view to port;
this is the one genuinely new picture the port owes. One new panel, `closure`,
with the pair as a control — five views, not five panels.

**Group C — the seam and the odd one, 2 rows.** `l3_solar_interface` is answered
by Group A's figure, since the driver set is what it publishes.
`sw_window_peak_level` belongs on the `design` panel as the window maximum
against the window mean — the gap `sw_central_expectation`'s own sheet names.

Eighteen rows, **two new panels and one addition to an existing one.**

### 22.4 · The order

1. §21 steps 1–2 — panels can read the engine, and `panel_check` enforces it.
   Nothing below is safe before this.
2. Per-branch control visibility. Removes the 43 repeats; no new drawing.
3. §21 step 3 — replace the design panel's three literals, and say on the panel
   which method it draws (22.2).
4. The `drivers` panel — Group A, with the legacy values beside this tree's.
5. The `closure` panel — Group B.
6. `sw_window_peak_level` onto `design` — Group C.
7. Re-run `tools/solar_rows.py`. "No figure" should then be empty, and if it is
   not, what remains is a real finding.

Steps 2 and 3 are subtraction and correction; 4 to 6 are the only new pictures,
and there are three of them rather than eighteen.

## 23 · The figure set, specified — 162 combinations down to 68 live views

§22 counted what is there. This is what to build, panel by panel, with what each
one contains, what it reads from the engine, and why it earns its place.

### 23.1 · The two rules everything below follows

**A control that picks between things a reader wants to COMPARE is not a
control. It is an encoding.** `predict` offers four percentiles as four separate
views; a reader wanting to know how the 50th relates to the 99th has to hold two
pictures in their head and cannot. Drawn as a fan they are one picture and the
relationship is the thing you see first. The same is true of `pattern`'s three
detrend windows and `forecast`'s two persistence baselines. This is where most
of the reduction comes from, and it ADDS information rather than removing it.

**A control that picks between things a DESIGNER decides between stays a
control.** `design`'s G level and requirement are choices a person makes and
must be able to turn. Those stay, and after §21 they read the rows rather than
carrying copies.

A third rule falls out of §22.1 and needs no argument: a control the current
branch does not read is not drawn. That removes the 43 repeats on its own.

### 23.2 · The eight existing panels

**`climate` — 15 combinations, 13 after, 13 live.**
Contains the record by year, by day of year, by month, the 13-month smoother,
and Kp against ap. Nothing collapses: these are five genuinely different
aggregations of one record, and the variable picks a different quantity rather
than a comparable series. `v` is hidden on the `kpap` branch, which is the only
repeat.
*Reads from the engine:* `sw_semiannual_amplitude` as the amplitude drawn on the
day-of-year view, `sw_f107a_ratio` as the scatter band on the smoother,
`sw_kp_from_ap` as the fitted line on `kpap`, `sw_mean_cycle_level` as the mean
cycle.
*Why it matters:* it is the context every other number sits inside. A design
value with no sense of the long run is a number nobody can sanity-check.

**`repeatability` — 18 → 10 live.**
Every cycle stacked on phase with the mean of the complete ones over them, and
the storm scale. `view=storm` hides `v` and `bins`; that is all eight repeats.
*Reads:* `sw_cycle_repeatability` as the stated r against the spread actually
drawn, `sw_mean_cycle_level` as the mean line — which the panel currently
recomputes, and recomputing a row's answer in a figure is how the two quietly
stop agreeing.
*Why:* this is the evidence for the cycle analogue, and the cycle analogue is
what replaced the legacy centre (§22.2). The picture has to carry its own case.

**`pattern` — 36 → 3 live, and better for it.**
The autocorrelation with the rotation peak marked, and the spike classification.
`w` becomes three lines in one picture instead of three views: comparing detrend
windows IS the question, and no reader can do it across three tabs. `lag` becomes
an axis zoom rather than a view, because it changes what you see and not what is
drawn. `view=spikes` hides the rest.
*Reads:* `sw_recurrence_lag` marking the peak, `sw_recurrence_strength` as its
height, `sw_spike_threshold` as the cut, `sw_event_duration` as the mean width.
*Why:* the rotation term in `sw_central_expectation` rests on this, and the row
that used to restate the lag was retired for duplicating it — the picture is now
the only place the lag is argued.

**`segmentation` — 8 → 5 live.**
The histogram with the band edges, and the regimes by phase. `view=phase` hides
`v` and `scale`.
*Reads:* `sw_regime`'s boundaries and `sw_activity_band`'s edges, drawn on the
histogram rather than written into it.
*Why:* both those rows are classifiers whose whole content is where the cuts
fall. A classifier with no picture of its cuts cannot be reviewed.

**`predict` — 48 → 4 live, and far better.**
Growth against lead, all four percentiles as a fan, split all-cycles or
by-cycle. `q` stops being a view and becomes the fan; `span` becomes an axis
zoom. Four controls multiplying to 48 is more than a reader can hold, and the
one relationship a designer needs — how much worse the 99th is than the 95th —
was the one the old form could not show.
*Reads:* `sw_uncertainty_growth` as the 95th line, `sw_horizon_climatology` and
`sw_horizon_persistence` as marked crossings, `sw_band_coverage` as whether the
stated band contains the truth.
*Why:* this is where the design window's spread comes from. It is the panel that
says how far ahead anything is knowable at all.

**`forecast` — 18 → 3 live.**
Verification by lead, by year, and issue age. `base` becomes two lines rather
than two views — strict against leaky persistence is a comparison, not a choice.
`m` becomes three small multiples in one frame for the same reason.
*Reads:* `sw_forecast_skill`, `sw_forecast_bias`, and `sw_recurrence_lag` as the
verification lead, which is the row those two now read after `sw_outlook_lead`
was retired for restating it.
*Why:* it is the evidence that persistence is worth nothing at mission leads,
which is why `sw_central_expectation` carries that term and calls it inert.

**`design` — 18 combinations, 18 live after the repair, and the most changed.**
Three things happen here. The three literals go (§21.1). The method it draws is
replaced with the one the rows use (§22.2) and the panel says which it is
drawing. And the `f107` branch gains its own G-equivalent and requirement so it
stops ignoring two controls — which removes the eight repeats by making them
mean something rather than by hiding them.
*Reads:* `sw_central_expectation`, `sw_storm_return_level`, `sw_ap_design`,
the four design-window rows, the two cold rows, `l3_solar_req_01` through `_05`,
and `sw_window_peak_level` as the window maximum against the window mean.
*Why:* it draws the two numbers that leave the subsystem. It is also the panel
that has been wrong the longest, and the only one where being wrong changes a
design decision rather than an impression.

**`density` — 1 view, unchanged.**
It draws nothing and says why: `sys_space_environment_atmospheric_density` is
seeded and no row answers it. Keep exactly as it is. A gap stated where it bites
is worth more than a gap nobody has written down.

### 23.3 · The two new panels

**`drivers` — new, 6 views.**
*Contains:* the five scenarios on one axis against f107, f107bar, ap, kp_mean and
kp_peak — one view per quantity and one showing all five together. Beside every
value, the legacy run's own from `matlab/reference/mission_drivers.csv`.
*Reads:* `l3_solar_interface`, all twenty-five published members.
*Why it matters, and it matters most:* twelve rows have no figure and this one
answers all of them. It is the subsystem's entire output in a single frame — the
thing that crosses to the system — and it is the picture the legacy tool never
had, because it printed the table instead. Drawing this tree's values against
the legacy run's in the same frame makes it a standing parity check that a
person sees rather than a script reports. Where the two disagree, §22.2 says the
disagreement is deliberate and which method is better; a reader should meet that
claim as a picture, not as a paragraph.

**`closure` — new, 5 views.**
*Contains:* one view per required/achieved pair. The requirement as a horizontal
line, the achieved value as a point on it, the margin as the labelled distance
between them, and the whole thing swept over the decision `/v1/levers` reports
as moving the margin most — so the reader sees not just whether it closes but
how much room there is and what would spend it.
*Reads:* `l3_solar_req_01`–`_05` and `l3_solar_ach_01`–`_05`.
*Why:* ten rows have no figure and this answers all ten. More than that, it is
the only picture in the tool of the question the tool exists to answer — is the
design met. The legacy study had no closures, so there is no view to port; this
is the port's own contribution and it should look like one. Until §20 nothing in
the tree even compared the two sides.

### 23.4 · What it comes to

| | views now | views after |
|---|---|---|
| `climate` | 15 → 13 distinct | 13 |
| `repeatability` | 18 → 10 | 10 |
| `pattern` | 36 → 19 | 3 |
| `segmentation` | 8 → 5 | 5 |
| `predict` | 48 → 48 | 4 |
| `forecast` | 18 → 13 | 3 |
| `design` | 18 → 10 | 18 |
| `density` | 1 | 1 |
| `drivers` | — | 6 |
| `closure` | — | 5 |
| **total** | **162 combinations, 119 pictures** | **68 views** |

Sixty-eight live views against a hundred and nineteen, and **every one of the
sixty-eight moves when its controls move**. Eighteen rows gain a figure. Three
wrong numbers leave the design panel, and the method it draws becomes the method
the rows use. Nothing that was distinct is lost: the reductions are percentiles
and windows becoming layers of one picture instead of separate ones, which is
more of the record on screen at once rather than less.

### 23.5 · Order of work

1. §21 step 1 — a panel can read a row.
2. §21 step 2 — `panel_check` fails a panel that names an engine row and does
   not move when it moves. Everything after this is protected; nothing before
   it is.
3. Branch-aware controls — the 43 repeats, one rule.
4. `design` — literals out, method replaced, `f107` branch given its own
   requirement.
5. `pattern`, `predict`, `forecast` — controls collapsed into encodings.
6. `drivers`.
7. `closure`.
8. Reference images and a person's name against each new panel, as
   `panels/README.md` requires.
9. Re-run `tools/solar_rows.py`; "no figure" should be empty.

### 23.6 · The reductions, verified against the code — and what the check changed

48 to 4 is a large claim and it was made from the outside. Read from the inside,
three of the four collapses are lossless and one needed correcting. The evidence
is in `web/js/solar.js` and it is short enough to state.

**`predict`: `span` is an axis bound and nothing else.**
`build` computes `for (let L = 30; L <= maxL; L = Math.round(L * 1.35))` and the
value at each `L` is a quantile of changes over that lead, computed from the
record alone. `maxL` does not appear in it. So the curve drawn to fifteen years
CONTAINS the curve drawn to one, at the same lead points, and an axis zoom shows
exactly what `span = 365` showed. Lossless.

**`predict`: `q` as a fan is strictly more than `q` as a view.**
Four separate quantiles of one sorted array. Drawn together they are four bands
of one picture, and the relationship between them — the thing four tabs cannot
show — becomes the first thing visible.

**`pattern`: `lag` is an axis bound, and cutting it was hiding a result.**
The autocorrelation loop is `for (let L = 1; L <= maxLag; L++)` and each value is
independent of `maxLag`. But the panel also marks three harmonic peaks, and the
third is found by `peakIn(72, 95)` — so at `lag = 60` that mark silently does not
exist. Always drawing to 200 with a zoom does not merely lose nothing; it
restores a mark two of the three settings were suppressing.

**`forecast`: `m` cannot be a fan, and the first draft of this plan was wrong to
imply it could.** The three metrics carry three different units — skill is `[-]`,
bias and RMS error are `[sfu]`. Series in different units on one axis is the
error the whole repository is built to prevent. They become three stacked panels
sharing an x-axis, which is the small-multiple device, not three lines.

#### What the check changed in the plan

**Nothing is filtered; things are highlighted.** The first draft collapsed a
control away. A control that picked one of four percentiles becomes a control
that EMPHASISES one of four bands always drawn — the complete picture every time,
with a reading aid for the band you came for. The same for `pattern`'s detrend
window and `forecast`'s baseline. Nothing is lost in focus either, and the count
of distinct PICTURES is still four rather than forty-eight.

**`pattern` weights the canonical window.** `w = 365` is the window
`sw_recurrence_lag` and `sw_recurrence_strength` were measured under — the panel
says so itself, and quoting a different number for a row it illustrates would be
worse than having no panel. So 365 is drawn heavy and carries the peak marks;
181 and 731 are drawn light as the sensitivity they are. Three lines, one claim.

**`predict` must draw its sample count.** `ns` shrinks as the lead grows, and the
old one-year view hid that. A fan drawn to fifteen years looks equally
trustworthy at both ends and is not, so the thin end is drawn thin — faded, or
with n on the axis. Exposing this is a gain, but only if it is exposed.

**`predict`'s caption keys on the visible range, not on a control.** The code
already notes that whether the eleven-year cycle is visible depends on how far
the lead goes, and earns that sentence per picture. With the span a zoom, the
sentence has to follow the zoom.

#### The acceptance test, per collapsed panel

Not "does it render" — the panels have always rendered. For each of `pattern`,
`predict` and `forecast`, every question its old view set could answer must be
answerable from the new one, and the check is written as a list of those
questions in the panel's `correct` block, which `panels/README.md` already
requires a person to sign. A collapse that cannot state its old questions has
not been verified; it has been asserted, which is what the first draft of §23.2
did.

#### The corrected count

`predict` 4 pictures, `pattern` 3, `forecast` 3 — unchanged as PICTURES, because
the collapses were sound. What changed is that each keeps its control as an
emphasis rather than a filter, `pattern` weights one window over two, `predict`
shows where its sample runs out, and `forecast` stacks rather than overlays. The
total stands at 68 live views.

## 24 · The complete figure set, and where all 162 went

Two questions, answered in full: what will exist, and what happened to everything
that will not.

### 24.1 · A correction to §23 first

§23.2 said `design` would go from 18 combinations to 18 live views by giving its
`f107` branch its own G level and requirement, so that the eight repeats became
real instead of hidden. Half of that is wrong. `g` is the **NOAA G storm scale**
— G1 minor, G3 strong — and it is geomagnetic. F10.7 has no G scale and
inventing one would be putting a number on the page that nothing in the record
supports. On the `f107` branch `g` genuinely does not apply and hiding it is the
correct answer, not a workaround.

The `requirement` control does apply to both: F10.7 has `l3_solar_req_01`
(sustained, 260 sfu) and `l3_solar_req_02` (single day, 350 sfu). So `design`
becomes Ap at 3 G levels × 3 requirements = 9, plus F10.7 at its 2 requirements
= 2. **Eleven views, not eighteen.** The total falls from 68 to 61.

### 24.2 · The ten figures

| # | panel | views | what it contains |
|---|---|---|---|
| 1 | `climate` | 13 | the record by year, by day of year, by month, the 13-month smoother, Kp against ap |
| 2 | `repeatability` | 10 | every cycle stacked on phase with the mean of the complete ones; the storm scale |
| 3 | `pattern` | 3 | the autocorrelation with three detrend windows as three lines and the rotation peak marked; the spike classification |
| 4 | `segmentation` | 5 | the histogram with the band edges drawn on it; the regimes by cycle phase |
| 5 | `predict` | 4 | growth against lead as a four-percentile fan, all-cycles or by-cycle, with the sample thinning shown |
| 6 | `forecast` | 3 | verification by lead, by year, and issue age — each as three stacked metrics against two baselines |
| 7 | `design` | 11 | the design window: Ap at 3 G levels × 3 requirements, F10.7 at its 2 |
| 8 | `density` | 1 | nothing, and why — the gap stated where it bites |
| 9 | **`drivers`** | 6 | five scenarios against the five driver quantities, with the legacy run's own values beside each |
| 10 | **`closure`** | 5 | one per required/achieved pair: the bound, the achieved value, the margin, swept over what spends it |
| | **total** | **61** | |

### 24.3 · Where all 162 went, and whether anything mattered

Every one of the 162 combinations falls into exactly one of five fates. Only the
first is a removal.

**(a) An exact repeat — 35 combinations, removed.** A branch ignored a control
that stayed on screen, so the same picture was reachable several ways.
`pattern`'s spike view ignored variable, window and lag: seventeen of them.
`repeatability`'s storm view ignored variable and bins: eight. `forecast`'s issue
age: five. `segmentation`'s phase view: three. `climate`'s Kp-against-ap: two.
**Nothing is lost.** The picture each reached still exists; only the extra routes
to it go, and a control that changes nothing is worse than no control.

**(b) Became a layer in a richer picture — 26 combinations.** `predict`'s four
percentiles become four bands of one fan. `pattern`'s three detrend windows
become three lines. `forecast`'s two baselines become two lines and its three
metrics three stacked panels. **Nothing is lost and something is gained**: the
relationship between the four percentiles was the one thing four separate views
could not show.

**(c) Became an axis zoom — 32 combinations.** `predict`'s three lead spans and
`pattern`'s three maximum lags. Verified against the code in §23.6: both are loop
bounds, and the longer curve contains the shorter one at the same points.
**Nothing is lost**, and `pattern` gains a harmonic mark that two of its three
settings were suppressing.

**(d) Correctly hidden — 8 combinations.** `design`'s `g` control on the F10.7
branch. A storm scale has no F10.7 meaning, so these eight were never distinct
pictures and cannot be made into any. §24.1.

**(e) Survives as its own view — 61 combinations,** which is the table above
minus the eleven that are new.

35 + 26 + 32 + 8 + 61 = 162. Plus eleven new views in the two new panels.

### 24.4 · What I would keep an eye on, rather than assert is fine

Three things where the collapse is defensible and I would still want a person to
look at the result before signing the reference image.

**A single-percentile `predict`.** A fan is right on screen and wrong in a
report, where one clean curve with its number is what goes in the document. The
highlight control gives emphasis but still draws the other three. If exporting
one band matters, that is a fourth option on the highlight control — "this one
only" — and it costs almost nothing to add. I have not put it in the plan because
nobody has asked for an export yet; it is the first thing to add if they do.

**Three lines in `pattern` may be crowded.** Weighting 365 heavy and the other
two light is the mitigation, and it may not be enough if the three curves sit on
top of each other for the first twenty lags, which they probably do. If the
reference image is unreadable the fallback is small multiples, as `forecast` is
already using for a different reason — three short panels rather than three
lines.

**`design` at eleven views is the one I am least sure is enough.** It is the
panel that carries the two numbers leaving the subsystem, and its requirement
control currently offers three hard-coded Ap values that do not include the real
one. After §21 it reads the rows — but it will then offer exactly the
requirements that exist, which is right for checking the design and may be too
rigid for exploring one. A free-entry requirement, or a slider across the
declared range, is the obvious extension and I would rather add it after a person
has used the fixed version than guess now.

Nothing in §24.3 is a judgement call — those are repeats, layers, zooms and one
scale that does not apply. These three are judgement calls, and they are the only
three.

## 25 · The removals audited, and the figures made worth looking at

### 25.1 · The audit: all 35 removals are provably the same picture

§24.3 claimed 35 combinations were exact repeats. That was argued from which
controls a branch reads; here it is settled from what the branch returns.

Each repeat branch ends in a call that takes the record and nothing else —
`spikes(rec)`, `stormScale(rec)`, `regimeByPhase(rec)`, `issueAge(idx)`,
`kpAgainstAp(rec)`, `f107Window(rec)`. None takes the options object. And
`p.data()` is invoked with no arguments at all, so `extra` cannot vary with a
control either. Same function, same inputs, same output — and the output object
carries the labels, marks and note as well as the data, so nothing visible
differs, not just nothing plotted.

**All 35 are genuine repeats. Nothing is added back.** The audit was worth doing:
had any of those functions taken `o`, the repeat count would have been wrong and
a real view would have been deleted.

### 25.2 · What is wrong with the figures, measured rather than felt

**The axis ticks are placed in pixel space.** `chart.js` divides the plot height
into `ny` equal parts and labels whichever data value lands there:
`const at = y1 - i * (y1 - y0) / ny`. `nice()` only formats what it is handed. So
`repeatability` is labelled 58.8, 90.3, 121.9, 153.4, 184.9, 216.5 and `design`
55.8, 87.0, 118.2, 149.3, 180.5, 211.7. A reader cannot use an axis whose ticks
are not round numbers — every value has to be interpolated by eye between two
arbitrary ones. **This one defect disfigures all ten panels**, and it is a
twenty-line fix: choose the step from {1, 2, 2.5, 5} × 10^k, extend the domain to
the nearest step, then label.

**The series palette fails two of the five colour checks**, measured with the
validator rather than judged:

```
#b5731a,#2a6f97,#7a9e3f,#8a3ffc,#c1440e,#4a4a4a   → FAILED
  [FAIL] Lightness band   #4a4a4a at 0.409, outside the band
  [FAIL] Chroma floor     #2a6f97 at 0.093 and #4a4a4a at 0 — read as gray
```

**`design` draws three series and has no legend.** Requirement, design bound and
the record's curve are distinguished by three inline labels and three colours. A
legend is required from two series up; direct labels are the supplement, not the
substitute.

**`repeatability` puts its legend inside the plot, over the data.** Four entries
sitting on the top-left corner where cycles 23 and 25 run.

**Nothing has a hover layer and nothing has a table view.** The panels are canvas
and read-only: no crosshair, no tooltip, no keyboard path to a value, and no
WCAG-clean twin. Every number in every figure is available only by eye against an
axis that, per the first defect, cannot be read precisely anyway.

**Two smaller ones.** `design`'s y axis is labelled `[-]` beside "daily Ap",
which reads as a missing unit rather than a dimensionless index. And cycle 25 in
`repeatability` ends in a flat horizontal segment where its bins run out, which
reads as data rather than as the end of the record.

### 25.3 · The palette, re-stepped and validated

```
#b5731a  #2f6fa8  #2e7d55  #8f43e0  #c2185b  #00918f
```

All five checks pass in light mode and in dark, against each mode's own surface.
Slot 1 is unchanged, so the orange that means "the record" everywhere in this
tool keeps meaning it. The CVD separation is 8.2 — above the floor of 8 but not
far above it, so direct labelling is not optional here; it is what makes the
palette legal.

### 25.4 · The work, in order of how much each fixes

1. **Nice ticks in `chart.js`.** One function, every panel, the largest visible
   improvement available. Do it first and re-shoot every reference image.
2. **The palette swap**, and the dark-mode steps from the same list.
3. **A legend component**, present whenever a panel draws two or more series,
   placed outside the plot rectangle. This removes `design`'s absent legend and
   `repeatability`'s overlapping one together.
4. **A crosshair and tooltip on the line panels**, with keyboard focus showing
   what hover shows. Canvas needs explicit hit-testing; a nearest-x lookup on the
   drawn series is enough for every panel here.
5. **A table view per panel** — the same data as rows, which is also what makes
   the values quotable in a document without screenshotting a chart.
6. **The two small ones**: unit labels that say what they are, and a drawn series
   that stops where its data stops.

Only after 1 to 3 is it worth drawing `drivers` and `closure` (§23.5 steps 6–7):
a new panel built on the current chart layer inherits every defect above, and
would then have to be redrawn and re-signed.

### 25.5 · Two things the figures should show and do not

**Where the data runs thin.** `sw_storm_return_level`'s own sheet says its top
end rests on ranks 2 and 3 of a 28.2-year sample, and `predict`'s sample count
falls away with lead. Both draw a line of constant weight from end to end. The
fix is the same in both: fade or thin the mark where n is small, and say n on the
axis. A curve that looks equally confident everywhere is the one way a figure
lies without containing a wrong number.

**Which method is being drawn.** §22.2 establishes that the design panel draws
the legacy method while the rows use a better one, and §21 that three of its
constants are stale. Once that is repaired the panel should name the method on
its face — the reader cannot otherwise tell a ported picture from a corrected
one, and this tool's whole claim is that it knows the difference.

## 26 · The work order

This supersedes the step lists in §21.3, §23.5 and §25.4, which were each written
about one part. Correctness first, then the chart layer, then the panels, then
the two new ones. Nothing in Part B is worth doing before Part A is finished, and
nothing in Part C or D is worth doing before Part B, because a panel built on a
broken chart layer has to be redrawn and re-signed.

### Three more findings from looking at the rest of the references

**`pattern` has no significance band, and that is not cosmetic.** The
autocorrelation shows five or six bumps and a reader has no way to tell a real
harmonic from noise. An ACF is read against ±1.96/√n and this one is drawn
against nothing. The legacy tab's own description includes "significance", so the
band is not an invention. Until it is there the panel cannot answer the question
it exists to answer.

**`pattern` marks two peaks where three are computed.** `peakIn(72, 95)` runs at
the default lag of 120 and there is a visible bump near 80 in the reference
image, unmarked. Either the third mark is computed and dropped, or the peak
search fails; either way the picture is not showing what the code found.

**No zero line on the autocorrelation.** Zero is the reference for the whole
quantity and the axis happens to label −0.197 and 0.0426 instead. The nice-ticks
fix puts a tick on zero; it should also draw that one rule slightly stronger.

### Part A — make it right. No drawing.

**A1. Panels can read the engine.** An `engine` list on the panel definition and a
helper fetching `/v1/run` and `/v1/sweep`. The bundle keeps supplying the record.
*Blocks everything.*

**A2. `panel_check` enforces it.** Done, and it needed two mechanisms rather
than one.

*The failed-state probe.* The face marks its mount `data-failed` when a render
throws and clears it on a good draw, and every check refuses a mount carrying it.
Without this, check 2 could not tell a redraw from a collapse: a failed render
blanks the canvas, a blank canvas has a different signature from a drawn one, and
"it moved" was satisfied by the panel BREAKING. A deliberately broken panel
passed.

*Check 2b.* For every row a panel declares in `engine`, the engine's reply is
rewritten in the browser and the picture must change. Declaring a row is not
reading it — a panel can fetch a value and go on drawing a literal, which is
where four of `design`'s numbers were, and no other check can see it: such a
panel renders, moves when its controls move, and matches yesterday's reference,
because it draws a perfectly steady picture of a number nobody computed.

2b renders the same path TWICE, differing only in what the engine answered. A
first version called a global redraw hook on the open panel, and after check 2's
reloads that hook could point at a DETACHED host — the redraw drew into nothing,
the signature changed, and 2b passed a panel that ignores the value. Rebuilding
the page each time cannot go stale and leaves no test hook in the product, which
is worth the second it costs.

Both are covered by `--selftest`, which now breaks `design` in the two ways only
these checks can see.

**A3. Delete the stale literals in `design`.** Done, and there were **five**,
not the three §21.1 found by reading. Each one was found by wiring up the one
before it.

| literal | what it was a copy of | what it is now |
|---|---|---|
| `CENTRAL = 114.8437` | `sw_central_expectation`, two revisions stale | read from the row (86.85) |
| `A = 92.515531, B = 40.926516` | `sw_storm_return_level`'s fit constants | the row, swept over mission duration |
| `AP_AT_G = {1: 48, 2: 80, 3: 132}` | `sw_ap_design`'s G-to-Ap conversion | the row, swept over the G level |
| `req` options `150 / 132 / 200` | nothing — no row has ever held 150 or 200 | the rows themselves, by id |
| `at: 250, 'required ≤ 250'` | nothing — `req_01` is 260, `_02` is 350 | the selected row |

Both sweeps reproduce the constants they replace to every digit printed, so the
picture did not move for that reason; the requirement line moved from a
hard-coded 150 to `l3_solar_req_03`'s 207, which is the number a person settled
on. Both marks now name the row they came from, so a reader can see the source
without opening the file.

The requirement is two controls rather than one, because an Ap bound has no
business on an F10.7 axis: `req` offers `l3_solar_req_03/04/05` and `reqf`
offers `_01/_02`. A5 hides whichever the current branch does not read.

*And a regression in A2, found by the selftest rather than by me.* Check 2's
failed-state probe had stopped biting. `wait_for_function` returns the instant
the canvas differs, and it differs as soon as the redraw CLEARS it — before the
render has finished and had the chance to record a failure. The probe was
reading the flag in that window, finding nothing, and check 2 had quietly gone
back to being the check it replaced. It settles before reading now. The lesson
is the selftest's: a check that has stopped working looks exactly like a check
with nothing to find.

*Two more things this turned up.* 2b was blind to a row a panel draws as a SHAPE — it
rewrote `/v1/run` only, and the return curve arrives over `/v1/sweep`. The
interception now bends both. And one selftest case patched a source string this
step reworded, which would have made the case a silent no-op: the selftest now
refuses a break that changes nothing, because a case that proves nothing is
worse than no case.

**A4. Replace the method `design` draws.** Done. The F10.7 branch computed
`central expectation + 95th percentile growth` from the record, which is
`sw_f107_design`'s relation — the row §20 DEPRECATED — and `prf_design`'s method,
the one §22.2 measures as 74% high. A figure drawing a retired method beside
live rows tells a reader something the tree has stopped believing.

It draws the four rows that replaced it: `sw_f107_design_long` and `_short` on
the hot side, `sw_f107_cold_long` and `_short` on the cold, each swept over
mission length. The band between the outer two IS the design window, which is
what the panel's own label has always claimed to draw, and each is built from a
within-rotation spread conditioned on the level it applies at. The note says
which method was replaced and why. All six rows are cited now; the panel cited
none of them before.

*2b caught me inside a minute.* The rewrite kept a guard on
`sw_central_expectation` and went on declaring it, while the four swept rows
carry the centre internally and nothing drew it — "asked the engine for
sw_central_expectation and drew the same picture when the answer changed". The
declaration was a promise the panel could no longer keep, and the check said so
before I noticed. Guard and declaration both removed.

*And a gap in check 3 this exposed.* No reference image changed, because the
reference is photographed at the panel's OPENING state and that is the Ap
branch. `design` has eleven views and one picture stands for all of them, so the
branch this step rewrote is not covered by a reference at all. Part C5 should be
a reference per view that matters, not one per panel. Until then the F10.7 half
rests on 2b and on a person looking at it.

*One unreproduced failure, recorded rather than dismissed.* A run reported
`climate 3 matches, 33.8% of pixels differ`; two further runs and a
re-record showed no change, and the same run carried a thrown evaluate from the
then-broken `design` panel. The likely mechanism is fallout in the shared
browser session rather than anything in `climate`. "Flake" is not a root cause,
so: it has not recurred, the mechanism is plausible but unproven, and it is
written down here so a second occurrence is a pattern rather than a surprise.

**A5. Branch-aware controls.** Done. Each control that a branch ignores carries
a `when(o)`, declared rather than inferred, because a branch that stops reading a
control should have to say so. Eight predicates across six panels.

| panel | combinations before | reachable after |
|---|---|---|
| `pattern` | 36 | 19 |
| `predict` | 48 | 48 |
| `repeatability` | 18 | 10 |
| `forecast` | 18 | 13 |
| `climate` | 15 | 13 |
| `segmentation` | 8 | 5 |
| `design` | 18 | **11** |
| `density` | 1 | 1 |
| | **162** | **120** |

35 repeats gone and 8 correctly hidden, which is 43 — and `design` gains **one
genuinely new view**, so 162 − 43 + 1 = 120. The new one is real: its F10.7
branch had a single picture that ignored both the G scale and the requirement,
and now has two, one per F10.7 requirement. The G scale is gone from that branch
entirely, which §24.1 established was never a repeat to merge but a control that
could not have meant anything.

Verified in the DOM rather than inferred from the count: on `design` the Ap
branch shows `v, g, req` and the F10.7 branch shows `v, reqf`; on `pattern` the
autocorrelation shows `view, v, w, lag` and the spike view shows `view` alone.

The remaining reductions — `pattern` 19 → 3, `predict` 48 → 4, `forecast`
13 → 3 — are Part C, where a control becomes an encoding rather than
disappearing.

**A6. Re-run `tools/solar_rows.py`.** Done. A4's re-citation moves four rows out
of "ported, no figure": 20 → 24 with a figure, 17 → 13 without. `l3_solar_req_01`
leaves the free set, which drops from 9 to 8, because the `design` panel now
draws it as the F10.7 requirement. `l3_solar_ach_01` stays free — nothing reads
it and no figure cites it, and its requirement being kept does not rescue it.

*Part A changes no pixels deliberately. Every reference image will still move,
because three numbers in `design` were wrong — which is the point.*

### Part B — the chart layer, once, for all ten panels

**B1. Nice ticks.** Done, and it is the largest single change in the visual
layer so far.

The step comes from the 1 / 2 / 2.5 / 5 ladder, and it is taken at the NEAREST
rung in log space rather than the next one up. Rounding up overshoots: a range of
160 asking for five ticks gives a raw step of 32, the next rung above 3.2 is 5,
and that is a step of 50 and three ticks for a range that wanted six. The nearest
rung is 2.5, a step of 25, and six ticks. Log space because the rungs are
multiplicative.

| panel | axis before | after |
|---|---|---|
| `repeatability` | 58.8, 90.3, 121.9, 153.4, 184.9, 216.5 | 75, 100, 125, 150, 175, 200 |
| `design` (Ap) | 55.8, 87.0, 118.2, 149.3, 180.5, 211.7 | 100, 150, 200 → 75…200 at step 25 |
| `pattern` | −0.197, 0.0426, 0.282, 0.522, 0.761, 1.00 | 0, 0.25, 0.5, 0.75, 1 |
| `predict` | 36.5, 53.3, 70.1, 86.9, 103.7, 120.5 | 40, 60, 80, 100, 120 |

**The zero rule is drawn one step stronger where zero is in range**, and on
`pattern` that is not cosmetic: an autocorrelation is read against zero, the old
axis labelled −0.197 and 0.0426 instead of it, and where the curve crosses was
not visible at all. It is now.

The formatter stopped adding decimals the ticks do not have — 150 rather than
150.0, 0.2 rather than 0.200 — since a round tick printed to two places is only
half the fix.

All eight canvas panels moved by 4 to 6 per cent of their pixels, which is the
ticks and nothing else: checks 1, 2 and 2b passed throughout, so no panel broke
on the way. Every reference is re-recorded and **all eight need a person's eye
before `panels/README.md` is satisfied** — a re-record is a claim that the new
picture is right, and only a person can make it.

**B2. The validated palette.** Done. `#b5731a #2f6fa8 #2e7d55 #8f43e0 #c2185b
#00918f`, all five checks passing in light and in dark against each mode's own
surface. Slot 1 is unchanged, so the orange that means "the record" keeps meaning
it.

The retired hues were not confined to `chart.js`: sixteen occurrences of
`#2a6f97`, `#8a3ffc`, `#c1440e`, `#7a9e3f` and `#4a4a4a` were written into
`solar.js`, `run.js` and `relation.js` as mark colours. All are re-pointed to the
same slot in the validated set, so a threshold drawn in one file and a series
drawn in another still agree.

Worth keeping in view: the worst adjacent CVD separation is 8.2 against a floor
of 8. That is a pass and not a comfortable one, which is why B4's legend and B5's
direct labels are what make the palette legal rather than what makes it pretty.

*Dark mode is still outstanding.* `INK` is one set of light-mode values and the
face has no dark steps at all; the skill's rule is that dark is selected from the
same ramps against the dark surface, never an automatic flip. That is a B2
remainder, not a thing B2 delivered.

**B3. Mark specs.** Done. Lines 2px with round join and cap, so a curve does not
go spiky at a corner and a one-point run still draws. Dots at radius 4 — eight
pixels is the smallest mark a person can point at — each with a 2px ring in the
SURFACE colour, which is what lets two dots overlap and stay two dots. Bars get a
2px surface gap rather than a 1px shave: white does the separating, and a border
drawn round a mark is ink that is not data.

The ring is skipped above 120 points. A scatter of thousands is a field rather
than a set of markers; there the alpha is doing the work and a ring per dot would
cost more than it buys.

Gridlines were already solid hairlines after B1. Dashes stay reserved for a
threshold, which is what `design`'s requirement rule legitimately is.

**B4. A legend, in its own band above the plot.** Done. It was drawn inside the
frame at the top left — on `repeatability` exactly where cycles 23 and 25 run, so
four entries sat on top of the thing they identify. It now has a band of its own,
the plot starts below it, and the entries pack into rows across the width rather
than one per line, because vertical space spent on a legend is stolen from the
picture.

A single series still gets none: there is one colour and the axis title already
names it, so a box with one swatch would restate the title and cost space.

Only `repeatability` moved — 8.7% of its pixels. Every other canvas panel draws
one named series at its reference state, which is also why `design`'s "missing
legend" was never visible in a reference: its four-series branch is the F10.7 one,
and no reference photographs it. That is the §A4 gap again, and it is now the
main thing standing between Part B and a figure set anyone can trust: **eight of
the eleven panels have exactly one view under test.**

**B5. Direct labels, selectively.** Done. Each named series carries its final
value at the end of its line, and nothing else is labelled — the axis, the legend
and the hover carry the rest, and a number beside every point is chaos that goes
unread. Only where a panel draws two or more named series; one series needs no
disambiguation.

Not optional: the palette's worst adjacent CVD separation is 8.2 against a floor
of 8, and a separation in that band is legal only with a second encoding. The
legend is one. A label riding the line is the one that works when a reader is
looking at the data rather than at the key.

*They needed a gutter, which the first attempt did not give them.* Written at the
line's end they sat ON the curve and ran off the right edge, clipped mid-digit.
The texts are known before the scale is — each is the last finite value of a
named series, which is data and not geometry — so they are measured first and the
right margin becomes whatever holds the widest. The same move the legend band
makes vertically. They now form a column just outside the plot that a reader can
scan, instead of four numbers scattered wherever their lines happened to finish.

Three panels moved: `climate`, `repeatability`, `segmentation`. The single-line
panels changed by less than the 2% tolerance, which is 1.6px to 2px on one thin
curve and is the honest amount.

**B6. Aspect.** Done, in the part that can be done without the data. The canvas
took the full host width at a fixed 0.40 of it, which on a wide screen is nearly
three to one: a gentle rise across three units of width and one of height is a
slope of about 18°, and a reader compares slopes by their ANGLE. Flattened like
that, the difference between two curves stops being visible before it stops being
real.

Now 2.2 : 1, with the width capped at 1180 so a very wide window adds height
rather than stretching the picture further. Banking the principal slope to 45° is
the classical answer and needs each panel's own data; capping the aspect is most
of the benefit and needs none of it.

**B7. Crosshair and tooltip on the line panels**, with keyboard focus showing what
hover shows. Done, and the interesting part was not the crosshair.

The crosshair and the tooltip were already there and were **mouse-only**. So every
number in these figures was reachable by pointing at it and by no other means: an
axis cannot be read to better than a tick, and the panels carry between fourteen
and two hundred and twenty-three points each. That is the whole dataset behind a
pointing device.

The fix was not to write a second readout for the keyboard. It was to notice that
writing them as two is *how* the keyboard came to have none — so there is now one
`drawReadout(canvas, xv, focused)` and both paths call it. The canvas takes
`tabindex="0"`, arrow keys step through the drawn points, Shift steps a tenth of
the series, Home and End jump to the ends, Escape clears, and the readout is
identical because it is the same function. The canvas carries `role="img"` and an
`aria-label` that becomes the value under the cursor as the reader steps.

*Hit target.* The step above asked for one no smaller than 24px, and the honest
answer is that this chart has no per-point target to size: the crosshair follows
the pointer continuously in x and reports the nearest drawn mark, so the target is
the plot area. The keyboard path is what removes the precision requirement, which
is the accessibility outcome the 24px was standing in for.

**AND IT FOUND TWO DEFECTS IN THE READOUT ITSELF, both by measurement rather than
by reading the code.** A probe walked every point of every panel with the arrow
keys and demanded that the readout name exactly the series the table fills at that
x. It disagreed in two places, and both were the readout inventing a number:

- **Beyond a series' own end.** Repeatability's cycle 25 stops at phase 0.532
  because it is still running. The readout took the nearest point unconditionally,
  so at phase 0.975 it answered *cycle 25: 153.4* — a value from less than half
  the phase the crosshair stood on.
- **Across a gap.** At phase 0.775 cycle 24 is missing, and the readout reported
  the neighbour on the far side of the hole. This module's first rule is that a
  line is never drawn across a null, on the grounds that it would be a claim
  nobody made. A readout across one is the same claim in text.

Both are now the same test — *is there a mark drawn at xv* — which is exactly what
the table under the panel shows, so the picture and the table agree by
construction. Eight panels, 691 rows, zero disagreements.

*And a third, smaller.* A single series carries no name, so its readout was a bare
number: **"F10.7 64. 5"**. Sighted readers have the y axis; a reader hearing the
`aria-label` did not. The axis title now stands in for the missing name, in the
tooltip as well as in the announcement, because one path is the point. Separately,
a canvas with `role="img"` and no label is announced as "image" and nothing else —
so until the first arrow key every panel was an unnamed picture. It is now labelled
at draw time from what was drawn: *"F10.7 by cycle phase, 4 series: mean of the
complete cycles, cycle 23, …"*.

**B8. A table view per panel.** Done. A `<details>` under every figure, closed by
default because the picture is the point, present always because a tooltip that is
the only way to reach a number gates the data behind a pointing device. Between 14
and 223 rows per panel, built in under half a millisecond.

**Built from the spec the chart was drawn from**, not from the panel's own arrays.
A table assembled separately is a second description of the data, and the first
time it disagrees with the chart the disagreement is invisible — the same argument
that put the record's CSV parser next to the engine's rather than shipping a shaped
summary. It is also what made the B7 probe possible at all: the table could only be
used as the reference for what the readout should say because it is not an
independent transcription.

*The node page's relation view got one too*, on the same grounds — it is the figure
a reader arrives at from a row, and its 120 swept points were reachable only by
hovering. It tables the relation once rather than the three series the canvas
draws, because the faint whole, the walked prefix and the head dot are one dataset
shown three ways and tabling all three would print the same column three times.
**The declared-value view deliberately has none**: its picture is a bar of 101
identical zeros with one mark on it, and the three numbers it actually shows — the
value and both bounds — are already in the note below in words. That is the
equivalent a reader needs; a table there would be the form of one without the
content.

*One thing measured after the fact.* The table inherited `width: 100%`, which
spread two columns across the whole panel: the header sat a foot from the value
under it. Columns now hug their contents. Found by rendering it and looking at it,
which is step seven and not step zero.

**B9. Thin the mark where the sample thins.** Done for the two panels that have
the count, and honestly refused for the third.

`predict` and `forecast` were both already counting `n` per point and throwing it
away. A series may now carry `n` alongside `x` and `y`, and where it does each
segment is drawn at an opacity following its own count against the best in the
series, floored at a fifth so a thin stretch fades rather than vanishes. On
`predict` at a fifteen-year span the far end is now visibly lighter than the
near, which is the honest picture: a growth curve at long lead rests on a
fraction of the pairs the short lead has.

**Not done for `sw_storm_return_level`, and the reason is not laziness.** Its
thinness is not a per-point count — the curve is a fit whose top end rests on
ranks 2 and 3 of a 28.2-year sample, and the fade above has no count to follow.
Marking it means deciding *where* the fit stops being supported, which is a
judgement about the row rather than a property the chart can read. It wants a
decision, and it is listed here so it does not pass for done.

*And a message the check was getting wrong.* Capping the aspect changed every
canvas's shape, and all eight panels reported "100.0% of pixels differ" — true,
useless, and indistinguishable from eight panels breaking at once. `_differ` now
returns the two shapes when they disagree, and the finding says so: a reference
of a different shape cannot be compared at all, so it is not a percentage.

### Part C — the panels, on the repaired layer

**The move all three make.** A control shows one picture at a time, and every one
of these three knobs was set over a quantity a reader wants to *compare*: three
detrend windows, four percentiles, three metrics. So the knob made the comparison
impossible and then charged a view count for it. Each becomes an encoding — a
second line, a ramp step, a second frame — and 80 combinations become 10 views
that each say more than the 80 did.

**C0. Stacked panes in the chart layer.** Done, and C3 could not exist without it.
`spec.panes` is a list of frames sharing one x extent, each with its own y scale,
y label, marks and legend; the x axis is drawn once at the bottom. A spec that
names `y`/`series`/`marks` directly is normalised to one pane, so there is one
path rather than two — two paths is how a chart layer comes to have a feature
that works on one and silently does nothing on the other.

*The check that it changed nothing.* `panel_check` was run before re-recording
anything, and **all eight unchanged panels still matched their existing reference
images**. A refactor of the drawing routine that moves no pixel on the single-pane
path is the only kind worth trusting, and the reference images are what proved it
rather than a reading of the diff.

The readout, the arrow-key ladder and the table all walk panes now, each value
formatted by its own frame — three frames in different units is the reason the
stack exists, and one formatter for all of them would print sfu to the precision
a dimensionless ratio wants.

**C1. `pattern`** — done. 19 combinations → **3 views**.

The detrend window and the maximum lag were controls and are now the picture.
Three windows are three lines, 365 heavy because that is the window
`sw_recurrence_lag` and `sw_recurrence_strength` were measured under, and the axis
simply runs to 200 — the longest the lag knob offered, so nothing is lost by
removing it. The comparison the knob prevented is the finding: **731 sits above
365 at 182 of the 200 lags**, because a longer window calls less of the record
trend and leaves more low-frequency signal to correlate. The published number is a
choice, and now a reader can see how much it moves.

*The third peak was computed, quoted in the prose and never drawn.* It is the one
that settles the period — furthest from the decay the first bump rides on — so the
sentence claiming 27.0 days was asking a reader to take the most important of the
three on trust. It is marked. The three read 26, 54 and 81: 26 for the first, and
27.0 from each harmonic.

*And a significance band, which is Bartlett's and not 2/√n.* The naive band tests
whether the series is white noise. This one decays from 0.94 at lag 1, so that
test is passed by everything and says nothing — ±0.0198, under every wiggle drawn.
Bartlett's large-lag standard error asks the question a reader actually has: is
this bump more than the decay below it already produces. It widens with lag, which
is the honest shape. The 365 curve is outside it at 140 of the 200 lags.

The band is drawn as `aside: true` — furniture, like a mark that happens to vary
with x. Named in the legend so the dashes mean something; kept out of the end
labels, the readout and the table, which are for the record.

**C2. `predict`** — done. 48 → **4 views**.

Four percentiles at once, and they are an ORDERED set, so they are an ordinal ramp
in one hue and never four categorical ones — a rainbow across 50, 90, 95 and 99
would say the four are unrelated things. `#86b6ef → #3987e5 → #1c5cab → #0d366b`,
run through the validator as an ordinal ramp: monotone lightness, every adjacent
gap clear, light end at 2.06:1 against the surface, hue spread 4°. The 95th is
heavy because it is the one `sw_uncertainty_growth` publishes.

What the knob was hiding is the cost of the choice: **at a lead of one year the
four run from 2 to 102 sfu**. That is the entire argument for a band, and a panel
showing one percentile at a time could not make it.

The span knob was three truncations of one curve, which is not a comparison at all
— it is the same picture with less of it. The axis runs the whole ladder, to 13.54
years. The measured shape survives: the 95th rises to 116 sfu at 4.08 years, falls
to 64 at 10.03, and rises again. The four never cross, because they are quantiles
of one sorted sample at each lead — sorted once for all four, so there is no way
for them to disagree about what the sample was.

**C3. `forecast`** — done. 13 → **3 views**, and this is the panel the stacked
frame was built for.

Three metrics of one outlook against one lead, and they cannot share a y axis:
skill is a dimensionless ratio, bias is signed sfu, RMS error is positive sfu on a
different scale. **A second y axis would let whoever drew it choose where the
curves cross**, which is the most reliable way to make a chart say something the
data did not — so the answer had been a control, and a control made "is the
outlook biased where its skill collapses" unanswerable. Three frames in a column
on one lead axis answers it by looking.

The baseline is two lines in the skill frame, which is better than the control
was: strict and leaky are drawn together, so the size of the leak is visible
rather than remembered. At lead 1 they read **0.069 strict against −1.314 leaky**.

*And it found a real error in what the panel had been computing.* Bias and RMS
error use no baseline — `sw_forecast_bias` is the mean of (forecast − observed) at
a lead, full stop — but the panel dropped every row whose persistence lookup came
back empty before computing them, so it quoted the row's quantity over a subset
the row does not take. The note carried the claim that the baseline "changes
nothing on this metric", which was very nearly true and not exactly, which is the
worst kind. Each metric is now taken over the pairs its own definition covers.

*One thing measured rather than assumed.* The first draft of that note illustrated
the correction with lead 27 — the one lead where the two counts happen to be
equal, so the sentence explained a fix using the case the fix does not touch. The
note now finds the lead where they differ most and names it.

**C4. `repeatability`, `segmentation`, `climate`, `design`, `density`** — no
structural change beyond Part A and Part B, as planned. 10, 5, 13, 11 and 1 views,
counted off their control lists: a control hidden by a `when` contributes nothing
in the states that hide it.

**The subsystem now stands at 50 views across eight panels** — 10 + 3 + 5 + 4 + 3
+ 11 + 13 + 1 — against 61 in §26's totals, the difference being the two Part D
panels that do not exist yet (6 + 5).

**C5. Re-shoot every reference image and have a person sign each `correct`
block.** Half done, and the half that is not is not mine to do.

The three changed references are re-recorded, and every `correct` block is
rewritten to state what the new picture must show — including the defects to look
for: three curves that do not cross, a band that WIDENS rather than sitting at
constant width, a dashed leaky baseline BELOW the solid one at short lead, three
frames registered on one x extent, and lines that break at a dropped year rather
than stepping over it.

**The three signatures are withdrawn.** They said "content unchanged, canvas
resized", and the content is now emphatically changed. Carrying a signature
forward across a chart that has been rebuilt would make check three prove only
that a machine has not changed its mind, which is exactly the failure
`panels/README.md` warns about. All three read `UNCONFIRMED` with what changed and
why, and they need a person to look at the PNG and put their name there. **An
agent cannot be that person**, and this is listed here so it does not pass for
done.

### Part D — the two new panels

**D1. `drivers`**, 6 views. Done, and it is the only figure in the subsystem
whose subject is the port itself.

Five scenarios, five quantities, and the legacy run's own answers as dots beside
each. §23 was right that the missing figure was not an omission — the study
published the driver set as a TABLE, so there was never a picture to port — but
"the legacy tool had no figure" is a reason not to have ported one, not a reason
not to have one. **What makes it worth drawing is that the two tools disagree**,
and a table of fifty numbers makes that invisible where a picture makes it the
first thing anybody sees.

*The scenarios are ordered cold to hot* — quietest day, cold sustained, nominal,
hot sustained, worst day — and in that order every quantity is monotone, so a
reader checks the picture by whether it rises. In the order the CSV happens to
list them it is a zigzag that says nothing.

*The parity view is a ratio on a log axis*, because agreement is 1 and the two
kinds of disagreement here are about a factor of two in each direction; on a
linear axis "twice" and "half" are 1.0 and 0.5 from the line and look like
different sizes of error. Nine of the twenty-five cells agree to within a tenth
of a per cent, and they are a block rather than a scatter: Ap, Kp mean and Kp
peak at each of the three SUSTAINED scenarios. *I wrote "three" into the
`correct` block before measuring it, while the panel's own note — computed at
build time — said nine. That disagreement between a generated number and a
hand-written one is exactly what having both is for.* Both families of
disagreement are deliberate and explained: F10.7 because the two tools centre the
window differently (158.33 held flat against 86.85 from the cycle analogue), and
every single-day value because the within-rotation departure is level-conditioned
here and a fixed-window percentile there.

**D2. `closure`**, 5 views. Done, and it draws the question the tool exists to
answer.

Two frames: the achieved quantity swept against the requirement as a flat dashed
line, and the same fact below as a signed margin with zero drawn. **The
horizontal axis is not chosen here.** `/v1/levers` measures every declared
decision upstream at both ends of its own range and reports which moves the
margin most; the panel takes that one, skipping the requirement — not because it
is small (it is usually the largest) but because a margin is a fraction OF the
bound, so moving the bound moves it by construction and says nothing about the
sky. The five axes come out as the mean band spread, the mean band spread,
mission duration, expected Ap, expected Ap.

*Verified rather than asserted, across all five pairs*: the requirement line is
flat; the crossing in the top frame and the margin's zero in the bottom agree to
the float; and the margin falls as the achieved value rises, which is the `<=`
sense every one of these five declares. Pairs 01 to 03 never reach zero inside
their lever's range and 04 and 05 cross at 43.40 and 63.55.

**A figure now opens on the row it was reached from.** `closure` lives on all ten
of its rows and draws one pair at a time, so a reader arriving at
`l3_solar_ach_03` — having clicked it precisely because they want the survival
closure — met a picture of the F10.7 one. The page knows which row it is and the
panel knows which of its views belongs to which row; neither knew alone. This was
found by check 2b rather than by reading: the panel declared it read
`l3_solar_ach_04` and did not move when that row's answer changed, because it was
drawing pair 01.

**AND 2b FOUND A RACE IN `engineValues`, WHICH IS THE BETTER FIND.** A run returns
every value on its path, not only the row asked for, and a row asked for directly
is often also on another asked-for row's path — `closure` asks for both sides of a
pair, and the requirement rides inside the achieved row's run. The results were
written into one object as each arrived, so **a row reached both ways took
whichever request resolved last**. 2b failed intermittently, which is the worst
way to find anything. A row that was asked for now takes its own run's answer,
always; everything else fills in only where nothing asked for it. Three
consecutive clean runs after the fix.

**D3. Reference images and signatures for both.** Images recorded; **both marked
`UNCONFIRMED`**. An agent may not be the person who has looked: the whole value of
check three rests on a human having agreed the picture is right, and a machine
recording its own output and signing for it proves nothing at all. Both `correct`
blocks are written to be checkable by that person, and both name the defect to
look for — an axis labelled "Required — …" on `closure` means the lever question
is not being asked, and twenty-five cells all sitting on 1 in `drivers` means the
panel is dividing a number by itself.

**D4. Re-run `tools/solar_rows.py`.** Done, and it comes out exactly as this step
predicted it should:

```
  ported 37 (37 with a figure, 0 without)
  added  18 (18 with a figure, 0 without)
  free to remove: 0
```

**Every one of the 55 live solar rows now has a figure, and none is free to
remove.** The thirteen "ported, no figure" rows are the driver set and `drivers`
draws them; seven of the eight "added, no figure" rows are the closure pairs and
`closure` draws them.

*The eighth was the real finding this step was told to expect.*
`sw_window_peak_level` — the maximum of the cycle analogue inside the mission
window — was read by nothing and cited by nothing, and the standing rule is that a
row of no use should not be kept. It is kept, and the reason is only visible once
it is drawn: put beside the four F10.7 design rows on their own axis, it **rises
monotonically in steps** while every one of them wanders, because a longer window
can only contain more of the cycle while a window MEAN depends on where the window
ends. The two cross at about 5.9 years, and past there the analogue's own peak is
above the hot single-day design value — a design sized on a mean sitting under the
thing it averages. That is worth a reviewer's attention, and a row that produces
it is not a row of no use.

It is a fifth, dashed line on `design`'s F10.7 branch. That branch is not covered
by the reference image, because the reference is shot at the state the panel opens
in — so the claim is written into `design`'s `correct` block instead, where the
person who signs it will read it. Which is the eight-of-eleven gap below, showing
up in practice.

*And one sentence in the generated `SOLAR_ROWS.md` is now wrong and corrected:*
"nothing plots a row's answer anywhere yet" was true until these two panels.
`drivers` plots the twenty-five variables the crossing publishes and `closure`
plots the margins.

### The gate, at every step

`cargo run -p xtask -- gate && cargo test`, and — because the local gate does not
cover the visual layer — `python3 tools/panel_check.py` and its `--selftest`. A
step is not finished until all four are green and the reference image is signed.

### Totals

Ten panels, 61 views, every one moving when its controls move. 35 repeats gone,
18 rows gaining a figure, three wrong numbers out of `design`, and a chart layer
that places its own ticks.

**Reached.** Ten solar panels and 61 views: 10 repeatability + 3 pattern +
5 segmentation + 4 predict + 3 forecast + 11 design + 13 climate + 1 density +
6 drivers + 5 closure. Every one of the 55 live rows carries a figure and none is
free to remove.

**What is still open, and none of it is bookkeeping.**

- **Five reference images are UNCONFIRMED** — `pattern`, `predict`, `forecast`
  from Part C and `drivers`, `closure` from Part D. Each needs a person to look
  at the PNG and put their name in `confirmed_by`. An agent cannot be that
  person, and a signature carried forward across a changed chart would make check
  three prove only that a machine has not changed its mind.
- **Dark mode has no steps at all.** `INK` is one light-mode set, and the skill's
  position is that a dark palette is selected and validated against its own
  surface rather than flipped.
- **Eight of eleven panels have exactly one view under test.** The reference is
  shot at the state the panel opens in, so `design`'s whole F10.7 branch — five
  curves, including the one Part D added — is covered by prose in `correct` and
  by nothing automatic. `panel_check` already takes a per-row `state`; the same
  mechanism would let a spec record several.
- **`sw_storm_return_level`'s support is still unmarked.** Its thinness is a
  fit's support — ranks 2 and 3 of a 28.2-year sample — not a per-point count, so
  B9's fade has nothing to follow. Marking it means deciding where the fit stops
  being supported, which is a judgement about the row.


---

## 27 · The curve audit — "MATLAB was all curves; some of ours are numbers"

The question this answers: the legacy study's 38 analysis methods produced ~57
**figures**. This tree's 55 live rows each answer **one number**. So where did the
curves go, and is anything lost?

### 27.1 · A classifier bug, found first

The ported/added split was **37 / 18** and is **38 / 17**. `tools/solar_rows.py`
read the plan's mapping tables with a pattern that required the source cell to be
exactly a backticked name, so two cells — `` `prf_ap2kp` table `` and
`` `prf_ap2kp` fit `` — failed to parse at all and their rows fell through to
"added by this tree". `sw_kp_from_ap` survived because it also holds a
`parity.csv`; **`sw_kp_slot_bias` did not, and has been counted as an addition for
as long as that script has existed.** The cell is now taken whole and searched.

*A pattern that is silent when it fails is the wrong shape for a classifier.*

### 27.2 · Two different questions were being run together

"Curve or number" in `SOLAR_ROWS.md` does **not** mean "has a figure". It is
measured through `/v1/levers` and means *does any declared decision upstream move
this row's answer*. A row can be a number in that sense and still be read off a
curve the face draws in full.

Measured the way the node page draws it — sweep over every declared row reachable
upstream that carries a range, take the widest:

| | its own page draws a curve | its own page draws a number | total |
|---|---|---|---|
| **ported** from the legacy MATLAB | 27 | 11 | 38 |
| **added** by this tree | 10 | 7 | 17 |
| | **37** | **18** | **55** |

**17 of the 18 "number" rows are `declared`** — a value a person chose and
confirmed. There is nothing to sweep: the node page shows the declared-value view
instead, which is where that number sits inside its own domain. The 18th is
`sw_regime`, whose natural axis is daily Ap, and Ap is not a decision anybody
makes — `segmentation` draws that curve instead.

### 27.3 · Where each ported "number" row's curve actually is

| row | legacy | the number | where its curve is drawn |
|---|---|---|---|
| `sw_recurrence_lag` | `C.rot_peak_lag` | 26 d | `pattern` — the whole autocorrelation, peak marked |
| `sw_recurrence_strength` | `C.rot_peak_r` | 0.376 | same curve, the peak's height |
| `sw_spike_threshold` | `prf_spikes` | — | `pattern` spike view, size against timing |
| `sw_event_duration` | `prf_events` | — | same view |
| `sw_cycle_repeatability` | `C.repeat` | 0.770 | `repeatability` — every cycle stacked on phase |
| `sw_regime` | `parity.csv` | — | `segmentation` — the Ap histogram with the boundaries |
| `sw_semiannual_amplitude` | Climate tab | — | `climate`, by day of year, with both equinoxes marked |
| `sw_ap_central_expectation` | `parity.csv` | 22.095 | `climate` by year; `drivers` as the nominal scenario |
| `sw_mean_band_spread` | `parity.csv` | 13.4544 sfu | **nowhere — see 27.4** |
| `sw_ap_mean_band_spread` | `parity.csv` | 3.5937 | **nowhere — see 27.4** |
| `sw_band_coverage` | `F.band_cov` | 0.9509 | **nowhere — see 27.4** |

Seven of the eleven are read off a curve this face draws in full, at higher
resolution than the study drew it. Four are not.

### 27.4 · What IS lost — four figures, and they are the same family

**The spread rows have no picture of their own spread.** Each of these is a
number computed from a sample the face never shows:

1. **`sw_mean_band_spread` = std(pred − truth) over 361 walk-forward
   next-rotation forecasts = 13.4544 sfu.** Nothing draws the 361 residuals. This
   σ sets the **entire design band** — every scenario is centre ± 1.28 σ — so the
   one number the whole driver table's width rests on is the one with no figure
   behind it. A reader cannot see whether those residuals are normal, which is
   what the 1.28 multiplier assumes.
2. **`sw_ap_mean_band_spread` = 3.5937**, the same construction on Ap, the same
   gap.
3. **`sw_f107a_ratio` = sd of F10.7 / F10.7A over the record = 0.1222.** The
   daily-against-81-day scatter is never drawn. *(Counted as added, not ported.)*
4. **`sw_band_coverage` = 0.9509** — "does the stated 95 per cent band actually
   contain the truth 95 per cent of the time", pooled over seven leads. `predict`
   draws the band's WIDTH as a four-percentile fan; nothing draws whether it
   COVERS. This is the row that audits the band, and it is the one the picture
   does not check.

These are not repeats of anything drawn elsewhere and each was a figure in the
study. **This is the honest answer to "are we losing anything": four, all of them
distribution or coverage pictures behind numbers the design depends on.**

A `spread` panel would close all four — three residual/scatter distributions and
one coverage-against-lead curve, four views — and it is the one real figure gap
left in the subsystem.

### 27.5 · What the 17 added rows are doing

| what | rows | why it is not in the legacy tool |
|---|---|---|
| **the five requirements** | `l3_solar_req_01` … `_05` | the study was an ANALYSIS tool. It measured the sky and never stated what the vehicle must survive, so it had nothing to close against |
| **the five closures** | `l3_solar_ach_01` … `_05` | the signed margin, in the sense the requirement declares. The study computed no margins at all; these are the rows that turn a description of the sky into a design test |
| **what happens past the design** | `sw_exceedance_rate`, `sw_exceedance_duration`, `sw_exceedance_phase`, `sw_design_safe_duration` | the study answered "what Ap do I design to". These answer "and when it is exceeded, how often, for how long, where in the cycle, and how long until that matters" — 0.284 days a year, in 1.14-day events, at cycle phase 0.60, safe for 2.62 years |
| **the design decision itself** | `sw_storm_design_level` | G1/G2/G3 as a row rather than a constant, which is what makes `sw_ap_design` a curve instead of a number |
| **the analogue's own ceiling** | `sw_window_peak_level` | the study used the window MEAN only. This is its maximum, and past 5.9 years it is above the hot single-day design value — see D4 |
| **the daily scatter** | `sw_f107a_ratio` | measured here to size the daily band; the study carried a fixed multiplier |

All seventeen earn their place, and the audit that says so is the one in §26 D4:
every one of them is read by something or drawn by something, and none is free to
remove.

### 27.6 · What is NOT claimed here

This audit checks that a curve EXISTS for each row, not that it is the same curve
the study drew. The legacy figures themselves are not in the repository — only
`mission_drivers.csv`, one run's twenty-five numbers — so "the same picture" is
not a thing that can be checked by machine. What `drivers` does instead is put the
study's numbers beside this tree's and let a reader see the two families of
deliberate disagreement.


---

## 28 · The capability audit — tab by tab, against §12's own table

§27 asked "does every row have a curve" and answered it. That was the wrong
question to stop on. This one asks what the legacy study COULD DO that this tree
cannot, and what a designer cannot get here, working from §12's table of the eight
tabs and what each of them drew.

### 28.1 · The eight tabs

| legacy tab | what §12 says it drew | here | state |
|---|---|---|---|
| `sw_repeatability` | mean cycle against every cycle, Ap by cycle, storm scale | `repeatability`, 10 views | **complete** — all three, and any of F10.7 / Ap / SSN |
| `sw_pattern` | 27-day recurrence lag and decay, spike classification and timing | `pattern`, 3 views | **complete and beyond** — three detrend windows at once, a Bartlett band, three harmonics marked |
| `sw_segmentation` | cycle phases, activity bands | `segmentation`, 5 views | **complete** — the Ap regime and the F10.7 band, plus regime against phase |
| `sw_predict` | 27-day prediction for F10.7 and Ap, significance, by cycle | `predict`, 4 views | **prediction and by-cycle complete; SIGNIFICANCE MISSING** — see 28.4 |
| `sw_forecast` | issued-window verification, rolling windows, daily skill, issue age | `forecast`, 3 views | **verification and issue age complete; ROLLING WINDOWS AND DAILY SKILL MISSING** — the by-year view is calendar-year aggregation, not a moving window |
| `sw_design` | the design window for F10.7 and for Ap | `design`, 11 views | **complete and beyond** — four design rows against two requirements, the G scale, the exceedance family, the analogue's peak |
| `sw_density` | profile, spread, sensitivity, by driver, by altitude | `density`, **1 view** | **NOT PORTED — see 28.3, and the reason recorded in the panel is wrong** |
| `sw_climate` | yearly F10.7 and Ap, Kp–ap, F10.7A, semiannual | `climate`, 13 views | **yearly, Kp–ap and semiannual complete; F10.7A MISSING** — nothing draws daily F10.7 against its own 81-day mean |

Six tabs of eight are reproduced or exceeded. One is missing three views across
`predict`, `forecast` and `climate`. **One — `density` — is not ported at all, and
it is the one that matters most.**

### 28.2 · The finding that outranks every figure: THE STUDY DRIVES NOTHING

This is not about pictures.

```
sys_space_environment_f10_7          104.071    read by NOTHING
sys_space_environment_f10_7_81day    104.071    read by NOTHING
sys_space_environment_ap              26.6953   read by NOTHING
sys_space_environment_kp               5.19777  read by NOTHING
sys_space_environment_solar_flux     104.071    read by NOTHING
```

Every one of the five rows this subsystem crosses to layer 2 is read by nothing at
all. Meanwhile the chain that turns the sky into drag is complete and live:

```
env_f107  = 150   (declared)  ┐
env_f107a = 150   (declared)  ├─→ env_exospheric_temperature = 949.603 K
env_kp    =   3   (declared)  ┘      ├─→ env_mass_density        = 6.63073e-11 kg/m^3
                                     ├─→ env_number_density, env_scale_height,
                                     │   env_local_temperature, env_mean_free_path,
                                     │   env_mean_molar_mass, env_atomic_oxygen_density
                                     └─→ aero_dynamic_pressure = 0.00199378 Pa
                                         orbit_decay_rate      = -0.0141250 m/s
                                         prop_collected_flow, prop_incident_flux,
                                         thm_aero_heating
```

**`env_f107`, `env_f107a` and `env_kp` are declared constants with no inputs**, and
they are what sizes the drag, the decay, the propellant and the aero heating. The
solar subsystem's 55 rows, 38 of them ported from the study, arrive beside them and
are not consulted.

| | the declared sky | the study's own hot sustained scenario |
|---|---|---|
| F10.7 | **150** | **104.07** |
| F10.7A | **150** | **104.07** |
| Kp | **3** | **5.198** |

Both differ, and in opposite directions: the declared flux is 44 per cent above
what the study computes, and the declared disturbance is 42 per cent below it. A
design sized on 150 / 3 is being told a denser quiet sky than the record supports
and a calmer storm sky than the record supports, at once.

§13 made this deliberately — *"`env_f107`, `env_f107a` and `env_kp` are not
touched — the window-derived sky becomes available beside the design point, not
instead of it"* — and at the time the subsystem had five rows. It now has
fifty-five and a crossing that publishes twenty-five variables, and "available
beside" has become "computed and ignored".

**This is the answer to "what am I missing from a design-capability
perspective."** It is not a figure. The study is complete as a study and connected
to nothing as a design input.

Closing it is a decision, not a task, and the decision is which of three:
`env_f107`/`_f107a`/`_kp` become `computed` and read the crossing; or the density
chain reads `sys_space_environment_*` directly and the three constants are
retired; or they stay declared as a deliberate override and something states, on a
row, that the study's answer was considered and not used. All three are cheap. The
present state — two skies, one computed and one typed, neither aware of the
other — is the only one that cannot be defended.

### 28.3 · The density tab, and a wrong reason recorded in the product

`panels/density.toml` draws one scatter and its note says:

> the study's density tab — profile, spread, sensitivity, by driver, by
> altitude — needs an atmosphere model, and that belongs to a different subsystem
> which has nothing written in it

**That is not true, and it was probably true once.** The `env` subsystem holds
sixteen rows, and the atmosphere model is published and answering right now. Every
sweep the five legacy views need already works:

| legacy view | the sweep that draws it | measured |
|---|---|---|
| profile | `env_mass_density` over `orbit_altitude` | 1.19e-12 … 2.28e-09 kg/m³ across 150–450 km |
| by altitude | `orbit_decay_rate` over `orbit_altitude` | −0.383 … −0.0003 m/s |
| sensitivity | `env_mass_density` over `env_f107` | 5.02e-11 … 1.15e-10 kg/m³ across F10.7 60–400 |
| sensitivity | `env_mass_density` over `env_kp` | 5.45e-11 … 1.29e-10 kg/m³ across Kp 0–9 |
| spread | `env_density_uncertainty` = 0.240 as a band on the profile | a row, live |
| by driver | the same profile at each of the five driver scenarios | the crossing publishes all five |

So the density tab is five views away, not a subsystem away. And the "sensitivity"
and "by driver" views are the ones that would put 28.2 on a screen: sweep density
over F10.7 and mark both 150 and 104.07 on it, and the gap stops being a paragraph
in a plan.

### 28.4 · The three legacy views inside otherwise-complete tabs

1. **`predict` · significance.** `sw_band_coverage` = 0.9509 answers "does the
   stated 95 per cent band actually contain the truth". `predict` draws the band's
   WIDTH as a four-percentile fan and nothing draws whether it COVERS. The row
   that audits the band is the one the picture does not check.
2. **`forecast` · rolling windows and daily skill.** The by-year view aggregates
   by calendar year of issue, which is not a moving window. A rolling skill series
   answers "is the outlook getting better", and a calendar-year one answers it
   only at twelve-month resolution with a hard boundary every December.
3. **`climate` · F10.7A.** Nothing draws daily F10.7 against its own 81-day
   centred mean. The 81-day mean is computed inside `spikes()` as a baseline and
   never shown, `sw_f107_81day` is deprecated, and `sw_f107a_ratio` = 0.1222 —
   the scatter about it — has no figure either (§27.4).

### 28.5 · So: is it sufficient

**As a solar-weather study — yes, with four figures and three views outstanding.**
Six of eight tabs are reproduced or exceeded. What is missing is listed in 28.4
and §27.4 and is seven pictures, every one of them a distribution, a coverage or
a moving-window view behind a number that already exists and is already checked.

**As a design capability — no, and the gap is not where the figures are.** The
subsystem computes a sky nothing reads, while the drag, decay, propellant and
heating chains are sized by three typed constants that disagree with it in both
directions. Until 28.2 is decided, every figure added here improves a study that
the design does not consult.

**The order that follows from this, and it is not the order the figures suggest:**

1. **Decide 28.2.** One of the three readings. It is a decision about the tree,
   not work on the face.
2. **Build the `density` panel**, five views, from rows that already answer. It is
   the missing tab, and two of its views make 28.2 visible.
3. **Build the `spread` panel**, four views (§27.4), and add `predict`'s coverage
   view, `forecast`'s rolling window and `climate`'s F10.7A overlay.

Step 1 is the one that changes what the tool can do. Steps 2 and 3 are what make
it legible.


---

## 29 · Is the solar-weather study sufficient, on its own terms

Density and everything downstream set aside. This asks only: as a piece of solar
weather work, is what is here enough, and if not what is needed. Everything below
is measured, and where a number is quoted the command that produced it is in the
paragraph.

### 29.1 · What is sufficient, and one of these is better news than it looks

**The driver set matches the atmosphere model's input set exactly.** The tree's
exospheric temperature is Jacchia-71:

    T_inf = 379 + 3.24*F10.7A + 1.3*(F10.7 - F10.7A) + 28*Kp + 0.03*exp(Kp)

Its drivers are F10.7A, F10.7 and Kp, and those are precisely the three the
subsystem computes, at five scenarios each. There is no missing quantity for the
model this repository actually uses. A worry worth naming and dismissing: MSIS
would want the 3-hourly ap history and JB2008 would want S10.7, M10.7, Y10.7 and
Dst, none of which is in the bundle — but neither model is in this tree, so
neither is a gap in the study.

Six of the eight legacy tabs are reproduced or exceeded (§28.1). All 55 live rows
answer; nothing refuses. 14 rows carry the legacy run's own answer as `parity.csv`
and 263 fixtures stand behind the subsystem.

### 29.2 · WHICH Kp FEEDS THE MODEL IS UNDECIDED, and it is worth more than any figure

The study publishes **two** Kp columns per scenario — `kp_mean` and `kp_peak` —
because `sw_kp_slot_bias` established that the published ap-to-Kp table under-reads
the daily peak. Jacchia-71 takes **one** Kp. Nothing in the tree says which, and
the difference is not small:

| scenario | Kp mean | Kp peak | T_inf(mean) | T_inf(peak) | ΔT |
|---|---|---|---|---|---|
| quietest day | 1.271 | 2.230 | 634.6 K | 661.6 K | **27.0 K** |
| cold sustained | 3.166 | 4.277 | 694.0 K | 726.5 K | **32.6 K** |
| nominal | 3.548 | 4.796 | 760.8 K | 798.3 K | **37.5 K** |
| hot sustained | 3.832 | 5.198 | 824.9 K | 867.2 K | **42.3 K** |
| worst day | 5.802 | 7.997 | 914.7 K | 1055.3 K | **140.6 K** |

On the worst day the two readings of the same scenario are **15 per cent apart in
exospheric temperature**, and density at a fixed altitude goes as roughly
exp(−h/H) with H proportional to T, so the spread in density is larger again.

This is the largest open question in the subsystem and it is a **solar-weather**
question, not a plumbing one: is the driver the day's mean disturbance or the
disturbance at its worst three-hour slot. Both are defensible and they are
different designs. What cannot be defended is publishing both and naming neither.

*What is needed:* one declared row — which slot the design is driven by, and why —
that the interface reads, so the choice is a decision somebody signed rather than
whichever column a consumer happens to pick up.

### 29.3 · The published band is not at the confidence it is labelled, and its two halves disagree

`sw_f107_design_long`'s own hole says it plainly:

> 1.28 is the confidence, and it is the 90th percentile while the run is called 95
> per cent. Phi(1.28) = 0.8997. A one-sided 95 per cent bound is 1.645 sigma, which
> at this sigma is a further 4.9 sfu. **The daily half of the same band DOES use
> 0.95**, so the two halves are not at one confidence, and this row reproduces that
> rather than silently repairing it.

So within a single scenario the **sustained** level is a one-sided 90 per cent
bound and the **daily** level a one-sided 95 per cent one, and the run is labelled
95 per cent throughout. Eight rows carry this. Reproducing the legacy faithfully
was the right call at port time — a port that silently repairs its source is a port
nobody can check — but it is now a published driver table whose header does not
describe it.

*What is needed:* a declared `sw_band_confidence` row that both halves read, set
to whatever a person decides, so the two cannot drift apart again and the label
and the multiplier are one fact. Moving it to 1.645 raises the sustained F10.7
scenarios by about 4.9 sfu; leaving it at 1.28 is fine too, as long as the number
and the word agree.

### 29.4 · A z-multiplier is used where an empirical percentile is available

Three rows assume the residual spread is normal enough for a z multiplier to mean
a percentile. `sw_f107_design_long`'s second assumption states the cost:

> The residuals of a forecast that misses hardest when activity is highest are
> skewed, and a normal multiplier under-covers the high tail — which is the tail a
> design is sized against. The empirical percentile of the residuals would be the
> honest statistic, and `sw_mean_band_spread` publishes only their standard
> deviation.

The 361 walk-forward residuals exist. What is published from them is one number,
σ = 13.4544 sfu, and §27.4 already found that nothing draws them either. So the
same sample is under-used twice: no picture, and no empirical quantile.

*What is needed:* `sw_mean_band_spread` publishes a set — σ and the empirical
90th/95th/99th of the same residuals — the way `l3_solar_interface` publishes a
set. Then a design can be sized on the record's own tail rather than on a normal
assumption about it, and the figure §27.4 asks for draws the sample both are read
from.

### 29.5 · Eight columns of the record are read by nothing

`observed_daily.csv` carries nineteen columns. The tree reads four — `f107`,
`ap_planetary`, `kp_max`, `ssn_sesc`. Unread anywhere, by any row or any figure:

| unread column | what it is | why it is or is not a gap |
|---|---|---|
| `kp_00z` … `kp_21z` | all eight 3-hour Kp slots | **the strongest case.** `sw_kp_slot_bias` exists to say the daily peak differs from the table, and it does it from a tabulated fit. The slots are the measurement it is a fit OF, and they also carry the diurnal profile — which is what 29.2 is really asking about |
| `a_college` | the auroral-zone A index (College, Alaska) | **a real physical gap.** Storm energy enters the thermosphere at high latitude; `ap_planetary` is a planetary average. A VLEO mission in a high-inclination or sun-synchronous orbit spends much of each pass in the auroral oval, and the record holds the index for it |
| `a_fredericksburg` | the mid-latitude A index | the low-latitude counterpart; together with College it gives the latitude contrast the planetary index averages away |
| `flares_c`, `flares_m`, `flares_x` | daily flare counts by class | a different hazard entirely — dose, single-event upsets, HF and GNSS scintillation — and this tool has `payload`, `com`, `navsense` and `fsw` subsystems that would read it. Not a density question, which is why it is last, but it is solar weather and the data is here |
| `sunspot_area` | a second EUV proxy | lowest priority; `ssn_sesc` is already read |

`alerts.csv` — 17,987 issued alerts with their trigger thresholds — is read only by
`sw_alert_threshold`, which is **deprecated for a stated and defensible reason**
("operations is not a subsystem this tool has"). That one is a decision, not an
oversight.

### 29.6 · Three numbers for one quantity

The cycle-amplitude ratio the analogue is scaled by appears three times and does
not agree with itself:

- `solar_cycles.csv`: peaks 196.4 and 146.1 — a ratio of **1.344**
- the `repeatability` panel, phase-stacked from the daily record: 207 against 149
  — a ratio of **1.389**
- three sheets' assumptions: *"two completed cycles whose peaks differ by **41 per
  cent**"* — a ratio of 1.41

Small in effect and exactly the drift this tool exists to prevent: the same
quantity, stated by hand in three places, already disagreeing. *What is needed:* it
becomes a row, and the three places read it.

### 29.7 · The standing statistical caveats, already declared

These are on the sheets already and are listed so the answer is complete rather
than because they are new:

- the return level's top end **rests on two observations in 28.2 years**
- **overlapping pairs are counted as independent** in the recurrence significance
- the daily scenarios **stack two one-sided percentiles**, giving something nearer
  a 1-in-100 day than the 1-in-20 the name suggests
- eight rows are **pooled over leads or levels rather than conditioned** on them
- `daily_regime.csv` **disagrees with the record on 171 of 10,299 days**, all at Ap
  0 or 1, understood and documented
- beyond one cycle past cycle 25's maximum the analogue is **scaled by a mean
  amplitude**, which does not bite at the declared 5-year duration from a 2027
  epoch but does across most of the declared 15-year upper bound

### 29.8 · The answer

**As a solar-weather study, it is sufficient in coverage and not yet sufficient in
statement.** Nothing is missing from what it measures for the model it drives. What
is missing is that three things it measured are not yet said clearly enough to
design against:

1. **which Kp is the driver** — worth up to 141 K of exospheric temperature, and
   undecided;
2. **what confidence the band is at** — currently 90 per cent on one half, 95 on
   the other, and 95 in the label;
3. **the tail of the residual sample** — published as a standard deviation and a
   normal assumption where the empirical quantiles are sitting right there.

Then the data gap: **eight columns of the record are read by nothing**, and of
those the eight Kp slots and the auroral index are the two that would change
answers rather than add views.

In order, and none of it is large:

| | what | why first |
|---|---|---|
| 1 | a declared row naming **which Kp slot drives the design** | the largest unstated number in the subsystem |
| 2 | a declared `sw_band_confidence` both halves of the band read | the published table does not match its own label |
| 3 | `sw_mean_band_spread` publishes **empirical quantiles** beside σ | the sample is there and the normal assumption is not free |
| 4 | the **eight Kp slots** read, so the slot bias is measured rather than tabulated | it is the measurement behind item 1 |
| 5 | **`a_college`** read, as the auroral-zone counterpart to planetary Ap | the only unread column that is a different physical quantity rather than a finer view of one already read |
| 6 | the cycle-amplitude ratio becomes **a row** | three hand-written copies, already disagreeing |
| 7 | the flare counts, if the radiation and comms subsystems are ever wanted | real solar weather, real data, no consumer yet |

Items 1 to 3 are decisions and cost almost nothing to implement once decided.
Items 4 to 7 are work.


---

## 30 · The plan: exospheric temperature, and the three things not said clearly

Two workstreams. **A** is the exospheric temperature row itself, which is where
the solar drivers first become physics. **B** is §29's three statements. They
overlap at exactly one point — which Kp — and that point is done once, in B1, and
read by A.

**What I cannot do, stated first.** Three of these steps need a number a person
chooses. `AGENTS.md`: *an agent may never supply mathematics, and without an
attribution nothing can tell whether one did*. Every declared value below is
marked **[needs a person]** and the step stops there until it has one. Where there
is evidence for what the number should be, the step carries the evidence and not a
decision.

### Workstream A · Exospheric temperature

The row is `env_exospheric_temperature`, subsystem `env`, crate
`vleo-mod-envorbit`. It computes
`T_inf = 379 + 3.24·F10.7A + 1.3·(F10.7 − F10.7A) + 28·Kp + 0.03·exp(Kp)` from
`vleo_core::physics::env::exospheric_temperature`, and it answers 949.603 K today.

**A1 · The fixtures do not test the relation, and that is the first thing to fix.**

All three fixtures set `f107 == f107a`:

| fixture | F10.7 | F10.7A | Kp | expects |
|---|---|---|---|---|
| solar minimum, quiet | 70 | 70 | 1 | 633.9 |
| moderate activity | 150 | 150 | 3 | 949.6 |
| solar maximum, storm | 250 | 250 | 7 | 1417.9 |

So `1.3·(F10.7 − F10.7A)` is multiplied by zero in **every** case. **The
coefficient 1.3 could be any number at all and all three fixtures would still
pass.** That term is what carries the daily departure from the 81-day mean — it is
the whole of the short-term response — and nothing checks it.

Worse, each expected value is the formula evaluated by hand: 379 + 3.24·70 + 28 +
0.03·e¹ = 633.88. That is arithmetic verification of the implementation against
the sheet, not evidence that the sheet matches Jacchia 1971. Their provenance says
`published-source`, and what they are is a restatement.

*The step:* add at least three fixtures whose expected values come from published
Jacchia-71 worked examples or an independent implementation — **[needs a person]**
for the source — and among them at least two with `F10.7 ≠ F10.7A` in both
directions, and one that separates `28·Kp` from `0.03·exp(Kp)` (at Kp 7 the
exponential is 32.9 K of 229 K, so it is separable there and invisible below Kp 4).

*Green when:* `cargo run -p xtask -- gate env_exospheric_temperature && cargo
test`. A fixture disagreement is a physics disagreement and goes to the owner.

**A2 · The row has one assumption and needs five.**

It declares *"Night-time minimum, with no diurnal or seasonal term"*, which is
right and is worth up to 30 per cent at 14:00 local solar time. Unstated, and each
of these changes what the number means:

1. **Kp here is a 3-HOURLY index and it is being fed a daily statistic.** J71's
   geomagnetic term takes the Kp of the interval. This tree hands it a daily mean
   or a daily peak — §29.2 — and the choice is worth 27 K at the quiet end and
   140.6 K on the worst day. Whatever B1 decides, this row must SAY which it takes.
2. **No geomagnetic lag.** J71 applies its correction to Kp lagged by about 0.25
   day; this applies it instantaneously, so a storm's heating arrives too early.
3. **F10.7 is the previous day's value in J71**, because the EUV that heated the
   thermosphere is yesterday's. This reads today's.
4. **No semiannual term**, which the record's own `sw_semiannual_amplitude`
   measures and this row does not take.
5. **The coefficients are the global night-time form**; J71 has variants and
   nothing here says which.

*The step:* five `[[assumption]]` blocks with `fails_when` clauses. Prose only —
no number moves. This is the cheapest step in the plan and probably the most
valuable, because every one of these is a way the number is read as more than it
is.

**A3 · The relation has nobody's name against it.**

`maths.confirmed_by` is absent and there is no `[theory]` block, so `xtask gap`
reports it twice: *the relation is stated but not derived*, and *an agent may never
supply mathematics*. **[needs a person]** to read the relation against Jacchia 1971
and sign it. I can draft the `[theory]` derivation — why 379, why the two-part
solar term, why the exponential in the geomagnetic one — for that person to check.

**A4 · A figure, because this is the row you want to look at.**

A `thermosphere` panel, 4 views, on `env_exospheric_temperature`:

| view | what it draws |
|---|---|
| against F10.7 | T_inf swept over `env_f107`, with the five driver scenarios marked and the declared `env_f107 = 150` marked beside them |
| against Kp | T_inf swept over `env_kp`, with **both** Kp readings of each scenario marked — this is §29.2 as a picture |
| the two Kp readings | the five scenarios as paired bars or a slope chart, mean against peak, so the 27 K … 140.6 K spread is one glance |
| the terms | the four terms of the relation stacked, so a reader sees that at Kp 3 the exponential is 0.6 K and at Kp 7 it is 32.9 K |

Every sweep it needs already works. This is the panel that makes B1 a decision
somebody can take by looking rather than by reading a table.

**A5 · The drivers this row reads — NOT done in this plan.**

It reads `env_f107 = 150`, `env_f107a = 150`, `env_kp = 3`, which are declared
constants, while the solar subsystem computes 104.07 / 104.07 / 5.198 and is read
by nothing (§28.2). Everything above is worth doing whichever way that goes, and
none of it depends on it. It is listed here so that it is not forgotten, and it is
yours to decide.

### Workstream B · The three statements

**B1 · Which Kp slot drives the design.**

*The evidence:* the legacy run's own header says **`Kp slot 'mean'`**. So the study
answered this for its own run and the answer was the daily mean. The tree
publishes both columns and names neither, and `sw_kp_slot_bias` exists precisely
because they differ.

*The step:* one new declared row in `vleo-mod-solar`:

```
id      = sw_kp_driving_slot
kind    = declared
question: Which Kp slot is the design driven by — the day's mean, or its worst
          three-hour interval?
```

**[needs a person]** for the value, and the choice is not obvious: the mean is what
the legacy run used and what a daily-averaged density model wants; the peak is what
a vehicle actually meets, and on the worst-day scenario it is 140.6 K hotter. A
defensible third answer is *both, and the design closes against the peak while it
is sized on the mean*, which would make this row a set rather than a switch.

Then `env_exospheric_temperature` reads it, or — if A5 stays undecided — states in
A2's first assumption which slot its declared `env_kp` is meant to be.

*Green when:* gate, and `sw_kp_slot_bias`'s sheet cross-references the new row so
the two cannot drift.

**B2 · What confidence the band is at.**

*The evidence, and it is worse than one wrong label.* The sustained rows use
`1.28σ`, which is Φ(1.28) = 0.8997 — the one-sided 90th percentile. The daily rows
use `0.95`. The run is labelled 95 per cent. **So the two halves of a single
scenario are at different confidences**, and neither half is at the label's.

*The step:* one new declared row, `sw_band_confidence`, that both halves read, so
there is one number and one name for it. **[needs a person]** for the value, and
the decision has a cost either way:

| | what it does | what it costs |
|---|---|---|
| keep 0.8997 | the label becomes true, no number moves | the driver table stays at a 90 per cent bound while the study's prose says 95 |
| move to 0.95 (z = 1.645) | one confidence throughout, matching the label | the sustained scenarios rise about 4.9 sfu and **parity with `mission_drivers.csv` is lost** — 14 rows hold the legacy run's own answers and four of them would stop matching |

*My reading, offered and not taken:* this repository's discipline is to reproduce
the study and say where it is wrong, not to repair it silently — so keeping 0.8997
and making the label true is more in character, with B3 providing the correct
number beside it rather than instead of it. But it is a design decision about a
published bound and it is yours.

**B3 · The empirical quantile, beside the z-multiplier rather than instead of it.**

*Why this is the same problem as B2.* `centre + z·σ` assumes the residuals are
normal. `sw_f107_design_long`'s own sheet says they are not: *"the residuals of a
forecast that misses hardest when activity is highest are skewed, and a normal
multiplier under-covers the high tail — which is the tail a design is sized
against."* The 361 walk-forward residuals exist. One number is published from them.

*A constraint that shapes this step:* the engine does not read the bundle at
runtime — measurements become declared values or tables baked into holes. And the
gate **refuses `[[publishes]]` on a declared row**: *"a declared row states one
measured number; a set is computed from the rows that measured its members."* So
this cannot be three members on `sw_mean_band_spread`. It is sibling rows.

*The step, in three parts:*

1. **`tools/rotation_residuals.py`** — reproduce the walk-forward that produced
   σ = 13.4544 sfu, from `observed_daily.csv`, and print the 361 residuals' σ
   alongside their empirical 90th, 95th and 99th. The first output that matters is
   whether σ comes back at 13.4544: if it does not, the script is wrong and nothing
   below it is trustworthy. Same for Ap against 3.5937.
2. **Two new declared rows** — `sw_mean_band_p95` and `sw_ap_mean_band_p95` — the
   empirical quantile of the same sample. **[needs a person]** to confirm each
   value, exactly as σ was confirmed by A. Rai on 2026-09-16.
3. **A view on the `spread` panel** (§27.4's panel, still unbuilt) drawing the 361
   residuals with `z·σ` and the empirical quantile both marked on them. That
   picture is the argument: if the two coincide the normal assumption was fine and
   this was cheap; if they do not, the gap is the under-coverage, in sfu, visible.

*What it does NOT do:* it does not change a design value. No design row reads the
new quantiles until somebody decides they should, which is a fourth decision and
is deliberately not in this plan.

### The order, and what each step leaves green

| # | step | needs a person | moves a number |
|---|---|---|---|
| 1 | **A2** five assumptions on `env_exospheric_temperature` | no | no |
| 2 | **A1** fixtures that test the 1.3 term and the exponential | yes — the source | no |
| 3 | **A3** `[theory]` and the relation's signature | yes — the signature | no |
| 4 | **B3.1** the walk-forward script, checked against σ | no | no |
| 5 | **B1** `sw_kp_driving_slot` | **yes — the value** | yes, downstream |
| 6 | **B2** `sw_band_confidence` | **yes — the value** | yes, if 0.95 |
| 7 | **B3.2** the two empirical-quantile rows | **yes — both values** | no |
| 8 | **A4** the `thermosphere` panel, 4 views | signature on the reference | no |
| 9 | **B3.3** the residual view on the `spread` panel | signature on the reference | no |

Steps 1 to 4 need nothing from you and change no number — they are the evidence
the rows are currently missing. Steps 5 to 7 are the three decisions. Steps 8 and 9
are the pictures that make 5 and 6 answerable by looking.

Every step ends green on `cargo run -p xtask -- gate && cargo test`, and steps 8
and 9 additionally on `python3 tools/panel_check.py` and its `--selftest`. Steps 8
and 9 produce reference images that **an agent may not sign**.


---

## 31 · Steps 1 to 4, done — and what they found

### 31.1 · A2, the five assumptions — done, no number moved

`env_exospheric_temperature` now declares six assumptions instead of one. The five
added are in §30 A2 and they are all about what this tree HANDS the relation
rather than about Jacchia 1971: the Kp is a three-hourly index fed a daily
statistic, there is no geomagnetic lag, J71's F10.7 is the previous day's and this
reads today's, there is no semiannual term on a record that measures one, and
nothing records which coefficient variant these four are.

### 31.2 · A3, the theory — drafted, and deliberately unsigned

A `[theory]` block with a four-step derivation: the intercept, the slow solar term
on the 81-day mean, the fast term as a DEPARTURE from that mean and why its
coefficient is smaller, and the geomagnetic term in two parts because Kp is
quasi-logarithmic. `reading` says what the answer is and is not — a night-time
minimum in a global average, and a fit against 1960s and 70s drag data.

`maths.confirmed_by = ""`, with a comment saying why. **A person has to read that
derivation against Jacchia 1971 and sign it.** An agent may not, and an unsigned
relation is indistinguishable from one an agent supplied.

### 31.3 · A1, the fixtures — done, and the gap was worse than stated

The three fixtures all set `f107 == f107a`. §30 predicted that the `1.3` coefficient
was therefore untested. **Measured by breaking the kernel, not by reading it:**

| break | what it does | old three | the five added |
|---|---|---|---|
| `1.3` → `9.9` | a seven-fold error in the fast term | **all pass** | 2 fail |
| exponential deleted | the storm tail removed entirely | Kp 7 catches it | 5 fail |
| `0.03` → `0.0305` | a 1.7 per cent error in the exponential | **all pass** | **all 5 fail** |

So the old set was weaker in two ways, not one. It does not test the fast term at
all — and a seven-fold error in it passes — and its tolerance of 0.002 relative is
±2.19 K at Kp 7 where the exponential contributes 32.9 K, so it admits a 1.7 per
cent error in that coefficient too. It covers whether the storm term is present,
not what size it is.

The five added are pairs that differ in exactly one thing, so each pins a
coefficient by a difference: flux 50 above and 50 below its own mean straddle the
equal case by ±65 K, which is 1.3 × 50; and Kp 7 against Kp 8 differ by 84.529745 K
where without the exponential they would differ by exactly 28. Their provenance is
`independent-derivation` and not `published-source`, and the fixtures file says
why: they are the published relation evaluated at chosen points, which proves the
implementation computes what the sheet states and **not** that the sheet states
Jacchia's relation. Nothing in that folder proves the second, which is what 31.2 is
waiting on.

### 31.4 · B3.1, the walk-forward — written, and IT DOES NOT REPRODUCE THE NUMBER

`tools/rotation_residuals.py` implements `sw_mean_band_spread`'s five theory steps.
The plan said the first output that matters is whether σ comes back at 13.4544.
**It does not:**

| | rotations | σ here | σ published | |
|---|---|---|---|---|
| F10.7, Yule-Walker | 376 | 14.3139 | 13.4544 | **+6.39 %** |
| F10.7, least squares | 376 | 14.3844 | 13.4544 | **+6.91 %** |
| Ap, Yule-Walker | 376 | 3.5264 | 3.5937 | −1.87 % |
| Ap, least squares | 376 | 3.5791 | 3.5937 | −0.41 % |

**Two differences, and they are the work left.** The rotation COUNT is 376 against
361. The record is 10,592 days on a dense grid, which is 392 rotations of 27, and
this scores every one past a warm-up of 16; reaching 361 needs a warm-up of 31 and
the sheet states none. Dropping the 13 rotations that contain an interpolated day
would give 379, and the sheet's own third assumption says those days **were**
counted, so that is not the route either.

The estimator is the other candidate and is not enough alone: the sheet says "an
AR(2) fitted on the training anomalies" without naming one, and least squares moves
Ap to −0.41 % while moving F10.7 the wrong way to +6.91 %. **That Ap nearly lands
and F10.7 does not says the difference is not one systematic choice.**

**What was NOT done, on purpose.** The free parameters were not tuned until the
number matched. A walk-forward whose warm-up and estimator were chosen to reproduce
13.4544 agrees with it by construction and is evidence for nothing — and the
quantiles taken off it would be the thing §30 B3.2 asks a person to sign.

**So the script refuses to print them.** It gates on both σ and the rotation count,
and prints the reason instead. The count is the stricter of the two and it is
there because of a near miss: Ap under least squares lands within half a per cent
while its sample is 376 rotations rather than 361, and printing quantiles off that
because one of the two agreed would be reporting a coincidence in the right units.

*This is the honest state of step 4: the tool exists, its disagreement is recorded,
and the two open differences are named. B3.2 stays blocked until one of them
closes, which is correct — it was blocked before and nothing knew.*

### 31.5 · Where this leaves §30

| step | state |
|---|---|
| 1 · A2 five assumptions | **done** |
| 2 · A1 fixtures | **done**, and the gap measured rather than asserted |
| 3 · A3 theory | **drafted**, unsigned — needs a person |
| 4 · B3.1 walk-forward | **written and failing its own test**, which is the finding |
| 5 · B1 which Kp slot | needs a person |
| 6 · B2 band confidence | needs a person |
| 7 · B3.2 empirical quantiles | **blocked on 31.4**, not on a person |
| 8 · A4 thermosphere panel | ready to build |
| 9 · B3.3 residual view | blocked with 7 |

Step 8 is the next one that can be done without a decision.


---

## 32 · Step 8, the `thermosphere` panel — done

Four views on `env_exospheric_temperature`, the row where the solar drivers first
become a physical quantity. `panels/thermosphere.toml`, reference recorded,
**UNCONFIRMED** — an agent may not sign one.

### 32.1 · The rule this panel was built under

**No coefficient of the relation appears anywhere in the panel's source.** §21
found three of `design`'s numbers going stale because the figure carried a row's
constants; here the temptation was worse, because Jacchia 1971 is four
coefficients and a decomposition of them is the obvious figure to draw.

So every shape is measured off the engine instead:

- the three slopes in the flux view come from three sweeps, not from the sheet;
- the straight line in the fourth view is drawn through the measured curve's own
  quiet end, so the exponential's departure is measured against the measurement;
- the two dots in the Kp view are runs at the exact Kp, not points read off a
  sweep.

**And it works as a check on the sheet.** The flux view recovers 3.24 K per sfu
for a sustained rise, 1.30 for a single day departing from a fixed 81-day mean,
and 1.94 for the mean moving under a fixed day — the sheet's own coefficients,
arrived at from the other side. The `correct` block now requires those three
numbers, which means the check fails if the panel ever starts carrying them as
literals instead.

### 32.2 · The four views

| view | what it says |
|---|---|
| **against the flux** | three lines, because "raising the flux" is three different quantities and the relation answers differently for each. They cross at the declared 150 where there is no departure. Two marks: `env_f107 = 150`, what the density chain is sized on, and 104.07, what the solar subsystem computes — §28.2, on an axis |
| **against Kp** | five curves, one per scenario at its own flux, as five steps of one hue because the scenarios are a ladder. Two dots on each: the day's mean slot and its worst slot |
| **the two readings** | the same fact with the curves removed. The gap runs **27.0 K at the quietest day to 140.6 K at the worst day** |
| **the geomagnetic shape** | the term's own contribution against the straight line its quiet end sets. By Kp 9 the measured curve is **242.2 K above** it |

### 32.3 · Three things found by building it

**Two views of one fact disagreed in the first decimal.** The Kp view read its
dots off a 46-point sweep and put the worst-day gap at 141.1 K; the slot view ran
the engine at the exact Kp and got 140.6. Both now run. The `correct` block
requires the two notes to agree, so it cannot come back.

**A data series was wearing the threshold colour.** `INK.series[4]` is `#c2185b`,
which this face uses for every line a value is measured against — the zero rules,
the requirement line, "exact agreement", and this panel's own `env_kp` mark one
view earlier. I picked it by index without noticing. The worst-slot line is now
`INK.series[3]`, and the `correct` block says so.

*The same collision again, one view over.* The mark for the computed sky was
drawn in `INK.series[2]`, which is the green the "81-day mean alone" line uses in
the same frame — a mark wearing a series' colour invites the reader to pair the
two. It is `INK.mark` now. Two colour mistakes of the same shape in one panel is
worth recording: picking an ink by slot index does not ask what else is already
wearing it.

**Five unnamed series became five identical table columns.** The dots are marks on
curves the table already carries, so they now set `aside: true` — the flag §27
added for the Bartlett band. Without it the table grew five columns all headed
"exospheric temperature [K]", which is what an unnamed series falls back to, and
the readout announced every value twice.

### 32.4 · Two helpers the engine boundary gained

`engineSweep` takes an optional `sets`, and `engineAt(node, sets)` runs one row
with declared values held elsewhere. Both pass the same `set=` the run endpoint
takes, so a swept point and a run at the same place agree — which is what let the
two views above be reconciled rather than explained.

They are what makes a *scenario* drawable at all: a sweep with nothing held asks
"what does this row do as X moves", and the five driver scenarios are five
different places to stand.

### 32.5 · Where §30 stands

| step | state |
|---|---|
| 1 · A2 five assumptions | done |
| 2 · A1 fixtures | done |
| 3 · A3 theory | drafted, **unsigned — needs a person** |
| 4 · B3.1 walk-forward | written, **failing its own test**, §31.4 |
| 5 · B1 which Kp slot | **needs a person**, and the third view is now the picture to decide it from |
| 6 · B2 band confidence | **needs a person** |
| 7 · B3.2 empirical quantiles | blocked on step 4 |
| 8 · A4 thermosphere panel | **done**, reference unsigned |
| 9 · B3.3 residual view | blocked with 7 |

Everything an agent can do without a decision is now done. Steps 5 and 6 are two
numbers; step 3 is a signature; step 4 is the one piece of open work, and it is
work rather than a decision.


---

## 33 · All 63 figures on one sheet, and what that shows

Every view of every panel captured as a canvas and tiled — 63 of them, which is
the first time the figure set has been looked at as a set rather than one at a
time. `tools/` does not hold the capture script; it was scratch, and the findings
below are what it was for.

### 33.1 · NINE VIEWS, ONE CURVE — measured, not eyeballed

`design`'s Ap branch has nine states: three G levels times three requirements. On
the contact sheet they are visibly the same picture. Measured by hashing the
drawn series across all nine:

```
distinct CURVE sets across the nine Ap views: 1
```

One. The curve is `sw_storm_return_level` against mission length and neither
control touches it. What moves is three marks — and they take only three distinct
positions, because the crossing depends on the G level alone:

| | G1 | G2 | G3 |
|---|---|---|---|
| the design level | Ap 48 | Ap 80 | Ap 132 |
| exceeded at | 0.50 yr | 0.74 yr | 2.62 yr |

The three requirement options move a second mark to 207, 48 or 132 and change
nothing else. **So nine frames carry one curve, three crossing points and three
bound positions**, and a reader who wants the comparison — which G level survives
which requirement, and for how long — has to click through nine states and
remember.

Part A hunted repeats and did not catch this, correctly: the pictures are not
identical, a mark does move, so "it moves" passes. They are not repeats. They are
**one small-multiple grid that has been unrolled into a control**.

The same shape is elsewhere:

| panel | views | what the knob actually is |
|---|---|---|
| `design`, Ap | 9 | one curve, a 3×3 grid of marks |
| `repeatability`, stack | 9 | 3 variables × 3 bin counts — bins is a smoothing knob, not a view |
| `segmentation`, hist | 4 | 2 variables × lin/log — the scale is a knob |
| `drivers`, set | 5 | five quantities across five scenarios — a 5-up row IS the driver table |

### 33.2 · The idea that follows: a grid, not just a column

`spec.panes` already stacks frames vertically on one x axis; that is what
`forecast` uses. Adding `cols` makes it a grid, and the four rows above become
four single views that say more than the twenty-seven they replace:

- **`design`** — one 3×3 of small frames, same curve and same scales in every
  cell, each with its own pair of marks. The design question is a table of
  crossing points and this draws that table.
- **`repeatability`** — a 3-up row, one per variable, bins staying a knob.
- **`drivers`** — a 5-up row, which is the legacy driver table as a picture
  rather than as five clicks.

This is the highest-leverage change available to the figure layer: it is one
feature in `chart.js`, it uses machinery that already exists, and it converts the
set's largest redundancy into its most comparative view.

### 33.3 · Not one of the 63 views uses a band, and at least six want one

`kind` is used 44 times as `line`, 4 as `bars`, 3 as `dots`, and **0 as anything
filled**. The layer has no area. Six places where the note already calls the
thing a band and the picture draws its edges:

| view | what the prose says | what is drawn |
|---|---|---|
| `predict` | "four percentiles as a fan" | four lines |
| `design`, F10.7 | "the band between the outer two is the design window" | four lines |
| `pattern` | "the dashed band is the 95 per cent interval" | two dashed lines |
| `closure` | the margin between required and achieved | two lines, the area unmarked |
| `climate`, by day of year | the semiannual swing | one noisy line through 5-day bins |
| `forecast` | skill against persistence | a line with no uncertainty at all |

One `kind: 'band'` taking `y0` and `y1`, filled at low alpha in the series
colour, fixes all six. It is a small addition and it is the difference between a
figure that shows a region and one that shows the region's edges and asks the
reader to fill it in.

### 33.4 · Four smaller things the sheet made obvious

**`forecast`'s issue-age view is one bar at about 99 per cent** and two bars too
short to see. It is a count histogram of a quantity whose whole mass is in one
bin, so the frame carries one fact and a lot of floor. It wants a log count axis
or a different form.

**`climate` by day of year is noise.** Three of its twelve views draw a single
line through 5-day bins of the whole record, and the semiannual signal the panel
exists to show is buried in the scatter of individual years. A band across the
years with the mean on it would show the thing; the current picture shows that
the thing is hard to see.

**Several frames use about half their height.** `closure`'s first two pairs are
the clearest: the requirement sits far above the achieved curve and most of the
frame is empty. Worth measuring rather than judging — the fraction of a frame's
y range that the drawn data actually occupies is computable, and `panel_check`
could report it the way it reports a failed render.

**Nothing has a log x axis.** `spec.y.log` exists and `spec.x.log` does not.
`design`'s crossings at 0.50, 0.74 and 2.62 years all sit in the left eighth of a
linear 0.5-to-15 axis; `predict`'s lead ladder is geometric and drawn linearly;
`segmentation`'s Ap histogram has a long tail it draws in full.

### 33.5 · The two standing risks, restated with the new count

**63 views, 14 reference images.** Every spec records one, shot at the state the
panel opens in. **Forty-nine views are covered by nothing automatic** — `design`'s
whole F10.7 branch, eight of `repeatability`'s nine, eleven of `climate`'s twelve.
`panel_check` already takes a per-row `state`; letting a spec declare several
views each with its own reference is a modest change and the only one here that
is about correctness rather than about reading.

**There is no dark mode anywhere.** `prefers-color-scheme` appears **zero** times
in `web/app.css`, and `chart.js` reads no CSS variable — it fills the canvas
`#fff` unconditionally. Earlier notes in this plan called this "dark mode has no
steps", which understated it: the face has no dark mode at all, so this is a
whole-face piece of work and not a chart-palette one.

### 33.6 · In order, if these are wanted

| | what | why it is first |
|---|---|---|
| 1 | **a grid of panes** | 27 views collapse into 4, and the comparison the knobs prevent becomes the picture. One feature, machinery already there |
| 2 | **a band kind** | zero of 63 use one, six want one, and in four of those the prose already promises it |
| 3 | **several references per spec** | 49 views with no automatic check is the layer's real risk |
| 4 | **log x** | three panels put their whole argument in the left eighth of a linear axis |
| 5 | **the two bad forms** — issue age, and by-day-of-year | each is one view drawing one fact badly |
| 6 | **a frame-fill warning in panel_check** | cheap, and it finds the half-empty frames without anybody judging |
| 7 | **dark mode** | the largest and the least epistemic. A whole-face job, and every reference re-shot |


---

## 34 · Making the picture carry the argument

§33 counted the figures and found redundancy. This asks a different question of
the same 63 views: **can a reader understand what a figure says by looking at
it?** Measured across the eleven panels at the state each opens in:

| panel | words of prose beneath | characters of text INSIDE the frame | marks |
|---|---|---|---|
| `forecast` | 354 | 64 | 2 |
| `pattern` | 281 | 37 | 3 |
| `predict` | 271 | **0** | **0** |
| `closure` | 240 | 78 | 3 |
| `thermosphere` | 219 | 73 | 2 |
| `density` | 179 | **0** | **0** |
| `drivers` | 141 | **0** | **0** |
| `repeatability` | 132 | **0** | **0** |
| `segmentation` | 116 | 28 | 2 |
| `design` | 68 | 102 | 3 |
| `climate` | 71 | 42 | 1 |

**Median: 179 words of argument, 37 characters of picture.** Four of the eleven
opening views carry no mark at all — nothing in the frame says anything, and the
entire finding is a paragraph underneath.

The panel already shows its QUESTION twice above the chart: `asks` in bold and
`draws` in grey. **What it never shows is the answer.** A reader meets a question,
a line, and then a paragraph, in that order, and the paragraph is where the work
is.

### 34.1 · Five moves that put the argument in the frame

**1 · The answer, as a number, where the question is.** Every panel declares a
question the engine can answer, and every one of those answers already exists:
`design` → *exceeded at 2.62 yr*; `closure` → *margin +0.4438*; `pattern` →
*26 days, r = 0.376*; `thermosphere` → *949.6 K*; `forecast` → *peak skill +0.44
at lead 9*. A stat line beside the question gives a reader the answer in half a
second, and the chart below then shows WHY rather than having to be decoded
first. This is the cheapest of the five and it touches all sixty-three views.

**2 · Shade the region that means something, instead of ruling a line at its
edge.** A dashed rule asks the reader to decide which side they are on. A pale
wash tells them. Where it applies, and what it would say:

| view | the region | what shading it says at a glance |
|---|---|---|
| `design`, Ap | above the design level | "outside what the vehicle was built for" |
| `forecast`, skill | below zero | "worse than not bothering" |
| `closure` | between required and achieved | that area IS the margin |
| `thermosphere`, slot | between the two Kp lines | that gap IS the open question |
| `segmentation` | quiet / active / storm | three regimes, not two dashed lines |
| `pattern` | inside the Bartlett band | "this wiggle is shape, not finding" |
| `predict` | between the percentiles | what a band buys and costs |

Seven of eleven panels, from one `kind: 'band'` and one region mark.

**3 · Annotate the feature, not the axis.** A mark labels a position on an axis. An
annotation points at a place ON A CURVE and says what happens there — and that is
the sentence the reader is currently getting from the paragraph:

- `design` — at the crossing: *"the design is exceeded here"*
- `predict` — at the hump: *"half a solar cycle"*; at the dip: *"a full one"*
- `pattern` — at each of the three peaks: the period it implies
- `drivers` — at the worst-day dot: *"the study applies one percentile at every level"*
- `climate` — at the 2017 gap: *"273 days the record does not have"*

Needs `spec.notes = [{x, y, text}]` with a short leader line. Five panels, and in
each case the text already exists in the note below.

**4 · One line is the answer; the rest are context, and should recede.** `predict`
does this — the 95th is heavy because it is the published percentile — and
nowhere else does. In `repeatability` the mean cycle and the three individual
cycles are the same weight, so the eye has no entry point and the reader has to
be told in prose which line the panel is about. The rule is mechanical: the row
the panel is named for gets full weight; context gets a thinner stroke and less
contrast. It costs nothing and it is what makes a five-curve frame readable.

**5 · Put the finding in the frame, in one line.** The `correct` blocks already
contain the sentence each figure exists to prove — *"731 sits above 365 at 182 of
the 200 lags"*, *"the dashed line must sit BELOW the solid one at short lead"*. A
one-line subtitle inside the frame, drawn from the data rather than typed, is the
difference between a chart a reader interprets and a chart that tells them what
they are looking at.

### 34.2 · Interaction, beyond the crosshair

B7 gave every figure a pointer and keyboard readout and B8 a table. What is still
missing is everything that lets a reader ASK something of the picture:

- **Brush to zoom on x.** `design`'s three crossings are at 0.50, 0.74 and 2.62
  years on an axis that runs to 15 — the whole argument lives in the left eighth
  and cannot be enlarged.
- **Click a legend entry to isolate or mute a series.** With five curves and two
  nearly coincident, muting three is how a reader separates the other two.
- **Click a curve or a mark to open the row that computes it.** The rows a panel
  argues about are already links ABOVE the chart; the picture itself is not
  navigable, so the connection between a line and the row behind it has to be
  made by reading.
- **Pin a view and overlay the next.** This is the comparison every control
  prevents, and it is the general form of what §33's grid does for four specific
  panels.
- **Copy a row, or the whole table.** The table exists; nothing gets a number out
  of it except selecting text.

### 34.3 · Visual polish, in order of how much it is worth

**The aspect is one shape for every figure.** 2.2:1 capped at 1180 wide — chosen
in B6 for slope readability, and right for the record panels. It is tall and empty
for a monotone curve, which is most of the set: `closure`'s first two pairs, the
nine `design` Ap views, `drivers`' five. A spec that can ask for a shorter frame
would fix the emptiest third of the sheet.

**The grid is drawn at every tick on both axes.** Halving it, or dropping the
vertical rules where the x axis is categorical, would take ink out of the picture
and put emphasis back into the data.

**The canvas fills `#fff` on a `#fcfcfb` page.** A one-step mismatch, visible as a
faint rectangle at every panel edge. Trivial, and it is the sort of thing that
makes a set of figures look assembled rather than designed.

**A fill under a single line, where zero is meaningful**, gives a lone curve
weight without claiming anything: `forecast`'s RMS error, `climate`'s yearly
means, `segmentation`'s counts.

**And dark mode, which does not exist** — §33.5. The largest of these and the
least urgent, because it changes how the figures look and not what they say.

### 34.4 · What this is worth, in order

| | move | reach | cost |
|---|---|---|---|
| 1 | the answer beside the question | all 63 views | small — the numbers exist |
| 2 | shade the region that means something | 7 of 11 panels | one `band` kind, one region mark |
| 3 | emphasis: the answer heavy, context thin | 5 of 11 | none — a width and an alpha |
| 4 | annotate the feature with a leader | 5 of 11 | `spec.notes`, and the words are already written |
| 5 | the finding as an in-frame subtitle | all 63 | small, once 1 exists |
| 6 | brush to zoom, click to isolate, click to open the row | all 63 | the largest piece of work here |
| 7 | a shorter aspect where the curve is monotone | ~20 views | small |
| 8 | grid, canvas fill, fills under single lines | all 63 | small |
| 9 | dark mode | the whole face | large |

Moves 1, 3 and 8 together are perhaps a day's work and they change every figure in
the set. Move 2 is the one that changes what the figures MEAN, because in seven
panels the region is the finding and only its edge is currently drawn.


## 35 · Moves 1 and 2, done — the answer beside the question, and the region shaded

§34 ranked nine moves. The first two are now in the face. This section records
what they turned out to be, the three defects they uncovered on the way, and the
one place the move was wrong and was taken back out.

### 35.1 · Move 1 — the answer beside the question

A build may now return `answer: { value, of }`. The face draws it between the
question the panel asks and the line describing what it draws: the number large
and tabular, the clause after it small and grey. It is optional by design — a
build with no single number must not invent one, and a headline that is a guess
is worse than no headline.

**Twenty-four views now carry one, which is every view in the solar subsystem.**
Not one of the numbers is typed: each is measured off the same arrays the figure
is drawn from, so an answer cannot drift from its own picture the way §21 found
three copied constants drifting from their rows.

Three of them are worth naming because they are not what a first pass would have
put there:

- `repeatability` answers **0.972 · how well cycle 24 repeats 23's shape — while
  its peak is 0.79 of 23's**. The correlation alone is the reading the panel
  exists to stop, so the ratio rides in the same line.
- `density` answers **no answer · nothing computes
  sys_space_environment_atmospheric_density**. Rule 5: a refusal is never a
  substitution. The correlation the panel draws is about something else, and
  putting it in this slot would answer the question with a number that is not an
  answer to it.
- `thermosphere`'s Kp-slot view answers **140.6 K · the most the choice of Kp
  slot is worth — and nothing says which slot to use**. That is §29.2's open
  question stated as a quantity at the top of the figure that raises it.

**An empty `<p>` is not a free `<p>`.** Adding the element moved three panels
this change does not touch — `predict` by a pixel of canvas height,
`repeatability` and `segmentation` by 5.3 and 2.5 per cent of their pixels —
because an extra block node shifts the layout the canvas is measured against by
a sub-pixel, and the canvas height is a function of its measured width. A direct
probe reported the canvas identical; `panel_check` reproduced the difference
twice. Giving the element the `caption` class its siblings already carry, and
zeroing its margin and padding when empty, makes the empty case identical to
having no node at all.

### 35.2 · Move 2 — shade the region that means something

Two additions to the chart: a series `kind: 'band'`, which fills between `y` and
a second edge `y0`, and a mark carrying `from`/`to` instead of `at`, which washes
a region of an axis. Both are drawn behind the data. A band is filled per
unbroken run, never closing across a gap — the area equivalent of the rule that a
line is never drawn across a null. Neither appears in the legend, the readout,
the table or the arrow-key ladder: they are furniture, and every value in them is
carried by a line that is already there.

Nine places took one:

| panel · view | what is shaded | what it says |
|---|---|---|
| `design` · Ap | above the design level | outside what the vehicle was built for |
| `design` · F10.7 | between the outer two curves | the design window, which the note already called it |
| `closure` · top | achieved to requirement, **split at the crossing** | margin left, then requirement exceeded |
| `closure` · margin | below zero | the requirement is not met |
| `forecast` · skill | below zero | worse than not bothering |
| `pattern` | inside the Bartlett band | wiggle rather than finding |
| `predict` | 50th to 99th | what choosing a percentile costs |
| `thermosphere` · flux | declared 150 to the subsystem's 104.1 | §28.2, drawn as a distance |
| `climate` · Kp/ap | 10th to 90th percentile | how wide a day is, at one worst slot |

The `closure` split is the one that changed what the picture MEANS. One fill drew
"this much margin is unspent" and "this much requirement is exceeded" as the same
wedge in the same ink — the single distinction the panel is named for. Two fills
meeting at the interpolated crossing, the unspent side in the achieved curve's
own hue and the exceeded side in the pink every bound in this face is drawn in,
say it without a word.

### 35.3 · Where the move was wrong, and was taken back out

**`segmentation`'s top band.** Shading the storm band is the obvious move and it
is wrong on that axis: storm is Ap 26 and above, the axis runs to 400, so the
band holding 2.3 per cent of the DAYS is 94 per cent of the WIDTH. It washed 69.5
per cent of the frame and told a reader the opposite of the share it was drawn to
show. The share is in the answer line instead, where it is a number and not an
area.

**`forecast`'s bias frame.** Same test, same result: the bias is negative at all
27 leads, so the wash covers the whole frame, and a frame that is entirely shaded
has said nothing. It also fails on meaning — on the skill frame zero is a verdict,
on the bias frame it is a direction, and an outlook that reads low is not a
failing outlook but a fact a design has to carry.

So the rule a region has to pass is not "is this side the bad side" but **"is
this a region"** — if the data lives inside the wash, there is no region, only a
tint.

### 35.4 · Three defects the moves uncovered

**A band shifted the palette.** A series with no colour takes the next hue in the
fixed order, and "next" was counting the bands. The design-window fill moved the
hot single-day curve onto the blue its sustained neighbour had already declared,
and two curves in one frame came out the same colour. Nothing failed; the picture
just started lying about which line was which. The hues of a pane are now
resolved once, in one function, skipping the bands — and a band with no colour of
its own borrows the hue of the series that follows it, which is what a band drawn
under one line wants anyway.

**An x region blanked a chart, silently, and `panel_check` caught it for the
wrong reason.** The x extent walked `m.at` only, so the first x region ever drawn
— §28.2's gap — put `undefined` through `Math.min` and made the extent NaN. Every
x position is then NaN, so the frame drew its axes, its ticks and not one mark of
data. It passed check 1, because the axis labels mean the mount is not blank. It
passed check 3 at 2.4 per cent, because three thin lines and some text ARE about
2.4 per cent of a white canvas. It failed 2b — *asked the engine for env_f107 and
drew the same picture when the answer changed* — for the one honest reason
available: a chart that draws nothing draws the same nothing whatever the engine
answers. **2b found a blanked chart that the reference comparison rated a 2.4 per
cent drift.** The y extent had always walked all three fields; the two loops did
the same job and only one of them knew about regions.

A non-finite extent now throws rather than drawing an empty frame, on both axes.
The failure mode it had was the worst kind: a picture that looks like a panel with
no data rather than like a bug.

**A wash started at one line and was coloured like the other.** `design`'s region
opened at the capability mark and was drawn in the pink of the requirement mark
three inches above it. A region belongs to the edge it opens at, and this is the
second time in this face that picking an ink without asking what else is wearing
it has produced a mark that invites the wrong pairing.

### 35.5 · What is still open

- **Six references are UNCONFIRMED and one more just joined them.** `design` was
  signed; its content has changed, so its signature is withdrawn rather than
  carried forward. A machine may not sign for a picture it drew.
- Moves 3 to 9 of §34 are untouched.
- The `sw_outlook_lead` citation in `forecast` is still stale — reported in Part
  D, deliberately not fixed, and it is a row question rather than a figure one.


## 36 · Moves 3 and 8 — emphasis, and the three small things

§34.4 ranked emphasis third and the visual-polish bundle eighth, on the grounds
that together they cost almost nothing and touch every figure. Both are now in.
One of the three small things turned out to be wrong in its premise, and finding
that out was worth more than the change would have been.

### 36.1 · Move 3 — one line is the answer, the rest recede

A series may now carry `context: true`. The chart draws it at half contrast and,
where it did not ask for a width, thinner; its legend swatch and its end label
recede with it, because a curve drawn at half weight with its number at full
strength puts the emphasis back where the width just took it from.

**It is a flag and not an alpha per panel**, and that is the whole point: *how
far should context recede* is one decision for the face, made in one place, not
eleven decisions that start the same and drift.

Ten series across seven panels are context:

| panel · view | receded | why it is not the answer |
|---|---|---|
| `repeatability` · stack | the five individual cycles | the mean is the row the panel is named for |
| `pattern` · recurrence | 181 d, 731 d | 365 is the published window, and what the band and all three peaks are computed on |
| `forecast` · by lead and by year | the leaky baseline | it shows what the leak is worth; the strict one is the score |
| `design` · F10.7 | `sw_window_peak_level` | the note has always said it is NOT a design value |
| `thermosphere` · shape | the straight reference line | it is what the curve is measured against |
| `segmentation` · phase | the quiet share | the view is called "where a storm is likely" |
| `climate` · Kp/ap | the 10th and 90th edges | they bound the shaded spread |

`repeatability` is the one §34 named, and it is the clearest: five curves at one
weight, no entry point, and a reader told in prose which line the panel is about.
The rule kept is that context must stay legible — "cycle 23 peaks above cycle
24" is still checked by looking, not by reading.

### 36.2 · Move 8, part one — the categorical axes lost their vertical rules

`spec.x.grid = false` suppresses the vertical rules. It is set on the two axes
that are five named scenarios — `drivers` and `thermosphere`'s slot view — where
a gridline is an invitation to read a value off the axis at a place no value
exists.

The first form of this kept the zero rule, on the reasoning that which side of
zero a point sits on is a fact. True of a signed quantity — and a signed quantity
is never the axis that asks for this. On `drivers` the axis runs -0.3 to 4.3, so
the kept rule landed on the first scenario and was the **only** vertical line in
the frame, drawn darker than a gridline: a mark, at a place meaning "index 0".
`grid: false` now means no vertical rule at all.

### 36.3 · Move 8, part two — the fill under a line, where the area is the quantity

A series may carry `fill: true`, filling between the line and **zero**, and it is
**refused where zero is off the frame**: a fill running to the floor of an axis
that starts at 9 would draw an area nobody measured. That refusal is what makes
the feature safe to offer at all.

Two places earned it, and only two:

- `thermosphere` · shape. The quantity is *the temperature the geomagnetic term
  adds*, measured from its own value at Kp 0. The area under the curve IS the
  quantity, and the exponential's departure from the straight reference is now a
  wedge rather than a vertical distance to estimate.
- `forecast` · RMS error, in both views — and this one needed the axis moved to
  zero first. RMS error is a magnitude measured from perfect, not a score against
  a baseline, so an axis starting at 9 was drawing the VARIATION in the error and
  labelling it the error. With zero on the frame the picture says the error
  roughly triples over the first five leads and then flattens, which is the claim
  the note makes.

Everywhere else the candidates were bars, which already have area.

### 36.4 · Move 8, part three — §34 had the canvas backwards

§34.3 said: *"The canvas fills `#fff` on a `#fcfcfb` page. A one-step mismatch,
visible as a faint rectangle at every panel edge."* I changed the fill to
`INK.surface` and then measured the page, which is the order those two should
have gone in.

**The canvas is not on a `#fcfcfb` page.** `app.css` gives `canvas.plot`
`background: var(--card)` — `#ffffff` — inside a 1px rule, on `#fbfaf7` paper.
The figure is deliberately a white card. Filling it `#fcfcfb` would have put the
one-step seam INSIDE the border instead of removing it.

The real inconsistency was the other way round and is worth more than the one
reported: **`INK.surface` was `#fcfcfb` while the surface is white.** That token
is not decoration — it draws the 2px separator ring that lets two dots overlap
and stay two dots, and the 2px gap between touching bars. Both were being drawn
three levels off the colour underneath them. `INK.surface` is now `#ffffff`, the
fill takes the token instead of a literal, and both palettes were re-validated
against the corrected surface: the six categorical hues pass all five checks, and
`predict`'s four-step ramp passes with its light end at 2.11:1 against a floor of
2.

**Halving the grid was not done.** The horizontal rules are how a value is read
off a y axis, and dropping every other one trades a real reading for less ink. It
is a preference, not a defect, and it is left as one.

### 36.5 · What the byte comparison caught that the check did not

`panel_check` compares pixels with a per-channel slack of 8 levels and a 2 per
cent tolerance, which is right for a chart and means a small true change can pass.
After these moves only `forecast` exceeded it. So each panel's fresh shot was
compared to its stored reference **byte for byte** as well, which said
`repeatability` was *identical* — and it should not have been.

The `context` flag had never reached it. The edit was in a script that failed its
NEXT replacement and exited before writing, so a change I had seen in a rendered
picture was one I had reasoned into a picture that never had it. What the eye
confirmed was the pre-existing weight difference, 2.4px black against 1.4px
colour, which is exactly what the move was supposed to improve on.

Two lessons, both cheap: a multi-edit script must write what succeeded or nothing
at all and say which, and **"the reference is unchanged" is a finding when a
change was expected**, not a pass.

### 36.6 · Still open

- `repeatability`'s signature is withdrawn — its content changed. Seven panel
  references are now UNCONFIRMED.
- Moves 4, 5, 6, 7 and 9 of §34 are untouched. 6 is the large one; 7 (a shorter
  aspect where the curve is monotone) is the next cheap one.
