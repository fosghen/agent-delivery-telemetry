#!/usr/bin/env bash
set -euo pipefail

check_file() {
  local path="$1"
  local matches

  if [[ ! -f "$path" ]] || ! grep -Iq . "$path"; then
    return
  fi

  matches="$(grep -nE '(@todo|TODO|FIXME)' "$path" || true)"
  [[ -z "$matches" ]] && return

  while IFS= read -r match; do
    if [[ ! "$match" =~ @todo\ \#[0-9]+:[1-9][0-9]*min\ .{20,} ]]; then
      echo "$path:$match: invalid puzzle marker" >&2
      echo "expected: @todo #ISSUE:MINUTESmin meaningful explanation" >&2
      return 1
    fi
  done <<< "$matches"
}

if (( $# > 0 )); then
  for path in "$@"; do
    check_file "$path"
  done
  exit
fi

failed=0
while IFS= read -r path; do
  case "$path" in
    AGENTS.md|docs/*|scripts/check-puzzles.sh|tests/check-puzzles.sh) continue ;;
  esac
  check_file "$path" || failed=1
done < <(git ls-files)

exit "$failed"
