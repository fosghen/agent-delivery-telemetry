use std::process::ExitCode;
use std::{fs::OpenOptions, io::Write};

use agent_delivery_telemetry::cli::{CliError, parse_collect_args, parse_publish_args, run};
use agent_delivery_telemetry::codex_usage::{
    UsageCaptureError, latest_snapshot, unavailable_usage,
};
use agent_delivery_telemetry::comment::TelemetryPayload;
use agent_delivery_telemetry::draft::WorkDraft;
use agent_delivery_telemetry::github::{GhCommentPort, SystemGh};
use agent_delivery_telemetry::reconcile::{ReconcileError, ReconciledTrace, reconcile};
use agent_delivery_telemetry::report::build_report;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

fn main() -> ExitCode {
    match execute() {
        Ok(()) => ExitCode::SUCCESS,
        Err((kind, code)) => {
            println!(r#"{{"error":"{kind}"}}"#);
            ExitCode::from(code)
        }
    }
}

fn execute() -> Result<(), (&'static str, u8)> {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let values: Vec<&str> = arguments
        .get(1..)
        .unwrap_or_default()
        .iter()
        .map(String::as_str)
        .collect();
    match arguments.first().map(String::as_str) {
        Some("publish") => execute_publish(&values),
        Some("begin") => execute_begin(&values),
        Some("finish") => execute_finish(&values),
        Some("refresh-tokens") => execute_refresh_tokens(&values),
        Some("collect") => execute_collect(&values),
        Some("report") => execute_report(&values),
        _ => Err(("input", 2)),
    }
}

fn execute_refresh_tokens(values: &[&str]) -> Result<(), (&'static str, u8)> {
    let [draft_path, payload_path] = values else {
        return Err(("input", 2));
    };
    let draft_json = std::fs::read_to_string(draft_path).map_err(|_| ("io", 11))?;
    let draft: WorkDraft = serde_json::from_str(&draft_json).map_err(|_| ("input", 2))?;
    let payload_json = std::fs::read_to_string(payload_path).map_err(|_| ("io", 11))?;
    let mut payload: TelemetryPayload =
        serde_json::from_str(&payload_json).map_err(|_| ("input", 2))?;
    let session_path = draft.session_log.as_deref().ok_or(("tokens", 12))?;
    let session = std::fs::read_to_string(session_path).map_err(|_| ("io", 11))?;
    draft
        .refresh_tokens(&mut payload, &session)
        .map_err(|_| ("tokens", 12))?;
    let json = serde_json::to_string_pretty(&payload).map_err(|_| ("input", 2))?;
    std::fs::write(payload_path, json).map_err(|_| ("io", 11))?;
    println!(r#"{{"action":"tokens_refreshed"}}"#);
    Ok(())
}

fn now() -> Result<String, (&'static str, u8)> {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|_| ("clock", 10))
}

fn execute_begin(values: &[&str]) -> Result<(), (&'static str, u8)> {
    let (repository, issue, output, session_log) = match values {
        [repository, issue, output] => (*repository, *issue, *output, None),
        [repository, issue, output, session] => (*repository, *issue, *output, Some(*session)),
        _ => return Err(("input", 2)),
    };
    let issue = issue.parse().map_err(|_| ("input", 2))?;
    let mut draft = WorkDraft::begin(repository, issue, &now()?).map_err(|_| ("input", 2))?;
    if let Some(path) = session_log {
        let contents = std::fs::read_to_string(path).map_err(|_| ("io", 11))?;
        let baseline = latest_snapshot(&contents).map_err(|_| ("tokens", 12))?;
        draft.attach_token_source(path, baseline);
    }
    let json = serde_json::to_string_pretty(&draft).map_err(|_| ("input", 2))?;
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)
        .and_then(|mut file| file.write_all(json.as_bytes()))
        .map_err(|_| ("io", 11))?;
    println!(r#"{{"action":"begun","issue":{issue}}}"#);
    Ok(())
}

fn execute_finish(values: &[&str]) -> Result<(), (&'static str, u8)> {
    let [repository, draft_path, head_sha, checks, output] = values else {
        return Err(("input", 2));
    };
    let source = std::fs::read_to_string(draft_path).map_err(|_| ("io", 11))?;
    let mut draft: WorkDraft = serde_json::from_str(&source).map_err(|_| ("input", 2))?;
    let mut payload = draft
        .finish(
            repository,
            head_sha,
            checks.parse().map_err(|_| ("input", 2))?,
            &now()?,
        )
        .map_err(|_| ("input", 2))?;
    let token_result = match draft.session_log.as_deref() {
        Some(path) => match std::fs::read_to_string(path) {
            Ok(contents) => draft
                .token_usage(&contents)
                .map_err(UsageCaptureError::as_str),
            Err(_) => Err("unreadable"),
        },
        None => Err("not_configured"),
    };
    let token_status = match token_result {
        Ok(usage) => {
            payload.usage.push(usage);
            "session_total"
        }
        Err(reason) => {
            payload.usage.push(unavailable_usage());
            reason
        }
    };
    let json = serde_json::to_string_pretty(&payload).map_err(|_| ("input", 2))?;
    std::fs::write(output, json).map_err(|_| ("io", 11))?;
    let draft_json = serde_json::to_string_pretty(&draft).map_err(|_| ("input", 2))?;
    std::fs::write(draft_path, draft_json).map_err(|_| ("io", 11))?;
    println!(r#"{{"action":"finished","tokens":"{token_status}"}}"#);
    Ok(())
}

fn execute_publish(values: &[&str]) -> Result<(), (&'static str, u8)> {
    let options = parse_publish_args(values).map_err(|_| ("input", 2))?;
    let payload = std::fs::read_to_string(&options.payload).map_err(|_| ("input", 2))?;
    let mut port = GhCommentPort::new(SystemGh, options.repository, options.pull_request);
    let outcome =
        run(&mut port, &options.owner, &payload, options.dry_run).map_err(|error| match error {
            CliError::Input => ("input", 2),
            CliError::Conflict => ("conflict", 3),
            CliError::Transport(_) => ("transport", 4),
        })?;
    let output = serde_json::to_string(&outcome).map_err(|_| ("input", 2))?;
    println!("{output}");
    Ok(())
}

fn execute_collect(values: &[&str]) -> Result<(), (&'static str, u8)> {
    let trace = collect_trace(values)?;
    println!(
        "{}",
        serde_json::to_string(&trace).map_err(|_| ("input", 2))?
    );
    Ok(())
}

fn execute_report(values: &[&str]) -> Result<(), (&'static str, u8)> {
    let report = build_report(vec![collect_trace(values)?]).map_err(|_| ("metrics", 9))?;
    println!(
        "{}",
        serde_json::to_string(&report).map_err(|_| ("input", 2))?
    );
    Ok(())
}

fn collect_trace(values: &[&str]) -> Result<ReconciledTrace, (&'static str, u8)> {
    let options = parse_collect_args(values).map_err(|_| ("input", 2))?;
    let mut port = GhCommentPort::new(SystemGh, options.repository, options.pull_request);
    let evidence = port.evidence_snapshot().map_err(|_| ("transport", 4))?;
    let comments = port.list_comments().map_err(|_| ("transport", 4))?;
    reconcile(evidence, &comments, &options.owner).map_err(|error| match error {
        ReconcileError::MissingTelemetry => ("missing_telemetry", 5),
        ReconcileError::DuplicateTelemetry => ("duplicate_telemetry", 6),
        ReconcileError::InvalidTelemetry(_) => ("invalid_telemetry", 7),
        ReconcileError::StaleTelemetry => ("stale_telemetry", 8),
    })
}
