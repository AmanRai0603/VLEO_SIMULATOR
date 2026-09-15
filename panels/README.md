# Panels — a picture gets a spec, like a node

A wrong number crashes a test. A wrong chart looks beautiful.

The measured record of this codebase says the visual layer is where the defects
actually live:

- a shaded region on the wrong side of a boundary
- an all-violated panel left white, reading as "everything is fine"
- three correct power numbers on one screen, correct at three different times
- a label variable overwritten by a coordinate and printed under the plot

Every one of those rendered perfectly and passed whatever checks existed. The
engine has generated verification; the visual layer had a prohibition and a
reviewer. That asymmetry is backwards relative to the evidence, and the fix is
the one that already worked everywhere else: declare it, generate it, check it.

## The spec

One `panels/<id>.toml` per panel:

```toml
id = "tree"
label = "The layer tree"
draws = "every row at the selected layer, as nested boxes, seeded ones hatched"
module = "web/js/tree.js"
mount = "#tree"
reads = ["layer", "selected"]
correct = "one box per row, nested by parent, in document order, none overlapping"
```

`mount` is the element the panel owns. `reads` names the state it claims to
depend on — and that claim is what check two tests.

## The three checks

`tools/panel_check.py` drives the real page in a real browser against the real
daemon. None of the three needs a person.

**1 · It renders.** After a draw, the mount is not empty and not zero-sized. An
all-violated panel left white is the defect this catches, and it is the one
people's eyes repair automatically.

**2 · It moves.** Change each input the panel declares it reads; the mount's
content must change. A panel wired to nothing passes every other check —
including a screenshot comparison, because it draws the same correct picture
every time regardless of the data.

**3 · It matches.** Against a stored reference in `panels/reference/`, within
`tolerance`. Recorded with `--record` by a person who has looked at the picture
and agrees with it; a reference nobody looked at is a snapshot of a bug.

`confirmed_by = "Name / date"` on the spec is who that person was. Nothing
enforces it — the check cannot tell a reference somebody studied from one a
machine drew unattended, which is the whole reason the name has to be written
down by hand. The eight solar panels carry it. The three older specs predate
the field and it has deliberately not been backfilled, because nobody now
remembers who looked and a guessed attestation is worse than none.

## A canvas needs different eyes

The first three panels all build DOM, so both "is it empty" and "did it change"
read `innerHTML`. A `<canvas>` has empty `innerHTML` by specification and never
changes, so for as long as that was the only test, **the one real chart in this
repository could not be checked at all** — declaring it reported
`canvas.sw-plot is still empty after the page settled` and stopped there. Three
structural diagrams were checked; the plot was not, and every node whose
`[view]` is a line is drawn by it.

A canvas mount is now compared on its rendered pixels — `toDataURL()` — for both
checks. "Empty" means equal to a fresh canvas of the same size, which is exact
and needs no threshold. The selftest carries both failures on a canvas panel as
well as on a DOM one, because the DOM pair passed happily while the gap was
open.

Two smaller changes came with it:

`ready` — JavaScript run after the page loads and **before** the three checks,
for a panel that does not draw until somebody asks. The sweep is the example:
its canvas is created hidden and blank, so checking it at the state the page
opens in would report a defect that is the panel working as designed. `ready` is
not `reference_state`: that one runs just before the screenshot, this one before
anything is checked.

`settle_ms` — check two now waits for the content to *change*, up to this many
milliseconds, instead of sleeping a fixed 250 ms and comparing. A fixed pause
has to be long enough for the slowest panel, and a panel that redraws behind a
fetch is slower than any pause anyone would write. Waiting on the condition is
faster when it is quick and correct when it is not; a panel wired to nothing
still fails, it just fails after the timeout rather than before it.

## Why `correct` is prose

Checks one and two are mechanical and catch the mechanical failures. `correct`
is for the reviewer, and it is the field that makes the visual review a
comparison rather than an impression: "one box per row, none overlapping" is
something a person can disagree with. "Looks right" is not.
