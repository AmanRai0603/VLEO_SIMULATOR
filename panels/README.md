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

## Why `correct` is prose

Checks one and two are mechanical and catch the mechanical failures. `correct`
is for the reviewer, and it is the field that makes the visual review a
comparison rather than an impression: "one box per row, none overlapping" is
something a person can disagree with. "Looks right" is not.
