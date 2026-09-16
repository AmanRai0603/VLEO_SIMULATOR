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
