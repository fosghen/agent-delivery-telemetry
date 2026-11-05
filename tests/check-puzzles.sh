#!/usr/bin/env bash
set -euo pipefail

checker="$(git rev-parse --show-toplevel)/scripts/check-puzzles.sh"
fixture_dir="$(mktemp -d)"
trap 'rm -rf "$fixture_dir"' EXIT

valid="$fixture_dir/valid.rs"
invalid="$fixture_dir/invalid.rs"
short="$fixture_dir/short.rs"

printf '%s\n' '// @todo #16:30min Explain the deferred behavior and completion condition.' > "$valid"
printf '%s\n' '// TODO: silently deferred work' > "$invalid"
printf '%s\n' '// @todo #16:30min Too short.' > "$short"

"$checker" "$valid"

if "$checker" "$invalid"; then
  echo "bare marker unexpectedly passed" >&2
  exit 1
fi

if "$checker" "$short"; then
  echo "underspecified puzzle unexpectedly passed" >&2
  exit 1
fi
