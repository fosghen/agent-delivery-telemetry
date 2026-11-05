use serde::{Deserialize, Serialize};

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
    NonMonotonic,
    Unchanged,
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
