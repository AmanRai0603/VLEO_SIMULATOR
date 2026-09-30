#!/usr/bin/env bash
# Publish a release from the artefacts the release workflow built.
#
#   tools/publish_release.sh <version> <artefacts-dir>
#
# One script for both ways a release is approved (a pushed vX.Y.Z tag, or a
# reviewer on the `release` environment), so the two cannot publish different
# things. It gathers the kits, the desktop apps and the wheel, writes
# SHA256SUMS.txt beside them — the one thing that lets somebody check an
# unsigned download is the file that was built — and creates the release.
set -euo pipefail
v="$1"
art="$2"
notes="$art/notes/notes.md"
[ -f "$notes" ] || { echo "no release notes at $notes" >&2; exit 1; }

up="$(mktemp -d)"
find "$art" -type f \( -name 'vleo-*.zip' -o -name 'vleo-*.whl' -o -name 'VLEO*Design*Tool*.zip' \) \
  -exec mv {} "$up/" \;
n=$(find "$up" -type f | wc -l)
[ "$n" -gt 0 ] || { echo "nothing to publish in $art" >&2; exit 1; }

# Checksums of exactly what is uploaded, named as uploaded.
( cd "$up" && sha256sum -- * > SHA256SUMS.txt )
{
  echo ""
  echo "### Checking a download"
  echo ""
  echo "These programs are not signed yet. \`SHA256SUMS.txt\` lists the SHA-256 of every file"
  echo "here: \`sha256sum -c SHA256SUMS.txt\` (Linux), \`shasum -a 256 -c SHA256SUMS.txt\` (macOS),"
  echo "or \`Get-FileHash <file>\` (Windows PowerShell). docs/FIRST_RUN.md says how to open"
  echo "an unsigned program the first time on each system."
} >> "$notes"

cat "$up/SHA256SUMS.txt"
gh release create "v$v" --title "v$v" --notes-file "$notes" "$up"/* \
  || gh release edit "v$v" --notes-file "$notes"
