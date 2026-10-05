# Acceptances

> **Answer first.** One file per group release: its lead's acceptance of the exact test application they tried, recorded by `cargo run -p xtask -- group-accept <file>`. A group branch merges only with one that matches what it holds; the pipeline checks. Never written by hand.
>
> **Kind:** reference · **For:** maintainers

`acceptances/<group>-<version>.toml` is the answer the group application writes
when the lead opens a test application's `DELIVERY.toml` beside their sealed
release and answers **Accepted**: the release, its seal, the commit and the
delivery record it was given for, their name, and what they tried.
`xtask group-accept` refuses it unless it is for the build of exactly what the
branch `group/<group>-<version>` holds, stores it here and commits it naming the
lead (`Tested-by:`). The pipeline's check *the author approved this exact
change* reruns the same test on the pull request, so anything pushed after an
acceptance but this record needs a new delivery and a new answer.

The files stay after the merge: they are the record of which group accepted
which build.
