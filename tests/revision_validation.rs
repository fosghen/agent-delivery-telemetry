use agent_delivery_telemetry::github::{CommandRunner, revision_belongs_to_repository};

struct FakeRunner {
    response: Result<String, ()>,
    arguments: Vec<String>,
}

impl CommandRunner for FakeRunner {
    type Error = ();

    fn run(&mut self, arguments: &[String], _input: Option<&str>) -> Result<String, Self::Error> {
        self.arguments = arguments.to_vec();
        self.response.clone()
    }
}

#[test]
fn revision_must_resolve_to_the_exact_target_repository_sha() {
    for (response, expected) in [
        (Ok("target\n".to_owned()), true),
        (Ok("foreign\n".into()), false),
    ] {
        let mut runner = FakeRunner {
            response,
            arguments: Vec::new(),
        };
        assert_eq!(
            revision_belongs_to_repository(&mut runner, "fosghen/project", "target"),
            Ok(expected)
        );
        assert_eq!(runner.arguments[1], "repos/fosghen/project/commits/target");
    }
}
