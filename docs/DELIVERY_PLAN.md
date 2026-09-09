# Delivery

One kernel, two artefacts, three doors.

    THE URL         the layered document and a demonstration kernel in WebAssembly
    THE INSTALLER   the same interface assets, the native engine, the shipped bundles

The installer opens the one kernel through three doors:

| door | is | opened by |
|---|---|---|
| the daemon | a local process serving the interface at `127.0.0.1` | a browser — the same interface as the URL, backed by a real engine |
| the library | `vleo.dll` / `.so` / `.dylib`, bound by the Python wheel | Python directly, and MATLAB through the wheel |
| the tool | `vleo run · sweep · campaign · data · selftest` | batch work, CI, anything scripted |

The installer is not a second codebase: one library crate with three thin
wrappers over it. The same `evaluate()`, the same evidence, the same
credibility, in all of them.

## Why an installer rather than everything in a browser

Both were considered. Compared on eight axes the installer wins six, and it
removes two entire classes of problem.

| axis | everything in a browser | URL plus installer | |
|---|---|---|---|
| speed on real work | WebAssembly is meaningfully slower on numerical loops | native wherever the work is real | **installer** |
| memory ceiling | a 32-bit address space caps each instance at 4 GB | the machine | **installer** |
| the customer's own hardware | unreachable | they install on it | **installer** |
| MATLAB and Python | a parallel path beside the browser path | one engine; the wheel binds the same library | **installer** |
| protecting the engine | WebAssembly decompiles to readable pseudo-code | a stripped, link-time-optimised binary is far more expensive to read | **installer** |
| data | every run needs the network, or a large download into a cache | a real local store, verified, pinned, offline | **installer** |
| reach — open it and use it now | a URL and nothing else | an install, which some people cannot do | **browser** |
| support surface | one environment | three operating systems, antivirus, firewalls | **browser** |

Two problems the installer deletes outright: heavy runs in a browser tab, and
cross-origin isolation. Because the public WebAssembly stays small and
single-threaded, the site never needs special response headers and any static
host will serve it.

## The one hard part

A page on the internet speaking to a program on the machine is the most likely
thing to break the whole model, and it is not obvious.

| rule | effect on a public page calling a local daemon |
|---|---|
| cross-origin resource sharing | the daemon must send the right headers and handle preflight |
| mixed content | loopback is treated as trustworthy — but that is a policy, not a guarantee |
| private network access | a public page reaching a local address needs an extra preflight, and the rules are tightening |

The third is the one most likely to break later, in a browser update, with no
change on this side.

**The boundary is removed rather than negotiated.** The daemon serves the
interface itself, so the page and the engine share one origin and none of the
three rules applies. It also works with networking disabled entirely.

Loopback by default. Exposing the daemon to a network is an explicit, separate
act, which also avoids a firewall prompt on first run.

## Reference data: a package manager, not a query interface

Treating reference and operational data as one problem is what makes the data
question look hard. Separated, each has an obvious answer.

|  | reference data | operational data |
|---|---|---|
| is | solar drivers, coefficients, fitted biases, validated cases | identity, entitlement, the run ledger |
| changes | slowly, by publishing a new version | constantly |
| needed | by every single evaluation | occasionally, never during one |
| lives in | immutable versioned bundles, synced to a local store | a database behind an interface |
| network down | everything works on what is local | history pauses; nothing else |

**There is no database server on the physics path.** A campaign of a million
cases makes zero network calls, and a run in an air-gapped facility is the same
run made anywhere else.

    vleo data sync      reconcile the store against configuration; write the lockfile
    vleo data           what is present, which version, verified or not
    vleo data verify    re-check every hash

Properties that are not optional:

- **Immutable and versioned.** A published version is never edited; a correction
  is a new version. That is what lets a result from eighteen months ago be
  reproduced exactly.
- **Content-addressed.** Checked before use. A tampered coefficient file
  silently changing every result is the one failure the credibility system
  cannot detect on its own.
- **A lockfile.** Without it, *one kernel everywhere* is true and *the same
  numbers everywhere* still is not.
