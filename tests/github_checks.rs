use agent_delivery_telemetry::evidence::CheckEvidence;
use agent_delivery_telemetry::github::{CommandRunner, GhCommentPort};

struct FixtureRunner;

impl CommandRunner for FixtureRunner {
    type Error = ();

    fn run(&mut self, arguments: &[String], _input: Option<&str>) -> Result<String, Self::Error> {
        assert_eq!(
            arguments,
            [
                "api",
                "--paginate",
                "--slurp",
                "repos/fosghen/project/commits/abc/check-runs"
            ]
        );
        Ok(r#"[{"check_runs":[{"id":9,"name":"quality","status":"completed","conclusion":"success","started_at":null,"completed_at":"2026-11-05T00:08:00Z"}]},{"check_runs":[]}]"#.to_owned())
    }
}

#[test]
fn paginated_check_runs_are_flattened() {
    let mut port = GhCommentPort::new(FixtureRunner, "fosghen/project", 42);

    assert_eq!(
        port.check_evidence("abc").unwrap(),
        vec![CheckEvidence {
            id: 9,
            name: "quality".to_owned(),
            status: "completed".to_owned(),
            conclusion: Some("success".to_owned()),
            started_at: None,
            completed_at: Some("2026-11-05T00:08:00Z".to_owned()),
        }]
    );
}
