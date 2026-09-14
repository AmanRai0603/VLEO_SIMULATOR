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

**A forecast lead — days.** `sw_forecast_skill`, `sw_forecast_bias` and
`sw_band_coverage` are all functions of how far ahead the forecast looks, which
is 1 to 27 days. The only Time row a solar node can reach is
`orbit_mission_duration`, declared 0.5 to 15 years, so feeding it would clamp
every answer to the 27-day end of the table and return one number forever. There
is no row for a lead and inventing one needs a confirmed value.

What is NOT blocked, and was written instead: every row whose input is an
existing declared value or another written row. That turned out to be ten of the
twenty-six plus the interface.

### The issued 27-day outlook, measured — two findings worth a row each

`forecast_issued.csv` was verified against the observed record even though the
two rows that would publish it are blocked, because the findings stand on their
own and the next person should not have to rediscover them.

**The outlook is biased LOW at every lead**, by 0.6 sfu at one day worsening to
6.0 sfu at twenty-seven. For a drag design that is the unsafe direction:
under-predicted flux gives under-predicted density and under-sized drag.

**It loses to persistence for the first four days.** Skill against a persistence
baseline, as 1 - RMSE_forecast/RMSE_persistence, is -0.52 at a one-day lead —
the bulletin's RMSE is 10.33 sfu against persistence's 6.79. It becomes useful at
lead 5, peaks at +0.24 around leads 13 to 14, and goes negative again from lead
24. So the published outlook is worth having over a window in the middle and is
worse than yesterday's number at the start of it.

**And `lead_days` in that table is not all forecast.** Every issue carries
exactly 27 rows and the lead is `target_date - issue_date`, so 742 of the 1,285
issues contain leads of zero or negative — the bulletin was published after its
own window had begun, and those rows are a record of days already past. 109 rows
are negative, 729 are zero, and 39 sit between 60 and 392 days, which one issue
(prf 1157) accounts for and which cannot be a 27-day outlook at all. Any use of
this table must filter to leads 1 to 27; the measurements above do.

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
