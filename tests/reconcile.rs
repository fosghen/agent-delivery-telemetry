use agent_delivery_telemetry::comment::{
    ReviewRecord, TelemetryPayload, VerificationRecord, render_comment,
};
use agent_delivery_telemetry::evidence::{
    EvidenceSnapshot, EvidenceVisibility, PullRequestEvidence,
};
use agent_delivery_telemetry::publish::CommentSnapshot;
use agent_delivery_telemetry::reconcile::{ReconcileError, reconcile};

fn evidence() -> EvidenceSnapshot {
    EvidenceSnapshot {
        pull_request: PullRequestEvidence {
            number: 42,
            state: "open".to_owned(),
            author: "fosghen".to_owned(),
            head_sha: "abc".to_owned(),
            created_at: "2026-11-05T00:00:00Z".to_owned(),
            merged_at: None,
        },
        linked_issues: vec![7],
        commits: vec![],
        checks: vec![],
        reviews: vec![],
        visibility: EvidenceVisibility {
            commits: true,
            checks: true,
            reviews: true,
        },
        failures: vec![],
    }
}

fn payload(head_sha: &str) -> TelemetryPayload {
    TelemetryPayload {
        schema_version: 1,
        head_sha: head_sha.to_owned(),
        work_started_at: None,
        attempts: vec![],
        stages: vec![],
        verification: VerificationRecord {
            outcome: "passed".to_owned(),
            checks: 3,
        },
        review: ReviewRecord::NotRequested,
        usage: vec![],
    }
}

fn comment(id: u64, head_sha: &str) -> CommentSnapshot {
    CommentSnapshot {
        id,
        author: "fosghen".to_owned(),
        body: render_comment(&payload(head_sha)).unwrap(),
    }
}

#[test]
fn matching_owned_comment_produces_a_deterministic_trace() {
    let trace = reconcile(evidence(), &[comment(9, "abc")], "fosghen").unwrap();
    assert_eq!(trace.comment_id, 9);
    assert_eq!(trace.telemetry, payload("abc"));
    assert_eq!(trace.evidence.pull_request.number, 42);
}

#[test]
fn mismatched_revision_is_stale() {
    assert_eq!(
        reconcile(evidence(), &[comment(9, "old")], "fosghen"),
        Err(ReconcileError::StaleTelemetry)
    );
}

#[test]
fn missing_and_duplicate_owned_telemetry_are_distinct() {
    assert_eq!(
        reconcile(evidence(), &[], "fosghen"),
        Err(ReconcileError::MissingTelemetry)
    );
    assert_eq!(
        reconcile(
            evidence(),
            &[comment(9, "abc"), comment(10, "abc")],
            "fosghen"
        ),
        Err(ReconcileError::DuplicateTelemetry)
    );
}

#[test]
fn malformed_and_unsupported_payloads_remain_typed() {
    use agent_delivery_telemetry::comment::{COMMENT_MARKER, CommentError};
    let malformed = CommentSnapshot {
        id: 9,
        author: "fosghen".to_owned(),
        body: format!("{COMMENT_MARKER}\n```json\n{{\n```"),
    };
    assert_eq!(
        reconcile(evidence(), &[malformed], "fosghen"),
        Err(ReconcileError::InvalidTelemetry(
            CommentError::MalformedJson
        ))
    );
    let mut future = payload("abc");
    future.schema_version = 2;
    let unsupported = CommentSnapshot {
        id: 10,
        author: "fosghen".to_owned(),
        body: render_comment(&future).unwrap(),
    };
    assert_eq!(
        reconcile(evidence(), &[unsupported], "fosghen"),
        Err(ReconcileError::InvalidTelemetry(
            CommentError::UnsupportedVersion(2)
        ))
    );
}
