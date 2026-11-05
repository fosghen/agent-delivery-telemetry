#!/usr/bin/env bash
set -euo pipefail

base_sha="${1:-${BASE_SHA:-}}"
head_sha="${2:-${HEAD_SHA:-}}"

if [[ -z "$base_sha" || -z "$head_sha" ]]; then
  echo "usage: check-change-limits.sh BASE_SHA HEAD_SHA" >&2
  exit 2
fi

if [[ -n "${HEAD_BRANCH:-}" && ! "$HEAD_BRANCH" =~ ^issue-[0-9]+- ]]; then
  echo "branch must match issue-N-*" >&2
  exit 1
fi

commit_count="$(git rev-list --count "$base_sha..$head_sha")"
if (( commit_count > 5 )); then
  echo "pull request has $commit_count commits; maximum is 5" >&2
  exit 1
fi

changed_lines() {
  git diff --numstat "$1" "$2" -- . ':(exclude)Cargo.lock' |
    awk '
      $1 == "-" || $2 == "-" {
        print "binary changes are forbidden" > "/dev/stderr"
        failed = 1
        next
      }
      { total += $1 + $2 }
      END {
        if (failed) exit 1
        print total + 0
      }
    '
}

total="$(changed_lines "$base_sha" "$head_sha")"
if (( total > 375 )); then
  echo "pull request changes $total lines; maximum is 375" >&2
  exit 1
fi

while read -r commit_sha; do
  lines="$(changed_lines "$commit_sha^" "$commit_sha")"
  if (( lines > 75 )); then
    echo "commit $commit_sha changes $lines lines; maximum is 75" >&2
    exit 1
  fi
done < <(git rev-list --reverse "$base_sha..$head_sha")

echo "change limits satisfied: $commit_count commits, $total lines"
