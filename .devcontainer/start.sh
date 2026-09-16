#!/usr/bin/env bash
# The tool is already running when you arrive.
#
# forwardPorts opens a preview on 7777 the moment something answers there. If
# nothing does, the preview is a connection refused and the first thing a new
# Codespace shows you is an error. This runs on every attach and makes 7777
# answer.
#
# It never fails the attach. Every path exits 0: a Codespace that will not let
# you in because a background convenience did not work is worse than one that
# tells you to type one command.
set -uo pipefail
cd "$(dirname "$0")/.."

PORT="${VLEO_PORT:-7777}"
LOG=/tmp/vleo-daemon.log

if curl -sf -o /dev/null --max-time 2 "http://127.0.0.1:${PORT}/v1/index"; then
  echo "vleo-daemon: already serving on ${PORT}"
  exit 0
fi

BIN=target/release/vleo-daemon
if [ ! -x "$BIN" ]; then
  # setup.sh builds this at creation, so reaching here means a rebuild or a
  # cleaned target. Say so rather than blocking the attach on a release build.
  echo "vleo-daemon: not built. Run it yourself when you are ready:"
  echo "    cargo run --release -p vleo-daemon"
  exit 0
fi

# setsid, not just nohup. postAttachCommand's shell is a task VS Code owns and
# may tear down with its process group when the task ends; a new session makes
# the daemon outlive it. nohup alone survives the hangup but not the group
# kill, and `setsid` is not in every image, so it is used when present.
if command -v setsid >/dev/null 2>&1; then
  setsid "$BIN" >"$LOG" 2>&1 < /dev/null &
else
  nohup "$BIN" >"$LOG" 2>&1 < /dev/null &
fi
disown 2>/dev/null || true
for _ in $(seq 1 30); do
  if curl -sf -o /dev/null --max-time 2 "http://127.0.0.1:${PORT}/v1/index"; then
    echo "vleo-daemon: serving on http://127.0.0.1:${PORT} — the preview tab is the tool"
    exit 0
  fi
  sleep 1
done

echo "vleo-daemon: did not answer within 30s. What it printed:"
tail -20 "$LOG" 2>/dev/null || true
exit 0
