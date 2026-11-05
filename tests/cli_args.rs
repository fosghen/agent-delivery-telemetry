use agent_delivery_telemetry::cli::{PublishArgs, parse_publish_args};

#[test]
fn publish_arguments_require_explicit_target_and_payload() {
    let arguments = [
        "--repo",
        "fosghen/project",
        "--pr",
        "42",
        "--owner",
        "fosghen",
        "--payload",
        "trace.json",
        "--dry-run",
    ];

    assert_eq!(
        parse_publish_args(&arguments).unwrap(),
        PublishArgs {
            repository: "fosghen/project".to_owned(),
            pull_request: 42,
            owner: "fosghen".to_owned(),
            payload: "trace.json".to_owned(),
            dry_run: true,
        }
    );
}

#[test]
fn publish_arguments_reject_missing_or_invalid_values() {
    assert!(parse_publish_args(&["--repo", "owner/repo"]).is_err());
    assert!(parse_publish_args(&["--pr", "zero"]).is_err());
    assert!(parse_publish_args(&["--unknown"]).is_err());
    assert!(complete_arguments("invalid", "1", "owner").is_err());
    assert!(complete_arguments("owner/repo", "0", "owner").is_err());
    assert!(complete_arguments("owner/repo", "1", " ").is_err());
}

fn complete_arguments(
    repo: &str,
    pr: &str,
    owner: &str,
) -> Result<PublishArgs, agent_delivery_telemetry::cli::ParseError> {
    parse_publish_args(&[
        "--repo",
        repo,
        "--pr",
        pr,
        "--owner",
        owner,
        "--payload",
        "trace.json",
    ])
}
