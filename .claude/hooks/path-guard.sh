#!/usr/bin/env bash
# Fires before every write.
#
# Generated directories are owned by nobody and editable by nobody. A hand edit
# there is discarded by the next regeneration and fails the regeneration diff —
# so blocking it here saves a person the round trip rather than being the
# authority. The authority is branch protection.
set -uo pipefail
cd "$(dirname "$0")/../.." || exit 0

input=$(cat)
path=$(printf '%s' "$input" | grep -oP '"file_path"\s*:\s*"\K[^"]+' | head -1)
[ -z "$path" ] && exit 0

case "$path" in
  */nodes/*/model.rs)
    # The one generated file with an editable region. The gate's regeneration
    # diff catches an edit outside a numbered HOLE block.
    exit 0 ;;
  */nodes/*/contract.rs|*/nodes/*/mod.rs|*/nodes/*/evidence.rs|*/nodes/*/page.html|*/nodes/*/meta.json)
    cat >&2 <<MSG
Refused: $path is generated from node.toml.

Everything in a node folder except node.toml, fixtures.toml and the numbered
HOLE blocks in model.rs is printed by 'cargo run -p xtask -- docs'. A hand edit
here is discarded by the next regeneration and fails the regeneration diff in
the gate.

Edit the sheet instead:
  ${path%/*}/node.toml
MSG
    exit 2 ;;
  */generated/*)
    echo "Refused: generated/ is an aggregate. It is built, never committed." >&2
    exit 2 ;;
esac
exit 0
