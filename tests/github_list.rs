use agent_delivery_telemetry::github::{CommandRunner, GhCommentPort, GhError};

struct FakeRunner(String);

impl CommandRunner for FakeRunner {
    type Error = ();

    fn run(&mut self, _: &[String], _: Option<&str>) -> Result<String, Self::Error> {
        Ok(self.0.clone())
    }
}

#[test]
fn list_flattens_paginated_github_comments() {
    let output = r#"[[{"id":7,"user":{"login":"bot"},"body":"one"}],
                    [{"id":8,"user":{"login":"human"},"body":"two"}]]"#;
    let mut port = GhCommentPort::new(FakeRunner(output.to_owned()), "fosghen/project", 42);

    let comments = port.list_comments().unwrap();

    assert_eq!(comments.len(), 2);
    assert_eq!(comments[0].author, "bot");
}

#[test]
fn list_preserves_command_and_decode_failures() {
    struct FailRunner;
    impl CommandRunner for FailRunner {
        type Error = &'static str;
        fn run(&mut self, _: &[String], _: Option<&str>) -> Result<String, Self::Error> {
            Err("denied")
        }
    }

    let mut failed = GhCommentPort::new(FailRunner, "fosghen/project", 42);
    assert_eq!(failed.list_comments(), Err(GhError::Command("denied")));
    let mut malformed = GhCommentPort::new(FakeRunner("{".to_owned()), "fosghen/project", 42);
    assert_eq!(malformed.list_comments(), Err(GhError::Decode));
}
