use crate::comment::{COMMENT_MARKER, CommentError, TelemetryPayload, parse_comment};
use crate::evidence::EvidenceSnapshot;
use crate::publish::CommentSnapshot;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ReconciledTrace {
    pub evidence: EvidenceSnapshot,
    pub comment_id: u64,
    pub telemetry: TelemetryPayload,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReconcileError {
    MissingTelemetry,
    DuplicateTelemetry,
    InvalidTelemetry(CommentError),
    StaleTelemetry,
}

/// # Errors
///
/// Returns a typed error for missing, ambiguous, invalid, or stale telemetry.
pub fn reconcile(
    evidence: EvidenceSnapshot,
    comments: &[CommentSnapshot],
    owner: &str,
) -> Result<ReconciledTrace, ReconcileError> {
    let eligible: Vec<_> = comments
        .iter()
        .filter(|comment| comment.author == owner && comment.body.contains(COMMENT_MARKER))
        .collect();
    let comment = match eligible.as_slice() {
        [] => return Err(ReconcileError::MissingTelemetry),
        [comment] => comment,
        _ => return Err(ReconcileError::DuplicateTelemetry),
    };
    let telemetry = parse_comment(&comment.body).map_err(ReconcileError::InvalidTelemetry)?;
    if telemetry.head_sha != evidence.pull_request.head_sha {
        return Err(ReconcileError::StaleTelemetry);
    }
    Ok(ReconciledTrace {
        evidence,
        comment_id: comment.id,
        telemetry,
    })
}
use serde::Serialize;
