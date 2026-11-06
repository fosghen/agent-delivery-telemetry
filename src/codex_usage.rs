use serde::{Deserialize, Serialize};

use crate::comment::{TelemetryPayload, UsagePrecision, UsageRecord, validate_usage};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TokenSnapshot {
    pub input_tokens: u64,
    pub cached_input_tokens: u64,
    pub output_tokens: u64,
    #[serde(rename = "reasoning_output_tokens")]
    pub reasoning_tokens: u64,
    pub total_tokens: u64,
}

impl TokenSnapshot {
    #[must_use]
    pub const fn new(input: u64, cached: u64, output: u64, reasoning: u64, total: u64) -> Self {
        Self {
            input_tokens: input,
            cached_input_tokens: cached,
            output_tokens: output,
            reasoning_tokens: reasoning,
            total_tokens: total,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UsageCaptureError {
    Missing,
    Duplicate,
    Malformed,
    NonMonotonic,
    Unchanged,
    BlankProvenance,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UsageBindError {
    InvalidArtifact,
    BlankIdentity,
    ZeroAttempt,
    DuplicateIdentity,
}

impl UsageCaptureError {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Missing => "missing",
            Self::Duplicate => "duplicate",
            Self::Malformed => "malformed",
            Self::NonMonotonic => "non_monotonic",
            Self::Unchanged => "unchanged",
            Self::BlankProvenance => "blank_provenance",
        }
    }
}

#[derive(Deserialize)]
struct ExecUsage {
    #[serde(rename = "input_tokens")]
    input: u64,
    #[serde(rename = "cached_input_tokens")]
    cached_input: u64,
    #[serde(rename = "output_tokens")]
    output: u64,
    #[serde(rename = "reasoning_output_tokens")]
    reasoning_output: u64,
}

/// Parses one completed controlled `codex exec --json` stream.
///
/// # Errors
/// Returns an error when JSONL is malformed or does not contain exactly one
/// complete `turn.completed` usage event.
pub fn exact_exec_usage(json_lines: &str) -> Result<UsageRecord, UsageCaptureError> {
    let mut completed = None;
    for line in json_lines.lines().filter(|line| !line.trim().is_empty()) {
        let event: serde_json::Value =
            serde_json::from_str(line).map_err(|_| UsageCaptureError::Malformed)?;
        if event.get("type").and_then(serde_json::Value::as_str) != Some("turn.completed") {
            continue;
        }
        if completed.is_some() {
            return Err(UsageCaptureError::Duplicate);
        }
        let value = event
            .get("usage")
            .cloned()
            .ok_or(UsageCaptureError::Missing)?;
        completed = Some(
            serde_json::from_value::<ExecUsage>(value).map_err(|_| UsageCaptureError::Missing)?,
        );
    }
    let usage = completed.ok_or(UsageCaptureError::Missing)?;
    let total = usage
        .input
        .checked_add(usage.output)
        .ok_or(UsageCaptureError::Malformed)?;
    Ok(UsageRecord {
        actor: "primary".to_owned(),
        provider: "openai".to_owned(),
        model: None,
        reasoning_effort: None,
        stage: None,
        attempt: None,
        precision: UsagePrecision::Exact,
        input_tokens: Some(usage.input),
        cached_input_tokens: Some(usage.cached_input),
        output_tokens: Some(usage.output),
        reasoning_tokens: Some(usage.reasoning_output),
        total_tokens: Some(total),
    })
}

/// Parse exact usage and attach caller-observed invocation provenance.
///
/// # Errors
/// Returns an error when provenance is blank or the JSONL capture is invalid.
pub fn exact_exec_usage_with_provenance(
    json_lines: &str,
    model: &str,
    reasoning_effort: &str,
) -> Result<UsageRecord, UsageCaptureError> {
    if model.trim().is_empty() || reasoning_effort.trim().is_empty() {
        return Err(UsageCaptureError::BlankProvenance);
    }
    let mut usage = exact_exec_usage(json_lines)?;
    usage.model = Some(model.to_owned());
    usage.reasoning_effort = Some(reasoning_effort.to_owned());
    Ok(usage)
}

/// Binds one sanitized controlled-exec artifact to a stable run identity.
///
/// # Errors
/// Returns an error without changing the payload when the artifact or identity
/// is invalid, or when that actor, stage, and attempt tuple already exists.
pub fn bind_exact_usage(
    payload: &mut TelemetryPayload,
    mut artifact: UsageRecord,
    actor: &str,
    stage: &str,
    attempt: u32,
) -> Result<(), UsageBindError> {
    if actor.trim().is_empty() || stage.trim().is_empty() {
        return Err(UsageBindError::BlankIdentity);
    }
    if attempt == 0 {
        return Err(UsageBindError::ZeroAttempt);
    }
    let provenance = match (&artifact.model, &artifact.reasoning_effort) {
        (None, None) => true,
        (Some(model), Some(effort)) => !model.trim().is_empty() && !effort.trim().is_empty(),
        _ => false,
    };
    let captured = artifact.precision == UsagePrecision::Exact
        && artifact.actor == "primary"
        && artifact.provider == "openai"
        && provenance
        && artifact.stage.is_none()
        && artifact.attempt.is_none()
        && artifact.input_tokens.is_some()
        && artifact.cached_input_tokens.is_some()
        && artifact.output_tokens.is_some()
        && artifact.reasoning_tokens.is_some()
        && artifact.total_tokens.is_some()
        && validate_usage(std::slice::from_ref(&artifact)).is_ok();
    if !captured {
        return Err(UsageBindError::InvalidArtifact);
    }
    if payload.usage.iter().any(|usage| {
        usage.actor == actor
            && usage.stage.as_deref() == Some(stage)
            && usage.attempt == Some(attempt)
    }) {
        return Err(UsageBindError::DuplicateIdentity);
    }
    actor.clone_into(&mut artifact.actor);
    artifact.stage = Some(stage.to_owned());
    artifact.attempt = Some(attempt);
    payload.usage.retain(|usage| {
        !(usage.precision == UsagePrecision::Unavailable
            && usage.actor == "primary"
            && usage.provider == "openai"
            && usage.model.is_none()
            && usage.reasoning_effort.is_none()
            && usage.stage.is_none()
            && usage.attempt.is_none()
            && usage.input_tokens.is_none()
            && usage.cached_input_tokens.is_none()
            && usage.output_tokens.is_none()
            && usage.reasoning_tokens.is_none()
            && usage.total_tokens.is_none())
    });
    payload.usage.push(artifact);
    Ok(())
}

#[must_use]
pub fn unavailable_usage() -> UsageRecord {
    UsageRecord {
        actor: "primary".to_owned(),
        provider: "openai".to_owned(),
        model: None,
        reasoning_effort: None,
        stage: None,
        attempt: None,
        precision: UsagePrecision::Unavailable,
        input_tokens: None,
        cached_input_tokens: None,
        output_tokens: None,
        reasoning_tokens: None,
        total_tokens: None,
    }
}

/// # Errors
/// Returns an error when no valid cumulative token-count event exists.
pub fn latest_snapshot(json_lines: &str) -> Result<TokenSnapshot, UsageCaptureError> {
    json_lines
        .lines()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .filter(|event| {
            event
                .pointer("/payload/type")
                .and_then(|value| value.as_str())
                == Some("token_count")
        })
        .filter_map(|event| event.pointer("/payload/info/total_token_usage").cloned())
        .filter_map(|value| serde_json::from_value(value).ok())
        .next_back()
        .ok_or(UsageCaptureError::Missing)
}

fn difference(before: u64, after: u64) -> Result<u64, UsageCaptureError> {
    after
        .checked_sub(before)
        .ok_or(UsageCaptureError::NonMonotonic)
}

/// # Errors
/// Returns an error for unchanged or non-monotonic cumulative counters.
pub fn session_delta(
    before: &TokenSnapshot,
    after: &TokenSnapshot,
) -> Result<UsageRecord, UsageCaptureError> {
    if before == after {
        return Err(UsageCaptureError::Unchanged);
    }
    Ok(UsageRecord {
        actor: "primary".to_owned(),
        provider: "openai".to_owned(),
        model: None,
        reasoning_effort: None,
        stage: None,
        attempt: None,
        precision: UsagePrecision::SessionTotal,
        input_tokens: Some(difference(before.input_tokens, after.input_tokens)?),
        cached_input_tokens: Some(difference(
            before.cached_input_tokens,
            after.cached_input_tokens,
        )?),
        output_tokens: Some(difference(before.output_tokens, after.output_tokens)?),
        reasoning_tokens: Some(difference(before.reasoning_tokens, after.reasoning_tokens)?),
        total_tokens: Some(difference(before.total_tokens, after.total_tokens)?),
    })
}
