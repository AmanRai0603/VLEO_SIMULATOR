#!/usr/bin/env bash
# Fires when a session ends.
#
# The hook that changes behaviour most: a session cannot end with a node whose
# evidence has not executed, so leaving the tests until tomorrow stops being
# available.
set -uo pipefail
cd "$(dirname "$0")/../.." || exit 0
command -v cargo >/dev/null 2>&1 || exit 0

changed=$(git diff --name-only HEAD -- 'crates/*/nodes/*' 2>/dev/null | head -40)
[ -z "$changed" ] && exit 0

if ! cargo test -q --workspace >/tmp/vleo-evidence.log 2>&1; then
  {
    echo "Refusing to end: a node folder changed and its evidence does not pass."
    echo
    grep -E "panicked at|relative error" /tmp/vleo-evidence.log | head -12
    echo
    echo "A fixture disagreement is a physics disagreement, not a build failure."
    echo "Take it to the node owner. Do not widen the tolerance: a tolerance"
    echo "change is a gate change and needs two reviewers."
  } >&2
  exit 2
fi
exit 0
