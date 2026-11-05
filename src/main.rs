use std::process::ExitCode;

use agent_delivery_telemetry::cli::{CliError, parse_collect_args, parse_publish_args, run};
use agent_delivery_telemetry::github::{GhCommentPort, SystemGh};
use agent_delivery_telemetry::reconcile::{ReconcileError, reconcile};

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
        Some("collect") => execute_collect(&values),
        _ => Err(("input", 2)),
    }
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
    let options = parse_collect_args(values).map_err(|_| ("input", 2))?;
    let mut port = GhCommentPort::new(SystemGh, options.repository, options.pull_request);
    let evidence = port.evidence_snapshot().map_err(|_| ("transport", 4))?;
    let comments = port.list_comments().map_err(|_| ("transport", 4))?;
    let trace = reconcile(evidence, &comments, &options.owner).map_err(|error| match error {
        ReconcileError::MissingTelemetry => ("missing_telemetry", 5),
        ReconcileError::DuplicateTelemetry => ("duplicate_telemetry", 6),
        ReconcileError::InvalidTelemetry(_) => ("invalid_telemetry", 7),
        ReconcileError::StaleTelemetry => ("stale_telemetry", 8),
    })?;
    println!(
        "{}",
        serde_json::to_string(&trace).map_err(|_| ("input", 2))?
    );
    Ok(())
}
