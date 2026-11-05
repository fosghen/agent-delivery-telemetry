# Product contract

## Outcome

Agent Delivery Telemetry explains where delivery time and model tokens were
spent from issue work start through merge. It reports observed evidence and
keeps unavailable measurements explicit; it does not grade engineers or agents.

## Inputs

The GitHub-backed MVP accepts three classes of input:

1. Native GitHub evidence: issue, pull request, commits, CI, optional review, and
   merge timestamps and identifiers.
2. One pull-request comment marked `adt:v1`, containing work start, local
   stages, attempts, verification outcomes, token usage, and provenance.
3. User annotations in that record for decisions GitHub cannot observe, such as
   why an independent review was requested.

The publisher creates the marked comment once and updates that same comment
idempotently. Native GitHub evidence remains authoritative for provider-visible
events and must not be overwritten by comment values.

## Provenance

Measurements use one of these precision levels:

- `exact`: supplied by the system that performed or recorded the operation;
- `session_total`: exact for a session but not attributable to an individual stage;
- `estimated`: derived by a documented algorithm from observed values;
- `unavailable`: the source does not expose a defensible value.

Estimated values are excluded from exact totals unless a report explicitly asks
for them. Session totals are never distributed among stages by assumption.

## Outputs

The durable MVP record is the versioned PR comment plus the native GitHub
timeline. The tool reconstructs a trace on demand and derives:

- work-start-to-merge delivery lead time and stage durations;
- pull-request-created-to-merge cycle time;
- active, waiting, retry, and rework time;
- tokens by actor, model, stage, attempt, and outcome;
- optional-review invocation, duration, outcome, and confirmed-finding yield;
- deterministic JSON reports and OpenTelemetry-compatible traces;
- warnings for conflicts, incomplete traces, and unavailable measurements.

Reports pair efficiency with outcome and verification status. A lower token or
time value alone is not treated as a better result.

## Privacy boundary

The telemetry comment stores metadata, counts, timestamps, stable identifiers,
and explicitly supplied annotations. It does not contain prompts, responses,
source code, diffs, credentials, environment variables, transcripts, raw logs,
or repository contents.

Adapters request the narrowest provider permissions needed. Tests use synthetic
fixtures. Exports preserve provenance and redact configured repository, user,
branch, and model identifiers without changing aggregate values.

## Failure behavior

Malformed comments fail validation without being interpreted. Multiple marked
comments are a conflict rather than an implicit choice. Rate limits and temporary
GitHub failures remain retryable. Missing optional review or token data produces
a valid incomplete trace, not a fabricated zero.

Work abandoned before a pull request exists is outside the v0.1 report boundary.
A small local draft may carry observed pre-PR metadata into the eventual PR;
durable local persistence and GitLab reconciliation remain post-MVP. Schema
changes require explicit versioning and compatibility tests.
