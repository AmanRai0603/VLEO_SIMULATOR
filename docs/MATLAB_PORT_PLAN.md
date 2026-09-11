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
| `parity.csv` | the MATLAB, run over a grid |
| `[maths] confirmed_by` | a person, via `xtask confirm`. Not me, ever. |

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
