use std::process::ExitCode;

use agent_delivery_telemetry::cli::{CliError, parse_publish_args, run};
use agent_delivery_telemetry::github::{GhCommentPort, SystemGh};

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
    if arguments.first().map(String::as_str) != Some("publish") {
        return Err(("input", 2));
    }
    let values: Vec<&str> = arguments[1..].iter().map(String::as_str).collect();
    let options = parse_publish_args(&values).map_err(|_| ("input", 2))?;
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
