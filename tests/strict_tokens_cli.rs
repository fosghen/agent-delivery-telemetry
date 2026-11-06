use std::{fs, process::Command};

use agent_delivery_telemetry::draft::WorkDraft;

const SESSION: &str = r#"{"payload":{"type":"token_count","info":{"total_token_usage":{"input_tokens":100,"cached_input_tokens":40,"output_tokens":20,"reasoning_output_tokens":5,"total_tokens":120}}}}"#;

#[test]
fn strict_begin_requires_a_valid_session_source_before_writing_draft() {
    let directory = std::env::temp_dir().join(format!("adt-strict-{}", std::process::id()));
    fs::create_dir_all(&directory).unwrap();
    let draft_path = directory.join("draft.json");
    let session_path = directory.join("session.jsonl");
    let binary = env!("CARGO_BIN_EXE_agent-delivery-telemetry");

    let rejected = Command::new(binary)
        .args(["begin", "--require-tokens", "fosghen/project", "80"])
        .arg(&draft_path)
        .output()
        .unwrap();
    assert_eq!(rejected.status.code(), Some(12));
    assert!(!draft_path.exists());

    fs::write(&session_path, SESSION).unwrap();
    let accepted = Command::new(binary)
        .args(["begin", "--require-tokens", "fosghen/project", "80"])
        .arg(&draft_path)
        .arg(&session_path)
        .output()
        .unwrap();
    assert!(accepted.status.success());
    let draft: WorkDraft = serde_json::from_str(&fs::read_to_string(&draft_path).unwrap()).unwrap();
    assert!(draft.require_tokens);
    assert_eq!(draft.session_log.as_deref(), session_path.to_str());

    let payload_path = directory.join("payload.json");
    let unfinished = Command::new(binary)
        .args(["finish", "fosghen/project"])
        .arg(&draft_path)
        .args(["abc", "1"])
        .arg(&payload_path)
        .output()
        .unwrap();
    assert_eq!(unfinished.status.code(), Some(12));
    assert!(!payload_path.exists());
    let draft: WorkDraft = serde_json::from_str(&fs::read_to_string(&draft_path).unwrap()).unwrap();
    assert_eq!(draft.finished_at, None);
    fs::remove_dir_all(directory).unwrap();
}
