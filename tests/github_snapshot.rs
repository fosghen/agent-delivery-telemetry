use std::collections::VecDeque;

use agent_delivery_telemetry::evidence::{EvidenceFailureKind, EvidenceSource};
use agent_delivery_telemetry::github::{ClassifyFailure, CommandRunner, GhCommentPort};

#[derive(Clone, Copy)]
enum Failure {
    Command,
    RateLimited,
}

impl ClassifyFailure for Failure {
    fn failure_kind(&self) -> EvidenceFailureKind {
        match self {
            Self::Command => EvidenceFailureKind::Command,
            Self::RateLimited => EvidenceFailureKind::RateLimited,
        }
    }
}

struct ScriptedRunner(VecDeque<Result<&'static str, Failure>>);

impl CommandRunner for ScriptedRunner {
    type Error = Failure;

    fn run(&mut self, _arguments: &[String], _input: Option<&str>) -> Result<String, Self::Error> {
        self.0.pop_front().unwrap().map(str::to_owned)
    }
}

#[test]
fn snapshot_retries_commands_and_marks_rate_limited_evidence_missing() {
    let responses = VecDeque::from([
        Ok(
            r#"{"number":42,"state":"closed","user":{"login":"fosghen"},"head":{"sha":"abc"},"created_at":"2026-11-05T00:00:00Z","merged_at":null,"body":"Closes #7"}"#,
        ),
        Err(Failure::Command),
        Ok(
            r#"[[{"sha":"abc","commit":{"author":{"date":null},"committer":{"date":"2026-11-05T00:05:00Z"}}}]]"#,
        ),
        Ok(r#"[{"check_runs":[]}]"#),
        Err(Failure::RateLimited),
    ]);
    let mut port = GhCommentPort::new(ScriptedRunner(responses), "fosghen/project", 42);

    let snapshot = port.evidence_snapshot().unwrap();

    assert_eq!(snapshot.linked_issues, vec![7]);
    assert_eq!(snapshot.commits.len(), 1);
    assert!(snapshot.visibility.commits && snapshot.visibility.checks);
    assert!(!snapshot.visibility.reviews);
    assert_eq!(snapshot.failures.len(), 1);
    assert_eq!(snapshot.failures[0].source, EvidenceSource::Reviews);
    assert_eq!(snapshot.failures[0].kind, EvidenceFailureKind::RateLimited);
}
