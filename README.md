# agent-delivery-telemetry

Local-first telemetry for understanding how long agent-driven changes take and
where model tokens are spent.

## Status

The repository currently contains its Rust scaffold and quality foundation. The
collector, provider adapters, storage, metrics, and reports are planned work; do
not use this project for production telemetry yet.

## Intended workflow

One delivery trace starts when work on an issue begins and ends when the change
is merged or abandoned. Local events describe implementation and verification;
GitHub or GitLab evidence supplies pull-request, CI, optional-review, and merge
timestamps. Token records retain whether their precision is exact, session-wide,
estimated, or unavailable.

Remote Codex review is optional. It is measured only when risk or unresolved
correctness uncertainty causes someone to request it.

The complete input, output, privacy, and failure contract is documented in
[`docs/product-contract.md`](docs/product-contract.md).

## Roadmap

Development is organized by capability rather than calendar date:

1. Quality foundation
2. Product contract and event schema
3. Local capture and SQLite persistence
4. GitHub and GitLab reconciliation
5. Delivery, rework, review, and token metrics
6. End-to-end validation and v0.1

Every change starts from a GitHub issue and is delivered as one focused pull
request. See `AGENTS.md` for the enforced contribution and review policy.

## Local checks

```sh
cargo fmt --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-features
RUSTDOCFLAGS="-D warnings" cargo doc --locked --no-deps --all-features
```