- **Versions travel in the run manifest.** This is what connects the data design
  to the credibility argument rather than leaving it as plumbing.

| data state | pedigree factor | shown as |
|---|---|---|
| pinned, verified | full | name and version in the provenance bar |
| current, verified, not pinned | full, with a note that the result may move | version and date |
| verified but stale beyond its stated age | reduced | amber, naming the age |
| shipped climatology standing in for measured drivers | reduced | amber — climatology fallback |
| hash does not verify | **zero — the run refuses** | a fault naming the bundle |
| no bundle for a required input | **zero — the run refuses** | a fault naming what to sync |

The last two are refusals rather than warnings. A result computed from
unverifiable data is not a degraded result; it is not a result.

## Openness

Full openness where the work happens; a closed surface where it ships. The
mechanism is a build profile, and the honest accounting includes what it does
not achieve.

    --profile dev    unstripped · debug info · the whole node tree · the internal mirror
    --profile user   stripped · lto fat · panic abort · one codegen unit
                     the node tree pruned at a declared depth
                     licensed bundles fetched, never embedded

**Both profiles build on every merge.** A customer build that fails only at
release time fails on the day it can least be afforded — and link-time
optimisation with a single codegen unit changes inlining, so the profile that
ships is the profile that must be proven.

The tester exercises the user profile first, deliberately: testing the
engineering build tests something no customer will ever run.

### What a closed binary actually buys

A stripped, link-time-optimised binary with inlined generic code bears little
resemblance to its source, and recovering a *trustworthy* physics kernel from
it costs more than writing one. That is a real commercial barrier and it is
worth having.

What it does **not** do is make the artefact secret. A plan that treats it as
secret has no position at all if it is read.

| defence | holds because | strength |
|---|---|---|
| the run ledger | accumulated runs, verdicts, regressions and provenance are in no binary | **strongest** |
| the data bundles | fitted biases and validated cases are entitled at sync, never embedded | strong |
| the restricted nodes | never present in any shipped artefact | absolute, and narrow |
| the stripped binary | expensive to read | a barrier, not a wall |
| the evidence and the method | published deliberately | not a leak — it is the argument |

## Decisions that are cheap now and expensive later

| | decision | cost now | cost deferred |
|---|---|---|---|
| D1 | name the two to four restricted nodes | a list of identifiers | the commercial position rests on obfuscation a determined reader defeats |
| D2 | the demonstration subset the public kernel carries | one build flag and a list | either the whole engine ships readable, or the URL is a brochure |
| D3 | code-signing certificates, and notarisation | a purchase and a pipeline secret | the first external user meets a security warning instead of the tool |
| D4 | redistribution terms for every third-party dataset | an afternoon reading licences | a licence problem discovered by the licensor |
| D5 | MATLAB licensing and the exact release | a procurement question | the machine that exists to test the thing cannot run it |
| D6 | monthly running budget | a spreadsheet | a bill |
| D7 | bundle boundaries | a list drawn before the first publication | splitting later is a migration on every installed machine |
| D8 | signing key custody | the same conversation as D3 | a compromised key means re-signing everything |
| D9 | licence expiry policy | a field in the manifest schema | a warning everyone ignores, or a customer cut off mid-study |

## Deliberately not building yet

Each carries a revisit trigger, which is the difference between deferring and
forgetting.

| not building | because | revisit when |
|---|---|---|
| a hosted registry service | a bucket is a valid registry; the service adds entitlement and rate limits, and neither is needed internally | the first customer outside the building |
| a hosted run queue | customers use their own servers | a customer without a server asks |
| Parquet bundles | a format anybody can read in an editor is worth more until size forces it | the first bundle that does not fit comfortably as text |
| a windowed desktop shell | the daemon plus a browser already gives native speed with no second interface toolkit | only if a single-window experience is genuinely demanded |
| the rig console | that is the hardware-in-the-loop rung; the design tool comes first | loop testing starts |
| three-dimensional surfaces | the two-dimensional field is the picture that changes decisions | after that field is in daily use |
| automatic evidence capture | automatic capture records noise as evidence | explicit capture first |
