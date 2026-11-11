use serde::{Deserialize, Serialize};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

pub const COMMENT_MARKER: &str = "<!-- adt:v1 -->";
const JSON_FENCE: &str = "```json\n";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommentError {
    MissingMarker,
    DuplicateMarker,
    MissingPayload,
    DuplicatePayload,
    MalformedJson,
    BlankHeadRevision,
    InvalidUsage,
    UnsupportedVersion(u16),
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TelemetryPayload {
    pub schema_version: u16,
    pub head_sha: String,
    #[serde(default)]
    pub work_started_at: Option<String>,
    pub attempts: Vec<AttemptRecord>,
    pub stages: Vec<StageRecord>,
    pub verification: VerificationRecord,
    pub review: ReviewRecord,
    pub usage: Vec<UsageRecord>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AttemptRecord {
    pub number: u32,
    pub outcome: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttemptError {
    InvalidLifecycle,
    Overflow,
}

/// Marks the current controlled run failed and opens the next attempt.
///
/// # Errors
/// Returns an error unless attempts are contiguous and end in one success placeholder.
pub fn record_failed_attempt(payload: &mut TelemetryPayload) -> Result<u32, AttemptError> {
    let count = u32::try_from(payload.attempts.len()).map_err(|_| AttemptError::Overflow)?;
    let valid = count > 0
        && payload.attempts.iter().enumerate().all(|(index, attempt)| {
            u32::try_from(index + 1) == Ok(attempt.number)
                && if attempt.number == count {
                    attempt.outcome == "succeeded"
                } else {
                    attempt.outcome == "failed"
                }
        });
    if !valid {
        return Err(AttemptError::InvalidLifecycle);
    }
    let next = count.checked_add(1).ok_or(AttemptError::Overflow)?;
    payload
        .attempts
        .last_mut()
        .ok_or(AttemptError::InvalidLifecycle)?
        .outcome = "failed".to_owned();
    payload.attempts.push(AttemptRecord {
        number: next,
        outcome: "succeeded".to_owned(),
    });
    Ok(count)
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

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "decision", rename_all = "snake_case")]
pub enum ReviewRecord {
    NotRequested,
    Requested {
        trigger: String,
        requested_at: String,
        completed_at: Option<String>,
        outcome: Option<String>,
        confirmed_findings: u32,
        rework_ms: Option<u64>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviewError {
    AlreadyRecorded,
    BlankField,
    InvalidTimestamp,
    InvalidOrder,
}

/// Records explicit evidence for one completed review.
///
/// # Errors
/// Returns an error for duplicate, blank, malformed, or reversed evidence.
pub fn record_completed_review(
    payload: &mut TelemetryPayload,
    trigger: &str,
    requested_at: &str,
    completed_at: &str,
    outcome: &str,
    confirmed_findings: u32,
    rework_ms: u64,
) -> Result<(), ReviewError> {
    if payload.review != ReviewRecord::NotRequested {
        return Err(ReviewError::AlreadyRecorded);
    }
    if trigger.trim().is_empty() || outcome.trim().is_empty() {
        return Err(ReviewError::BlankField);
    }
    let requested =
        OffsetDateTime::parse(requested_at, &Rfc3339).map_err(|_| ReviewError::InvalidTimestamp)?;
    let completed =
        OffsetDateTime::parse(completed_at, &Rfc3339).map_err(|_| ReviewError::InvalidTimestamp)?;
    if completed < requested {
        return Err(ReviewError::InvalidOrder);
    }
    payload.review = ReviewRecord::Requested {
        trigger: trigger.to_owned(),
        requested_at: requested_at.to_owned(),
        completed_at: Some(completed_at.to_owned()),
        outcome: Some(outcome.to_owned()),
        confirmed_findings,
        rework_ms: Some(rework_ms),
    };
    Ok(())
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
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
    #[serde(default)]
    pub reasoning_effort: Option<String>,
    pub stage: Option<String>,
    pub attempt: Option<u32>,
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
    SessionTotalAttributed,
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
        if record.precision == UsagePrecision::SessionTotal
            && (record.stage.is_some() || record.attempt.is_some())
        {
            return Err(UsageError::SessionTotalAttributed);
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
    if payload.head_sha.trim().is_empty() {
        return Err(CommentError::BlankHeadRevision);
    }
    validate_usage(&payload.usage).map_err(|_| CommentError::InvalidUsage)?;
    Ok(payload)
}
