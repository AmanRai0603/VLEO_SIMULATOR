# Approvals

> **Answer first.** One file per form branch: its node engineer's approval of the exact preview build they tried, recorded by `cargo run -p xtask -- approve <file>`. A form branch merges only with one that matches what it holds; the pipeline checks. Never written by hand.
>
> **Kind:** reference · **For:** the developer, until the switch-over

`approvals/<author>--<node>.toml` is the file the preview saves when its node engineer
presses **Approve this preview…** — the branch, the commit and the build it was
given for, their name, and that they ran the changed nodes. `xtask approve`
refuses it unless it is for the build of exactly what the branch holds, stores
it here and commits it naming the node engineer (`Tested-by:`). The pipeline's check
*the author approved this exact change* reruns the same test on the pull
request, so anything pushed after an approval needs a new preview and a new
approval.

The files stay after the merge: they are the record of who tried which change.
