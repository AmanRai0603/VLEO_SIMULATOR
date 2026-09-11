# areas/faces.md

The command line, the daemon, the WebAssembly binding, the C ABI and the Python
wheel — the doors onto one kernel.

Applies to `crates/vleo-cli/**`, `crates/vleo-daemon/**`, `crates/vleo-wasm/**`,
`crates/vleo-ffi/**`, `crates/vleo-py/**`.
Agent I works here.

## The one rule about what crosses

**Everything crossing the boundary is SI.** A face converts for display and
never for transport. A kilometre on the wire is how two faces come to disagree
about the same number while both look right.

## A face computes nothing

If a face works something out, that calculation exists in only one face. Push
it into the kernel, where every face gets it and the gate can see it.

This includes the tempting small ones: a margin, a percentage, a unit label
chosen by a threshold. `vleo_bus::present` formats; it does not decide.

## Faces agree, or one of them is wrong

Golden vectors run across every face. They can only agree if the arithmetic
happens in one place and the maths is portable — which is why `pmath` exists
and why the kernel crates are `no_std`.

## Refusals cross the boundary intact

A face may not turn a refusal into an absence. `NotRun` arrives with the node's
name and leaves with it. A blocked count of zero is printed as zero, and a
non-zero one is printed with every name behind it. A face that quietly shows
the rows that worked is a face that reports a design as closed when it is not.

## The daemon

Local, on 7777 by default, one process. It holds no state a run depends on: a
run either has verified data on disk or refuses to start. It never fetches
during an evaluation.

A stale daemon serving an old tree is the failure people actually hit. The
sheet hash on a page refuses a mismatched fragment; a stale *process* has its
own consistent copy of everything, so check the port before believing the page.
