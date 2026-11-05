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

```sh
cargo run -- publish --repo fosghen/project --pr 42 \
  --owner fosghen --payload trace.json --dry-run
```

Remove `--dry-run` to write the `adt:v1` comment. The command emits one JSON
object. Exit codes are `0` success, `2` input, `3` conflict, and `4` transport.
