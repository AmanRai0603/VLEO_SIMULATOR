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

echo "filling the local reference-data store"
cargo run -q -p vleo-cli --bin vleo -- data sync || true

echo "generating the per-node artefacts and checking they match what is committed"
cargo run -q -p xtask -- docs
git diff --quiet || echo "note: the committed artefacts differ from the sheets — run 'cargo run -p xtask -- docs' and commit"

echo
echo "one command that must be green:"
echo "    cargo run -p xtask -- gate && cargo test"
echo
echo "then:"
echo "    cargo run --release -p vleo-daemon"
echo "and open http://127.0.0.1:7777"
