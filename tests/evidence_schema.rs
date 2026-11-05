use agent_delivery_telemetry::evidence::{
    CheckEvidence, CommitEvidence, EvidenceSnapshot, EvidenceVisibility, PullRequestEvidence,
    ReviewEvidence,
};

#[test]
fn evidence_round_trips_without_repository_contents() {
    let snapshot = EvidenceSnapshot {
        pull_request: PullRequestEvidence {
            number: 42,
            state: "merged".to_owned(),
            author: "fosghen".to_owned(),
            head_sha: "abc".to_owned(),
            created_at: "2026-11-05T00:00:00Z".to_owned(),
            merged_at: Some("2026-11-05T00:10:00Z".to_owned()),
        },
        linked_issues: vec![7],
        commits: vec![CommitEvidence {
            sha: "abc".to_owned(),
            authored_at: None,
            committed_at: "2026-11-05T00:05:00Z".to_owned(),
        }],
        checks: vec![CheckEvidence {
            id: 9,
            name: "quality".to_owned(),
            status: "completed".to_owned(),
            conclusion: Some("success".to_owned()),
            started_at: None,
            completed_at: Some("2026-11-05T00:08:00Z".to_owned()),
        }],
        reviews: vec![ReviewEvidence {
            id: 10,
            author: "reviewer".to_owned(),
            state: "approved".to_owned(),
            submitted_at: None,
        }],
        visibility: EvidenceVisibility {
            commits: true,
            checks: true,
            reviews: false,
        },
    };

    let json = serde_json::to_string(&snapshot).unwrap();
    assert_eq!(
        serde_json::from_str::<EvidenceSnapshot>(&json).unwrap(),
        snapshot
    );
    assert!(!json.contains("diff"));
    assert!(!json.contains("body"));
}
