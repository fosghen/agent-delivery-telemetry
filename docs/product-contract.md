# Product contract

## Outcome

Agent Delivery Telemetry explains where delivery time and model tokens were
spent from issue work start through merge. It reports observed evidence and
keeps unavailable measurements explicit; it does not grade engineers or agents.

## Inputs

The tool accepts four classes of input:

1. Local lifecycle events emitted by a CLI or wrapper: trace and stage starts,
   finishes, retries, verification results, and abandonment.
2. Provider evidence from GitHub or GitLab: issue, pull or merge request, CI,
   optional review, and merge timestamps and identifiers.
3. Model usage records: provider, model, input, cached input, output, reasoning,
   and total tokens when the source exposes them.
4. User annotations for decisions that providers cannot observe, such as why an
   independent review was requested.

Every input carries its source, observed time, source time when available,
idempotency key, and provenance. Provider evidence may complete local traces but
must not silently overwrite conflicting observations.

## Provenance

Measurements use one of these precision levels:

- `exact`: supplied by the system that performed or recorded the operation;
- `session_total`: exact for a session but not attributable to an individual stage;
- `estimated`: derived by a documented algorithm from observed values;
- `unavailable`: the source does not expose a defensible value.

Estimated values are excluded from exact totals unless a report explicitly asks
for them. Session totals are never distributed among stages by assumption.

## Outputs

The durable output is a versioned event stream and materialized trace stored
locally. Derived outputs are:

- issue-to-merge lead time and stage durations;
- active, waiting, retry, and rework time;
- tokens by actor, model, stage, attempt, and outcome;
- optional-review invocation, duration, outcome, and confirmed-finding yield;
- deterministic JSON reports and OpenTelemetry-compatible traces;
- warnings for conflicts, incomplete traces, and unavailable measurements.

Reports pair efficiency with outcome and verification status. A lower token or
time value alone is not treated as a better result.

## Privacy boundary

The default collector stores metadata, counts, timestamps, stable identifiers,
and explicitly supplied annotations. It does not collect prompts, responses,
source code, diffs, credentials, environment variables, transcripts, raw logs,
or repository contents.

Adapters request the narrowest provider permissions needed. Tests use synthetic
fixtures. Exports preserve provenance and redact configured repository, user,
branch, and model identifiers without changing aggregate values.

## Failure behavior

Malformed records fail validation without partial persistence. Duplicate records
are idempotent. Rate limits and temporary provider failures remain retryable.
Missing optional review or token data produces a valid incomplete trace, not a
fabricated zero. Schema changes require explicit versioning and migration tests.
