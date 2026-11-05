use serde::Serialize;
use std::collections::BTreeMap;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

use crate::comment::{ReviewRecord, StageRecord, UsagePrecision, UsageRecord};
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
    pub stages: Vec<StageMetric>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct StageMetric {
    pub name: String,
    pub duration_ms: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TraceAnalysis {
    pub linked_issues: u32,
    pub delivery: TraceMetrics,
    pub tokens: TokenMetrics,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct StagePercentiles {
    pub name: String,
    pub samples: u32,
    pub p50_ms: u64,
    pub p90_ms: u64,
    pub p95_ms: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct PortfolioMetrics {
    pub traces: u32,
    pub merged_issues: u32,
    pub exact_token_traces: u32,
    pub tokens_per_merged_issue: Option<u64>,
    pub failed_attempt_tokens: Option<u64>,
    pub review_requests: u32,
    pub review_invocation_bps: Option<u32>,
    pub confirmed_findings: u64,
    pub findings_per_review_milli: Option<u32>,
    pub stages: Vec<StagePercentiles>,
}

fn nearest_rank(sorted: &[u64], percentile: usize) -> u64 {
    let rank = percentile.saturating_mul(sorted.len()).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn aggregate_stages(traces: &[TraceAnalysis]) -> Result<Vec<StagePercentiles>, MetricsError> {
    let mut groups: BTreeMap<String, Vec<u64>> = BTreeMap::new();
    for stage in traces.iter().flat_map(|trace| &trace.delivery.stages) {
        groups
            .entry(stage.name.clone())
            .or_default()
            .push(stage.duration_ms);
    }
    groups
        .into_iter()
        .map(|(name, mut values)| {
            values.sort_unstable();
            Ok(StagePercentiles {
                name,
                samples: u32::try_from(values.len()).map_err(|_| MetricsError::Overflow)?,
                p50_ms: nearest_rank(&values, 50),
                p90_ms: nearest_rank(&values, 90),
                p95_ms: nearest_rank(&values, 95),
            })
        })
        .collect()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MetricsError {
    InvalidTimestamp,
    InvalidDuration,
    Overflow,
}

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct TokenTotals {
    pub input: Option<u64>,
    pub cached_input: Option<u64>,
    pub output: Option<u64>,
    pub reasoning: Option<u64>,
    pub total: Option<u64>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct TokenBreakdown {
    pub actor: String,
    pub provider: String,
    pub model: Option<String>,
    pub stage: Option<String>,
    pub attempt: Option<u32>,
    pub totals: TokenTotals,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TokenMetrics {
    pub exact_totals: TokenTotals,
    pub exact: Vec<TokenBreakdown>,
    pub session_totals: Vec<TokenBreakdown>,
    pub estimates: Vec<TokenBreakdown>,
    pub unavailable: Vec<TokenBreakdown>,
    pub cache_share_bps: Option<u32>,
    pub failed_attempt_tokens: Option<u64>,
}

fn record_totals(record: &UsageRecord) -> TokenTotals {
    TokenTotals {
        input: record.input_tokens,
        cached_input: record.cached_input_tokens,
        output: record.output_tokens,
        reasoning: record.reasoning_tokens,
        total: record.total_tokens,
    }
}

fn add_count(target: &mut Option<u64>, value: Option<u64>) -> Result<(), MetricsError> {
    if let Some(value) = value {
        *target = Some(
            target
                .unwrap_or(0)
                .checked_add(value)
                .ok_or(MetricsError::Overflow)?,
        );
    }
    Ok(())
}

fn add_totals(target: &mut TokenTotals, value: TokenTotals) -> Result<(), MetricsError> {
    add_count(&mut target.input, value.input)?;
    add_count(&mut target.cached_input, value.cached_input)?;
    add_count(&mut target.output, value.output)?;
    add_count(&mut target.reasoning, value.reasoning)?;
    add_count(&mut target.total, value.total)
}

fn token_breakdown(record: &UsageRecord) -> TokenBreakdown {
    TokenBreakdown {
        actor: record.actor.clone(),
        provider: record.provider.clone(),
        model: record.model.clone(),
        stage: record.stage.clone(),
        attempt: record.attempt,
        totals: record_totals(record),
    }
}

/// # Errors
///
/// Returns an error when exact token counts overflow.
pub fn compute_token_metrics(trace: &ReconciledTrace) -> Result<TokenMetrics, MetricsError> {
    let failed_attempts: Vec<u32> = trace
        .telemetry
        .attempts
        .iter()
        .filter(|attempt| attempt.outcome != "succeeded")
        .map(|attempt| attempt.number)
        .collect();
    let mut exact_totals = TokenTotals::default();
    let mut exact = Vec::new();
    let mut session_totals = Vec::new();
    let mut estimates = Vec::new();
    let mut unavailable = Vec::new();
    let mut failed_attempt_tokens = None;
    let mut cache_compatible = true;
    for record in &trace.telemetry.usage {
        let value = token_breakdown(record);
        match record.precision {
            UsagePrecision::Exact => {
                add_totals(&mut exact_totals, value.totals)?;
                cache_compatible &=
                    record.cached_input_tokens.is_none() || record.input_tokens.is_some();
                if record
                    .attempt
                    .is_some_and(|attempt| failed_attempts.contains(&attempt))
                {
                    add_count(&mut failed_attempt_tokens, record.total_tokens)?;
                }
                exact.push(value);
            }
            UsagePrecision::SessionTotal => session_totals.push(value),
            UsagePrecision::Estimated => estimates.push(value),
            UsagePrecision::Unavailable => unavailable.push(value),
        }
    }
    exact.sort();
    session_totals.sort();
    estimates.sort();
    unavailable.sort();
    let cache_share_bps = match (
        cache_compatible,
        exact_totals.cached_input,
        exact_totals.input,
    ) {
        (true, Some(cached), Some(input)) if input > 0 && cached <= input => Some(
            u32::try_from(cached.saturating_mul(10_000) / input)
                .map_err(|_| MetricsError::Overflow)?,
        ),
        _ => None,
    };
    Ok(TokenMetrics {
        exact_totals,
        exact,
        session_totals,
        estimates,
        unavailable,
        cache_share_bps,
        failed_attempt_tokens,
    })
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

/// # Errors
///
/// Returns an error for invalid timestamps, durations, or arithmetic overflow.
pub fn compute_trace_metrics(trace: &ReconciledTrace) -> Result<TraceMetrics, MetricsError> {
    let lead_time_ms = trace
        .evidence
        .pull_request
        .merged_at
        .as_deref()
        .map(|merged| elapsed_ms(&trace.evidence.pull_request.created_at, merged))
        .transpose()?;
    let active_time_ms = total_duration(&trace.telemetry.stages)?;
    let wait_time_ms = lead_time_ms
        .zip(active_time_ms)
        .map(|(lead, active)| {
            lead.checked_sub(active)
                .ok_or(MetricsError::InvalidDuration)
        })
        .transpose()?;
    let rework: Vec<_> = trace
        .telemetry
        .stages
        .iter()
        .filter(|stage| stage.name == "rework")
        .cloned()
        .collect();
    let rework_ms = total_duration(&rework)?;
    let rework_share_bps = rework_ms.zip(active_time_ms).and_then(|(rework, active)| {
        (active > 0)
            .then(|| u32::try_from(rework.saturating_mul(10_000) / active).ok())
            .flatten()
    });
    let mut stages: Vec<_> = trace
        .telemetry
        .stages
        .iter()
        .filter_map(|stage| {
            stage.duration_ms.map(|duration_ms| StageMetric {
                name: stage.name.clone(),
                duration_ms,
            })
        })
        .collect();
    stages.sort();
    Ok(TraceMetrics {
        pull_request: trace.evidence.pull_request.number,
        outcome: trace.evidence.pull_request.state.clone(),
        verification: trace.telemetry.verification.outcome.clone(),
        lead_time_ms,
        active_time_ms,
        wait_time_ms,
        retries: u32::try_from(trace.telemetry.attempts.len().saturating_sub(1))
            .map_err(|_| MetricsError::Overflow)?,
        rework_ms,
        rework_share_bps,
        review: review_metrics(&trace.telemetry.review)?,
        stages,
    })
}

/// # Errors
///
/// Returns an error when delivery or exact token metrics are invalid.
pub fn analyze_trace(trace: &ReconciledTrace) -> Result<TraceAnalysis, MetricsError> {
    Ok(TraceAnalysis {
        linked_issues: u32::try_from(trace.evidence.linked_issues.len())
            .map_err(|_| MetricsError::Overflow)?,
        delivery: compute_trace_metrics(trace)?,
        tokens: compute_token_metrics(trace)?,
    })
}

/// # Errors
/// Returns an error when aggregate counts overflow.
pub fn aggregate_metrics(traces: &[TraceAnalysis]) -> Result<PortfolioMetrics, MetricsError> {
    let trace_count = u32::try_from(traces.len()).map_err(|_| MetricsError::Overflow)?;
    let merged: Vec<_> = traces
        .iter()
        .filter(|trace| trace.delivery.lead_time_ms.is_some())
        .collect();
    let merged_issues = merged.iter().try_fold(0_u32, |sum, trace| {
        sum.checked_add(trace.linked_issues)
            .ok_or(MetricsError::Overflow)
    })?;
    let exact_token_traces = merged
        .iter()
        .filter(|trace| trace.tokens.exact_totals.total.is_some())
        .count()
        .try_into()
        .map_err(|_| MetricsError::Overflow)?;
    let exact_tokens = merged.iter().try_fold(Some(0_u64), |sum, trace| {
        match (sum, trace.tokens.exact_totals.total) {
            (Some(sum), Some(value)) => sum
                .checked_add(value)
                .map(Some)
                .ok_or(MetricsError::Overflow),
            _ => Ok(None),
        }
    })?;
    let tokens_per_merged_issue = exact_tokens
        .filter(|_| merged_issues > 0)
        .map(|tokens| tokens / u64::from(merged_issues));
    let failed_attempt_tokens = traces.iter().try_fold(Some(0_u64), |sum, trace| {
        let value = trace
            .tokens
            .failed_attempt_tokens
            .or((trace.delivery.retries == 0).then_some(0));
        match (sum, value) {
            (Some(sum), Some(value)) => sum
                .checked_add(value)
                .map(Some)
                .ok_or(MetricsError::Overflow),
            _ => Ok(None),
        }
    })?;
    let requested: Vec<_> = traces
        .iter()
        .filter_map(|trace| match trace.delivery.review {
            ReviewMetrics::Requested {
                confirmed_findings, ..
            } => Some(confirmed_findings),
            ReviewMetrics::NotRequested => None,
        })
        .collect();
    let review_requests = u32::try_from(requested.len()).map_err(|_| MetricsError::Overflow)?;
    let confirmed_findings = requested.iter().map(|value| u64::from(*value)).sum::<u64>();
    let findings_per_review_milli = if review_requests > 0 {
        u32::try_from(confirmed_findings.saturating_mul(1_000) / u64::from(review_requests)).ok()
    } else {
        None
    };
    Ok(PortfolioMetrics {
        traces: trace_count,
        merged_issues,
        exact_token_traces,
        tokens_per_merged_issue,
        failed_attempt_tokens,
        review_requests,
        review_invocation_bps: (trace_count > 0)
            .then_some(review_requests.saturating_mul(10_000) / trace_count),
        confirmed_findings,
        findings_per_review_milli,
        stages: aggregate_stages(traces)?,
    })
}
