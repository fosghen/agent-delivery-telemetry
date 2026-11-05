use serde::Serialize;

use crate::comment::UsagePrecision;
use crate::evidence::{EvidenceFailureKind, EvidenceSource};
use crate::metrics::{PortfolioMetrics, TraceAnalysis};
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
