#!/usr/bin/env bash
# Fires after every write.
#
# Format, lint and the declaration check — about a second, so it can run on
# every edit. The expensive checks (golden vectors, mutation, cross-face
# agreement) run on demand and in the pipeline. The split is by cost, not by
# importance.
set -uo pipefail
cd "$(dirname "$0")/../.." || exit 0

input=$(cat)
path=$(printf '%s' "$input" | grep -oP '"file_path"\s*:\s*"\K[^"]+' | head -1)
[ -z "$path" ] && exit 0
command -v cargo >/dev/null 2>&1 || exit 0

case "$path" in
  *node.toml|*fixtures.toml)
    node=$(grep -oP '^id\s*=\s*"\K[^"]+' "${path%/*}/node.toml" 2>/dev/null | head -1)
    [ -z "$node" ] && exit 0
    out=$(cargo run -q -p xtask -- gate "$node" 2>&1)
    if printf '%s' "$out" | grep -q FAIL; then
      printf 'The gate refused %s:\n%s\n' "$node" "$out" >&2
      exit 2
    fi
    printf '%s\n' "$out" ;;
  *.rs)
    cargo fmt -- --check "$path" >/dev/null 2>&1 || cargo fmt -- "$path" >/dev/null 2>&1 ;;
esac
exit 0
