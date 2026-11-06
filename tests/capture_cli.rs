use std::{
    fs,
    io::Write,
    process::{Command, Stdio},
};

use agent_delivery_telemetry::comment::UsageRecord;

const EVENT: &[u8] = br#"{"type":"turn.completed","usage":{"input_tokens":3,"cached_input_tokens":1,"output_tokens":2,"reasoning_output_tokens":1}}"#;

fn capture(name: &str, provenance: &[&str]) -> UsageRecord {
    let output = std::env::temp_dir().join(format!("adt-capture-{name}.json"));
    let mut command = Command::new(env!("CARGO_BIN_EXE_agent-delivery-telemetry"));
    let mut child = command
        .arg("capture-codex-usage")
        .arg(&output)
        .args(provenance)
        .stdin(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(EVENT).unwrap();
    assert!(child.wait().unwrap().success());
    let usage = serde_json::from_str(&fs::read_to_string(&output).unwrap()).unwrap();
    fs::remove_file(output).unwrap();
    usage
}

#[test]
fn capture_cli_records_explicit_model_provenance() {
    let usage = capture("explicit", &["gpt-6-luna", "medium"]);
    assert_eq!(usage.model.as_deref(), Some("gpt-6-luna"));
    assert_eq!(usage.reasoning_effort.as_deref(), Some("medium"));
}

#[test]
fn capture_cli_preserves_legacy_unattributed_mode() {
    let usage = capture("legacy", &[]);
    assert_eq!(usage.model, None);
    assert_eq!(usage.reasoning_effort, None);
}
