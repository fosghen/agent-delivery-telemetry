use serde::{Deserialize, Serialize};

pub const COMMENT_MARKER: &str = "<!-- adt:v1 -->";
const JSON_FENCE: &str = "```json\n";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommentError {
    MissingMarker,
    DuplicateMarker,
    MissingPayload,
    DuplicatePayload,
    MalformedJson,
    InvalidUsage,
    UnsupportedVersion(u16),
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TelemetryPayload {
    pub schema_version: u16,
    pub attempts: Vec<AttemptRecord>,
    pub stages: Vec<StageRecord>,
    pub verification: VerificationRecord,
    pub usage: Vec<UsageRecord>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AttemptRecord {
    pub number: u32,
    pub outcome: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StageRecord {
    pub name: String,
    pub outcome: String,
    pub duration_ms: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VerificationRecord {
    pub outcome: String,
    pub checks: u32,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UsagePrecision {
    Exact,
    SessionTotal,
    Estimated,
    Unavailable,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct UsageRecord {
    pub actor: String,
    pub provider: String,
    pub model: Option<String>,
    pub stage: Option<String>,
    pub precision: UsagePrecision,
    pub input_tokens: Option<u64>,
    pub cached_input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub reasoning_tokens: Option<u64>,
    pub total_tokens: Option<u64>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UsageError {
    BlankIdentity,
    InconsistentTotal,
    SessionTotalHasStage,
    UnavailableHasCounts,
}

/// # Errors
///
/// Returns an error when token provenance or counts are inconsistent.
pub fn validate_usage(records: &[UsageRecord]) -> Result<(), UsageError> {
    for record in records {
        if record.actor.trim().is_empty() || record.provider.trim().is_empty() {
            return Err(UsageError::BlankIdentity);
        }
        let counts = [
            record.input_tokens,
            record.cached_input_tokens,
            record.output_tokens,
            record.reasoning_tokens,
            record.total_tokens,
        ];
        if record.precision == UsagePrecision::Unavailable && counts.iter().any(Option::is_some) {
            return Err(UsageError::UnavailableHasCounts);
        }
        if record.precision == UsagePrecision::SessionTotal && record.stage.is_some() {
            return Err(UsageError::SessionTotalHasStage);
        }
        if let (Some(input), Some(output), Some(total)) = (
            record.input_tokens,
            record.output_tokens,
            record.total_tokens,
        ) && input.checked_add(output) != Some(total)
        {
            return Err(UsageError::InconsistentTotal);
        }
    }
    Ok(())
}

/// # Errors
///
/// Returns an error when the payload cannot be encoded as JSON.
pub fn render_comment(payload: &TelemetryPayload) -> serde_json::Result<String> {
    let json = serde_json::to_string_pretty(payload)?;
    Ok(format!(
        "{COMMENT_MARKER}\n\
         ## Agent Delivery Telemetry\n\n\
         - Attempts: {}\n\
         - Stages: {}\n\
         - Verification: {} ({} checks)\n\
         - Token usage records: {}\n\n\
         <details><summary>Machine data (adt:v1)</summary>\n\n\
         ```json\n{json}\n```\n\n\
         </details>",
        payload.attempts.len(),
        payload.stages.len(),
        payload.verification.outcome,
        payload.verification.checks,
        payload.usage.len(),
    ))
}

/// # Errors
///
/// Returns an error when the marker, JSON block, or payload is invalid.
pub fn parse_comment(comment: &str) -> Result<TelemetryPayload, CommentError> {
    match comment.matches(COMMENT_MARKER).count() {
        0 => return Err(CommentError::MissingMarker),
        1 => {}
        _ => return Err(CommentError::DuplicateMarker),
    }
    let marker = comment
        .find(COMMENT_MARKER)
        .ok_or(CommentError::MissingMarker)?;
    let body = &comment[marker + COMMENT_MARKER.len()..];
    match body.matches(JSON_FENCE).count() {
        0 => return Err(CommentError::MissingPayload),
        1 => {}
        _ => return Err(CommentError::DuplicatePayload),
    }
    let start = body.find(JSON_FENCE).ok_or(CommentError::MissingPayload)?;
    let json = &body[start + JSON_FENCE.len()..];
    let end = json.find("\n```").ok_or(CommentError::MissingPayload)?;
    let payload: TelemetryPayload =
        serde_json::from_str(&json[..end]).map_err(|_| CommentError::MalformedJson)?;
    if payload.schema_version != crate::SCHEMA_VERSION {
        return Err(CommentError::UnsupportedVersion(payload.schema_version));
    }
    validate_usage(&payload.usage).map_err(|_| CommentError::InvalidUsage)?;
    Ok(payload)
}
