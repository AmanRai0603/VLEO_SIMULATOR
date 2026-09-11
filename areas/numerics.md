# areas/numerics.md

Precision, portable maths, profiling, and where a second implementation may
differ from the first.

Applies to `crates/vleo-units/src/pmath.rs`, `crates/vleo-core/**`.
Agent I works here, with this file.

## The rule, and why it is not negotiable

Addition, subtraction, multiplication, division and square root are exactly
specified by the floating-point standard and agree on every target. Sine,
cosine, exponential, logarithm and power are not: a native build calls the
operating system's maths library, a WebAssembly build calls the one compiled
into the binary, and the two differ in the last bit.

"Golden vectors agree across every face, bit for bit" is gated nightly. Without
one portable maths library, every node with a trigonometric or exponential term
— which is most of the orbital ones — fails that gate on the first night for a
reason that is not a defect. Within a fortnight the team learns to ignore a red
nightly build, which is strictly worse than having no gate.

## How it is enforced, in two places

**In the kernel crates**, by the compiler. `vleo-units` and `vleo-core` are
`no_std`, so `f64::cos` does not exist there:

    error[E0599]: no method named `cos` found for type `f64` in the current scope

That is the strongest available enforcement and it costs nothing to keep. Do
not add `std` to a kernel crate to make an error go away — Cargo unifies
features across a workspace, so one face enabling `std` would hand it to the
kernel. That is why the no-std job builds a constrained target and never
`--workspace`.

**In hole bodies**, by refusal. The node crates are not `no_std`, so a hole
*can* compile `.sin()`. `cargo xtask fill` refuses the call by name before it
reaches the file, and the gate's `portable-maths` check refuses it again on the
committed file.

## Precision

`pmath` is a port, not a wrapper. Its job is the same answer everywhere, not
the fastest answer. Where a routine is measurably slow enough to matter,
measure it first and say by how much — a faster routine that changes the last
bit costs the cross-face gate, which is worth more than the microseconds.

Never compare floats with `==`. `clippy::float_cmp` is denied in `vleo-core`
for this reason; if a comparison is genuinely exact, say why in a comment beside
the allow.

## The shader twin

If a second implementation of a relation exists for speed, it is a twin, not a
replacement. The kernel's answer is the answer. A twin may differ only within a
stated tolerance, that tolerance is declared, and the two are compared over the
declared domain — not spot-checked.
