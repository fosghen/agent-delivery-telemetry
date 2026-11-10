use std::{fs, os::unix::fs::PermissionsExt, process::Command};

use agent_delivery_telemetry::draft::WorkDraft;

#[test]
fn foreign_revision_does_not_consume_draft_or_write_payload() {
    let directory = std::env::temp_dir().join(format!("adt-revision-{}", std::process::id()));
    fs::create_dir_all(&directory).unwrap();
    let gh = directory.join("gh");
    fs::write(&gh, "#!/bin/sh\nprintf 'foreign\\n'\n").unwrap();
    let mut permissions = fs::metadata(&gh).unwrap().permissions();
    permissions.set_mode(0o700);
    fs::set_permissions(&gh, permissions).unwrap();

    let draft_path = directory.join("draft.json");
    let payload_path = directory.join("payload.json");
    let draft = WorkDraft::begin("fosghen/project", 82, "2026-11-10T00:00:00Z").unwrap();
    fs::write(&draft_path, serde_json::to_string(&draft).unwrap()).unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_agent-delivery-telemetry"))
        .env("PATH", &directory)
        .args(["finish", "fosghen/project"])
        .arg(&draft_path)
        .args(["target", "1"])
        .arg(&payload_path)
        .output()
        .unwrap();

    assert_eq!(result.status.code(), Some(13));
    assert!(!payload_path.exists());
    let unchanged: WorkDraft =
        serde_json::from_str(&fs::read_to_string(&draft_path).unwrap()).unwrap();
    assert_eq!(unchanged.finished_at, None);
    fs::remove_dir_all(directory).unwrap();
}
