# agent-delivery-telemetry

Repository-native telemetry for understanding how long agent-driven changes take
and where model tokens are spent.

## Status

The repository currently contains its quality foundation and versioned domain
schema. The PR publisher, GitHub reconciler, metrics, and reports are planned;
do not use this project for production telemetry yet.

## Intended workflow

For the MVP, one delivery trace belongs to one GitHub pull request. The agent
publishes local stages, verification, and token provenance in one versioned
`adt:v1` comment. Native GitHub evidence supplies commits, CI, optional-review,
and merge timestamps. Work abandoned before a PR exists is not captured in v0.1.

Remote Codex review is optional. It is measured only when risk or unresolved
correctness uncertainty causes someone to request it.

The complete input, output, privacy, and failure contract is documented in
[`docs/product-contract.md`](docs/product-contract.md).

## Roadmap

Development is organized by capability rather than calendar date:

1. Quality foundation
2. Product contract and event schema
3. Idempotent PR telemetry comment and token capture
4. GitHub timeline and comment reconciliation
5. Delivery, rework, review, and token metrics
6. End-to-end validation and v0.1

Local SQLite persistence, pre-PR attempts, and GitLab support are post-MVP work.

Every change starts from a GitHub issue and is delivered as one focused pull
request. See `AGENTS.md` for the enforced contribution and review policy.

## Local checks

```sh
cargo fmt --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-features
RUSTDOCFLAGS="-D warnings" cargo doc --locked --no-deps --all-features
```

## Publish telemetry

Authenticate with `gh auth login`; the token needs pull-request read and issue
comment write access.

For exact per-run token counts, route a non-interactive agent through the
supported JSON event stream. The telemetry process consumes the stream and
persists only a sanitized usage record; it does not persist the raw trace,
prompt, response, or source contents:

```sh
set -o pipefail
codex exec --json --sandbox workspace-write \
  -c 'approval_policy="never"' 'Implement issue #42' </dev/null \
  | agent-delivery-telemetry capture-codex-usage \
      /tmp/issue-42-usage.json gpt-6.1-sol medium
```

Explicit nonblank model and reasoning-effort provenance is recorded when supplied;
the legacy output-only form remains unattributed. Both forms require exactly one
complete `turn.completed` event. They fail on missing, duplicate, malformed, or
incomplete usage. After finishing a payload,
bind each capture to an explicit stable run identity:

```sh
cargo run -- bind-codex-usage /tmp/payload.json \
  /tmp/issue-42-usage.json subagent-1 implementation 1
```

Actor and stage must be nonblank and attempt must be positive. Distinct tuples
support multiple primary or subagent runs; duplicate tuples and invalid capture
artifacts leave the payload unchanged. Interactive Codex session logs remain a
best-effort fallback because their cumulative counters may lag or be absent.

Before coding, record the task start in an explicit local draft:

```sh
cargo run -- begin fosghen/project 42 /tmp/issue-42.json \
  /path/to/codex/session.jsonl
```

When token metrics are mandatory, use strict mode. It requires a readable session
with a valid token baseline before it creates the draft:

```sh
cargo run -- begin --require-tokens fosghen/project 42 \
  /tmp/issue-42.json /path/to/codex/session.jsonl
```

Missing or invalid token input exits with code `12` and writes no draft. Strict
drafts also make `finish` exit `12` without consuming the draft when counters are
missing, unchanged, unreadable, or non-monotonic. Do not start implementation
until strict `begin` succeeds when repository policy requires token metrics.

After checks pass and the PR head exists, finish the draft. The final argument
is the number of verification checks and the output is a publishable payload:

```sh
cargo run -- finish fosghen/project /tmp/issue-42.json HEAD_SHA 4 /tmp/payload.json
```

Finishing marks the draft consumed. Drafts older than seven days, reused drafts,
repository mismatches, and revisions not resolved from the named GitHub repository
fail before consuming the draft or producing telemetry. Supply the exact pushed PR
head SHA; `finish` verifies it through authenticated `gh api` access.
When an explicit Codex JSONL session is supplied, `begin` records only its latest
cumulative token counters. `finish` reads the same file and publishes the
checked delta as unassigned `session_total` usage. Prompts, responses, transcript
contents, and the local session path never enter the PR comment. Missing,
unchanged, unreadable, or non-monotonic counters remain unavailable in default
best-effort mode; `finish` reports the reason instead of inventing counts. Omit
the session argument only when repository policy permits unavailable tokens.
Strict workflows must stop instead.

Codex may append the final cumulative counters only after the measured agent
turn completes. In the next turn, refresh the already-finished payload and
publish it again; timestamps and all non-token evidence remain unchanged:

```sh
cargo run -- refresh-tokens /tmp/issue-42.json /tmp/payload.json
cargo run -- publish --repo fosghen/project --pr 42 \
  --owner fosghen --payload /tmp/payload.json
```

```sh
cargo run -- publish --repo fosghen/project --pr 42 \
  --owner fosghen --payload trace.json --dry-run
```

Remove `--dry-run` to write the `adt:v1` comment. The command emits one JSON
object. Exit codes are `0` success, `2` input, `3` conflict, and `4` transport.

After publishing telemetry for the current PR head, reconstruct its trace with:

```sh
cargo run -- collect --repo fosghen/project --pr 42 --owner fosghen
```

Collection is read-only. Its JSON combines authoritative PR, commit, check, and
review evidence with the owned `adt:v1` payload. Additional exit codes are `5`
missing telemetry, `6` duplicate telemetry, `7` invalid telemetry, and `8` stale
telemetry. Partial provider visibility is reported inside successful output.

For the versioned metric report, replace `collect` with `report`:

```sh
cargo run -- report --repo fosghen/project --pr 42 --owner fosghen
```

The report includes the reconciled trace, delivery/rework/review/token metrics,
portfolio aggregates, and visibility warnings. Metrics failures use exit code
`9`; the command performs no GitHub mutations.
