use std::{fs, process::Command};

use agent_delivery_telemetry::comment::{ReviewRecord, TelemetryPayload};

const PAYLOAD: &str = r#"{
  "schema_version": 1,
  "head_sha": "abc",
  "work_started_at": "2026-11-06T00:00:00Z",
  "attempts": [{"number": 1, "outcome": "succeeded"}],
  "stages": [],
  "verification": {"outcome": "passed", "checks": 2},
  "review": {"decision": "not_requested"},
  "usage": []
}"#;

const ARTIFACT: &str = r#"{
  "actor": "primary", "provider": "openai", "model": null,
  "stage": null, "attempt": null, "precision": "exact",
  "input_tokens": 3, "cached_input_tokens": 1, "output_tokens": 2,
  "reasoning_tokens": 1, "total_tokens": 5
}"#;

#[test]
fn cli_binds_once_and_does_not_rewrite_on_duplicate() {
    let directory = std::env::temp_dir().join(format!("adt-bind-{}", std::process::id()));
    fs::create_dir_all(&directory).unwrap();
    let payload_path = directory.join("payload.json");
    let artifact_path = directory.join("usage.json");
    fs::write(&payload_path, PAYLOAD).unwrap();
    fs::write(&artifact_path, ARTIFACT).unwrap();
    let invoke = || {
        Command::new(env!("CARGO_BIN_EXE_agent-delivery-telemetry"))
            .arg("bind-codex-usage")
            .arg(&payload_path)
            .arg(&artifact_path)
            .args(["subagent", "implementation", "1"])
            .output()
            .unwrap()
    };

    assert!(invoke().status.success());
    let contents = fs::read_to_string(&payload_path).unwrap();
    let payload: TelemetryPayload = serde_json::from_str(&contents).unwrap();
    assert_eq!(payload.head_sha, "abc");
    assert_eq!(payload.attempts[0].outcome, "succeeded");
    assert_eq!(payload.usage[0].actor, "subagent");
    assert_eq!(payload.usage[0].stage.as_deref(), Some("implementation"));
    assert_eq!(payload.usage[0].attempt, Some(1));

    assert!(!invoke().status.success());
    assert_eq!(fs::read_to_string(&payload_path).unwrap(), contents);
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn cli_records_a_completed_review_without_inference() {
    let directory = std::env::temp_dir().join(format!("adt-review-{}", std::process::id()));
    fs::create_dir_all(&directory).unwrap();
    let payload_path = directory.join("payload.json");
    fs::write(&payload_path, PAYLOAD).unwrap();
    let invoke = |completed: &str| {
        Command::new(env!("CARGO_BIN_EXE_agent-delivery-telemetry"))
            .arg("record-review")
            .arg(&payload_path)
            .args([
                "correctness_uncertainty",
                "2026-11-06T00:01:00Z",
                completed,
                "no_findings",
                "0",
                "0",
            ])
            .output()
            .unwrap()
    };

    assert!(!invoke("2026-11-05T00:02:00Z").status.success());
    assert!(invoke("2026-11-06T00:02:00Z").status.success());
    let payload: TelemetryPayload =
        serde_json::from_str(&fs::read_to_string(&payload_path).unwrap()).unwrap();
    assert!(matches!(payload.review, ReviewRecord::Requested { .. }));
    assert_eq!(payload.usage.len(), 0);
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn cli_records_failure_before_usage_is_bound() {
    let directory = std::env::temp_dir().join(format!("adt-attempt-{}", std::process::id()));
    fs::create_dir_all(&directory).unwrap();
    let payload_path = directory.join("payload.json");
    fs::write(&payload_path, PAYLOAD).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_agent-delivery-telemetry"))
        .arg("record-attempt-failure")
        .arg(&payload_path)
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(
        String::from_utf8(output.stdout)
            .unwrap()
            .contains(r#""attempt":1"#)
    );
    let payload: TelemetryPayload =
        serde_json::from_str(&fs::read_to_string(&payload_path).unwrap()).unwrap();
    assert_eq!(payload.attempts[0].outcome, "failed");
    assert_eq!(payload.attempts[1].outcome, "succeeded");
    assert!(payload.usage.is_empty());
    fs::remove_dir_all(directory).unwrap();
}
