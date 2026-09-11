# Recipes — rung two of the escalation ladder

A recipe is a fix that has been needed twice, written down so it is never
invented a third time.

The ladder is three rungs, and each fires only when the one below could not:

| rung | mechanism | lives in |
|---|---|---|
| 1 | a bot opens the pull request | `.github/dependabot.yml` |
| 2 | a transform solved once, replayed | this folder |
| 3 | an agent, only when the first two could not | F diagnoses, I fixes |

## The form

`tools/recipes/<name>.py`, with three names:

```python
WHY = "one line: the fix this replays, and where it came from"

def apply(root):
    """Make the change. Return how many files moved — 0 when already applied."""

def verify(root):
    """Raise if the tree is not in the state apply() leaves it in."""
```

A leading underscore means a shared helper rather than a recipe.

## The one rule it is checked against

`tools/recipe.py --check` copies the tree, runs `apply` twice, and requires the
second run to change nothing. A transform that is not idempotent cannot be
replayed safely, and replaying is the entire point of the rung.

## Why this folder is empty

Because the ratchet runs one way. Every messy fix becomes a recipe, so rung-three
work migrates down to rung two over time — and a folder seeded with imagined
transforms runs that backwards, filling the rung with recipes nobody needed and
teaching everybody that the folder is decoration.

When a fix is needed the second time, that is the moment. Not before.
