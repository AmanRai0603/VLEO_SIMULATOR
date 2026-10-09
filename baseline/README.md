# Today's answers

> **Answer first.** `today.csv` is what this engine answers today, for every row of the design in every shared case, with the reference data and without it, at both ends of every input's range, and on every fixture. It was recorded before the engine changes for 1.0, so the new engine can be held to it. `cargo test` fails the moment an answer moves.
>
> **Kind:** reference · **For:** developers

## Why it exists

Phases D and E of `docs/PLAN_1_0.md` moved the graph out of the compiled code
and into the design's files. Their parity gate asks whether the new engine gives, for every
row in every case, the answer this one gives, and refuses where this one
refuses, for the same reason. That has no answer unless this engine's answers
were written down first. This is that record.

## What it holds

One line per answer, as CSV: `run,what,id,value,status,detail`.

| what | means |
|---|---|
| `value` | a row's value, as the shortest text that reads back to the same number; its status; its credibility, eight digits |
| `blocked` | a row that did not answer: its kind of refusal, and the message |
| `counted` | how many rows ran and were blocked, and how many sweeps the loop took |
| `absent` | a row that answers at the declared values and not at this range end |
| `fixture` | a fixture's value as the engine computes it today, and whether it passed |

- **The whole design** is recorded in full for each case: the design as declared,
  and each example in `cases/examples`, with the reference data and without it.
- **A range end** is recorded by what it changes from the declared design.
  Anything a new engine moves there still shows, as a line that was not here.

## When it fails

The test is `crates/vleo-cli/tests/today_s_answers_are_on_record.rs`. It names the
first lines that differ.

- **An engine change that moves an answer is a defect.** Fix the change.
- **A deliberate change to the design** — a release taken in, a relation
  corrected — moves answers on purpose. Record them again, in the same commit
  as the change, so the difference is reviewed with it:

      VLEO_BASELINE=write cargo test -p vleo-cli --test today_s_answers_are_on_record

The record is never written again to get a build green.

## Where it breaks

- **Only the cases that exist are recorded.** One case and two examples today;
  a case added later is recorded when it is added.
- **A range end moves one input at a time.** Two inputs at their ends together
  are not recorded.
- **A range is recorded only at its ends.** The points between them are not
  recorded.

## The other records

The compiled engine left the repository in phase E. Two more records are
written with today's answers, by the same command, and the design read from
its files is held to them too:

| file | holds | test |
|---|---|---|
| `graph.txt` | every node, variable, case and cycle of the graph, and its fingerprint | `the_graph_read_at_run_time_is_the_graph_on_record` |
| `methods.csv` | every method's answer at each of its fixtures, and with each input not a number, infinite, far outside every range, zero and negative | `every_method_answers_and_refuses_as_on_record` |

Both were first written while the compiled engine and the translated methods
existed, and were those exactly. A deliberate change to the design records all
three again:

    VLEO_BASELINE=write cargo test -p vleo-cli --test today_s_answers_are_on_record

`transcribed.csv` holds what each relation answered while it was still code,
before it became a method (`the_transcribed_relations_answer_as_the_code_they_replaced`).
It was written once, while that code existed, and is never written again.
