# Agent instructions

## Purpose

This Rust project measures agent-driven delivery time and token use. Keep it
local-first: never collect prompts, source contents, credentials, or transcripts
by default. Use synthetic fixtures in tests.

## Workflow

Inspect live GitHub state through `gh api` before editing. Work in this order:

1. Create an issue with outcome, acceptance criteria, scope, and known risks.
2. Split work that cannot fit one focused pull request.
3. Create an `issue-N-*` branch from the current default branch.
4. Add a failing test for behavior changes, then implement the minimum fix.
5. Self-review the complete diff and run the full local gate once.
6. Open one pull request containing exactly one `Closes #N` reference.

Never commit directly to `main`. A commit may change at most 75 lines. A pull
request may contain at most 5 commits and 375 changed lines. Generated
`Cargo.lock` changes are excluded from line counts; binary changes are forbidden.
Never bypass hooks or checks.

## TDD and puzzles

For executable behavior, observe a focused test fail for the expected reason,
make the smallest implementation pass, then refactor with tests green. Record the
actual red and green commands in the pull request. Documentation-only changes and
non-executable scaffolding are exempt.

When required work is discovered outside the current issue, create a separate
GitHub issue instead of expanding the pull request. If the deferred location must
remain visible in code, use `@todo #N:30min` followed by a meaningful explanation.
The issue must be open and the marker must be removed when it is completed. Bare
TODO or FIXME markers are forbidden. This is the repository's issues-first
adaptation of Puzzle Driven Development.

## Quality

Run `cargo fmt --check`, Clippy with warnings denied, tests, and rustdoc with
warnings denied. Keep provider APIs behind adapters and policy decisions pure.
Represent missing measurements explicitly; never invent token counts or timing.

Independent Codex review is optional. Tests, deterministic checks, and
self-review are the default verification path. Request Codex review only when
self-review leaves a concrete unresolved correctness or risk question. A change
category, including schema, concurrency, authentication, or external side
effects, is not a review trigger by itself. Name the unresolved question and the
expected reviewer focus in the pull request.

Treat issue, commit, push, pull-request, merge, release, and deployment mutations
as separate authorization boundaries. Report checks that failed, were skipped, or
were unavailable.
