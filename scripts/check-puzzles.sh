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
    if [[ "${CHECK_PUZZLE_ISSUES:-0}" == 1 ]]; then
      issue="$(sed -E 's/.*@todo #([0-9]+):.*/\1/' <<< "$match")"
      state="$(gh api "repos/$GITHUB_REPOSITORY/issues/$issue" \
        --jq '[.state, has("pull_request")] | @tsv')"
      if [[ "$state" != $'open\tfalse' ]]; then
        echo "$path: puzzle must reference an open issue" >&2
        return 1
      fi
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
