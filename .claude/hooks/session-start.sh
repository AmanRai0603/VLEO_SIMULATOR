#!/usr/bin/env bash
# Fires when a session opens.
#
# Prints what a person needs before they touch anything: what state the tree is
# in, what is blocking, and whether the local reference-data store is filled.
# A session that starts by guessing at those is a session that discovers them
# at the worst moment.
set -uo pipefail
cd "$(dirname "$0")/../.." || exit 0

# The commit-message rule lives in a tracked folder, so pointing git at it once
# is the whole installation. Done here rather than asked for in a document: a
# setup step somebody has to remember is a setup step somebody skips.
[ -z "$(git config --get core.hooksPath || true)" ] && git config core.hooksPath tools/githooks

echo "── VLEO integrated design tool ──────────────────────────────────────"
if command -v cargo >/dev/null 2>&1; then
  cargo run -q -p xtask -- status 2>/dev/null | tail -8
  echo
  echo "reference data:"
  cargo run -q -p vleo-cli --bin vleo -- data 2>/dev/null | head -4 \
    || echo "  the store is empty — 'cargo run -p vleo-cli --bin vleo -- data sync' fills it"
else
  echo "no cargo on PATH; the toolchain is pinned in rust-toolchain.toml"
fi
echo
echo "one command that must be green:  cargo run -p xtask -- gate && cargo test"
echo "the working tool:                cargo run --release -p vleo-daemon"
echo "─────────────────────────────────────────────────────────────────────"
exit 0
