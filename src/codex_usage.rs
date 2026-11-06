use serde::{Deserialize, Serialize};

use crate::comment::{UsagePrecision, UsageRecord};

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

#[must_use]
pub fn unavailable_usage() -> UsageRecord {
    UsageRecord {
        actor: "primary".to_owned(),
        provider: "openai".to_owned(),
        model: None,
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
