use serde::Serialize;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

use crate::comment::{ReviewRecord, StageRecord};
use crate::reconcile::ReconciledTrace;

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

fn elapsed_ms(start: &str, finish: &str) -> Result<u64, MetricsError> {
    let start =
        OffsetDateTime::parse(start, &Rfc3339).map_err(|_| MetricsError::InvalidTimestamp)?;
    let finish =
        OffsetDateTime::parse(finish, &Rfc3339).map_err(|_| MetricsError::InvalidTimestamp)?;
    u64::try_from((finish - start).whole_milliseconds()).map_err(|_| MetricsError::InvalidDuration)
}

fn total_duration(stages: &[StageRecord]) -> Result<Option<u64>, MetricsError> {
    stages.iter().try_fold(Some(0_u64), |total, stage| {
        match (total, stage.duration_ms) {
            (Some(total), Some(value)) => total
                .checked_add(value)
                .map(Some)
                .ok_or(MetricsError::Overflow),
            _ => Ok(None),
        }
    })
}

fn review_metrics(review: &ReviewRecord) -> Result<ReviewMetrics, MetricsError> {
    Ok(match review {
        ReviewRecord::NotRequested => ReviewMetrics::NotRequested,
        ReviewRecord::Requested {
            requested_at,
            completed_at,
            confirmed_findings,
            rework_ms,
            ..
        } => ReviewMetrics::Requested {
            duration_ms: completed_at
                .as_deref()
                .map(|completed| elapsed_ms(requested_at, completed))
                .transpose()?,
            confirmed_findings: *confirmed_findings,
            rework_ms: *rework_ms,
        },
    })
}

