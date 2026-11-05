use serde::{Deserialize, Serialize};

pub const COMMENT_MARKER: &str = "<!-- adt:v1 -->";

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
