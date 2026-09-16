#!/usr/bin/env bash
# The environment nobody configures.
#
# No engineer's own machine enters the picture. Everything a node needs is
# pinned here and in rust-toolchain.toml, so day seven — a second engineer
# building the second node from the runbook alone — fails on a template defect
# rather than on an environment difference.
set -euo pipefail

echo "pinning the toolchain from rust-toolchain.toml"
rustup show active-toolchain
rustup component add rustfmt clippy
rustup target add thumbv7em-none-eabihf || true

# The store lands under $HOME, OUTSIDE the checkout. It used to be pointed at
# ${containerWorkspaceFolder}/.vleo/data, which put a generated directory in
# the working tree: every Codespace opened with ten untracked files that were
# a byte copy of bundles/ and looked like work somebody had forgotten to
# commit. A laptop never had that, because VLEO_DATA is unset there.
echo "filling the local reference-data store (under \$HOME, not in the checkout)"
cargo run -q -p vleo-cli --bin vleo -- data sync || true

echo "generating the per-node artefacts and checking they match what is committed"
cargo run -q -p xtask -- docs
cargo run -q -p xtask -- variables
git diff --quiet || echo "note: the committed artefacts differ from the sheets — run 'cargo run -p xtask -- docs' and commit"

# Built here, once, so that attaching to the Codespace starts the tool in a
# second instead of blocking you behind a release build. start.sh runs on every
# attach and only launches what this step produced.
echo "building the tool so it is ready to serve"
cargo build --release -p vleo-daemon

echo
echo "the tool starts itself on 7777 when you attach — the preview tab is it."
echo "to run it by hand instead:"
echo "    cargo run --release -p vleo-daemon"
echo
echo "one command that must be green:"
echo "    cargo run -p xtask -- gate && cargo test"
