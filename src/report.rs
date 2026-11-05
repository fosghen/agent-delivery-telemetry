use serde::Serialize;
use std::collections::BTreeSet;

use crate::comment::UsagePrecision;
use crate::evidence::{EvidenceFailureKind, EvidenceSource};
use crate::metrics::{
    MetricsError, PortfolioMetrics, TraceAnalysis, aggregate_metrics, analyze_trace,
};
use crate::reconcile::ReconciledTrace;

pub const REPORT_SCHEMA_VERSION: u16 = 1;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TraceReport {
    pub trace: ReconciledTrace,
    pub analysis: TraceAnalysis,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct AnalysisReport {
    pub schema_version: u16,
    pub traces: Vec<TraceReport>,
    pub portfolio: PortfolioMetrics,
    pub warnings: Vec<ReportWarning>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct ReportWarning {
    pub pull_request: u64,
    pub kind: WarningKind,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WarningKind {
    PartialEvidence {
        source: EvidenceSource,
        failure: EvidenceFailureKind,
    },
    UnavailableMetric {
        name: String,
    },
    NonExactUsage {
        precision: UsagePrecision,
    },
}

/// # Errors
///
/// Returns an error when trace or portfolio metrics cannot be computed.
pub fn build_report(traces: Vec<ReconciledTrace>) -> Result<AnalysisReport, MetricsError> {
    let mut reports = Vec::with_capacity(traces.len());
    let mut warnings = BTreeSet::new();
    for trace in traces {
        let pull_request = trace.evidence.pull_request.number;
        for failure in &trace.evidence.failures {
            warnings.insert(ReportWarning {
                pull_request,
                kind: WarningKind::PartialEvidence {
                    source: failure.source,
                    failure: failure.kind,
                },
            });
        }
        for usage in &trace.telemetry.usage {
            if usage.precision != UsagePrecision::Exact {
                warnings.insert(ReportWarning {
                    pull_request,
                    kind: WarningKind::NonExactUsage {
                        precision: usage.precision,
                    },
                });
            }
        }
        let analysis = analyze_trace(&trace)?;
        for (name, missing) in [
            ("lead_time", analysis.delivery.lead_time_ms.is_none()),
            ("active_time", analysis.delivery.active_time_ms.is_none()),
            ("exact_tokens", analysis.tokens.exact_totals.total.is_none()),
        ] {
            if missing {
                warnings.insert(ReportWarning {
                    pull_request,
                    kind: WarningKind::UnavailableMetric {
                        name: name.to_owned(),
                    },
                });
            }
        }
        reports.push(TraceReport { trace, analysis });
    }
    reports.sort_by_key(|report| report.trace.evidence.pull_request.number);
    let analyses: Vec<_> = reports
        .iter()
        .map(|report| report.analysis.clone())
        .collect();
    Ok(AnalysisReport {
        schema_version: REPORT_SCHEMA_VERSION,
        portfolio: aggregate_metrics(&analyses)?,
        traces: reports,
        warnings: warnings.into_iter().collect(),
    })
}
