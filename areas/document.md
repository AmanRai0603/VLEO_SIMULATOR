# areas/document.md

The layered interactive document, the browser face, the panels and the figures.

Applies to `web/**`, `docs/img/**`.
Agent J works here. It may never touch `crates/**`, `xtask/**` or `tools/**`.

## The document is the GUI

Not attached to it — it *is* it. A reader following the tree from mission level
down to one node can run that node at the bottom of the page, because the
contract that generated the page generated the binding.

## Design tokens

There is a fixed set. Using one is free; inventing one is prohibited.

A new token is a decision about how the whole tool looks, so it is a reviewed
change to the token list, not a hex value typed into a panel. A colour that
appears once is a colour that will appear twice next month and three times
after that, and then the tool has no palette.

## The figure contract

A figure that nobody has rendered is not a figure. Before a plot component is
shipped: render it, look at it, and say in the pull request that you did.

The defects this codebase has actually had are all of one kind — a picture that
renders perfectly and says something false:

- a shaded region on the wrong side of a boundary
- an all-violated panel left white, reading as "everything is fine"
- three correct power numbers on one screen, correct at three different times
- a label variable overwritten by a coordinate and printed under the plot

None of those crashes. None fails a unit test. A wrong number crashes a test; a
wrong chart looks beautiful.

## Three checks, all mechanical

A panel is declared in `panels/<id>.toml` and checked by
`tools/panel_check.py`, which drives the real page in a real browser against
the real daemon. None of the three needs a person:

1. **It renders.** The mount is not empty, occupies space, and the page threw
   nothing. An all-violated panel left white is what this catches.
2. **It moves.** Change each input the panel declares it `reads`; the mount's
   content must change. A panel wired to nothing passes every other check —
   including the screenshot, because it draws the same correct picture whatever
   the data says.
3. **It matches.** Against `panels/reference/<id>.png`, by pixels rather than
   compressed bytes, within the spec's `tolerance`. Recorded with `--record` by
   somebody who looked at the picture and agreed with it.

Two things learned recording the first three references, both worth knowing
before adding a panel:

- **Take the reference where the panel has something to be wrong about.** The
  path ribbon draws no arcs with nothing selected — correctly — so a reference
  taken there would still match if every arc were dropped. `reference_state`
  names the state to reach first.
- **Give the panel a viewport it fits inside.** An element screenshot larger
  than the viewport is stitched, and a transparent pane then composites
  whatever the page had scrolled behind it. That put the tree's row labels
  inside the matrix and made its diagonal look as though it started at row 9.

## The depth filter

The tree reads in the order the source wrote it, not alphabetically. `order` is
a field on every row and the display sorts by it. `Achievable lifetime` before
`Specific impulse` reverses what the source said, and the order is part of what
the source said.

## One module per concern

`web/js/` is split by what a file is about — dom, state, display, tree, paths,
matrix, figure, node, run, architecture, app. A change to how the matrix is
drawn touches `matrix.js` and nothing else. If it touches four files, the split
is wrong and that is worth fixing before the change.
