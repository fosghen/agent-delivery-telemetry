use serde::Serialize;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "decision", rename_all = "snake_case")]
pub enum ReviewMetrics {
    NotRequested,
    Requested {
        duration_ms: Option<u64>,
        confirmed_findings: u32,
        rework_ms: Option<u64>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TraceMetrics {
    pub pull_request: u64,
    pub outcome: String,
    pub verification: String,
    pub lead_time_ms: Option<u64>,
    pub active_time_ms: Option<u64>,
    pub wait_time_ms: Option<u64>,
    pub retries: u32,
    pub rework_ms: Option<u64>,
    pub rework_share_bps: Option<u32>,
    pub review: ReviewMetrics,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MetricsError {
    InvalidTimestamp,
    InvalidDuration,
    Overflow,
}
