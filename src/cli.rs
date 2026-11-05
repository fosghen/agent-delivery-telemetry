#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PublishArgs {
    pub repository: String,
    pub pull_request: u64,
    pub owner: String,
    pub payload: String,
    pub dry_run: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ParseError;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    Create,
    Update,
    Unchanged,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct CliOutcome {
    pub action: Action,
    pub id: Option<u64>,
    pub dry_run: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CliError<E> {
    Input,
    Conflict,
    Transport(E),
}

/// # Errors
///
/// Returns an error for unknown, missing, or invalid arguments.
pub fn parse_publish_args(arguments: &[&str]) -> Result<PublishArgs, ParseError> {
    let mut repository = None;
    let mut pull_request = None;
    let mut owner = None;
    let mut payload = None;
    let mut dry_run = false;
    let mut index = 0;
    while index < arguments.len() {
        let target = match arguments[index] {
            "--dry-run" => {
                dry_run = true;
                index += 1;
                continue;
            }
            "--repo" => &mut repository,
            "--owner" => &mut owner,
            "--payload" => &mut payload,
            "--pr" => {
                index += 1;
                pull_request = Some(
                    arguments
                        .get(index)
                        .ok_or(ParseError)?
                        .parse()
                        .map_err(|_| ParseError)?,
                );
                index += 1;
                continue;
            }
            _ => return Err(ParseError),
        };
        index += 1;
        *target = Some((*arguments.get(index).ok_or(ParseError)?).to_owned());
        index += 1;
    }
    let repository = repository.ok_or(ParseError)?;
    let pull_request = pull_request.ok_or(ParseError)?;
    let owner = owner.ok_or(ParseError)?;
    let payload = payload.ok_or(ParseError)?;
    let valid_repository = repository
        .split_once('/')
        .is_some_and(|(account, name)| !account.is_empty() && !name.is_empty());
    if !valid_repository
        || pull_request == 0
        || owner.trim().is_empty()
        || payload.trim().is_empty()
    {
        return Err(ParseError);
    }
    Ok(PublishArgs {
        repository,
        pull_request,
        owner,
        payload,
        dry_run,
    })
}
use crate::comment::{TelemetryPayload, render_comment, validate_usage};
use crate::publish::{
    CommentPort, PublishError, PublishOutcome, UpsertDecision, decide_upsert, publish,
};
use serde::Serialize;

/// # Errors
/// Returns an input, comment-selection, or provider transport error.
pub fn run<P: CommentPort>(
    port: &mut P,
    owner: &str,
    payload: &str,
    dry_run: bool,
) -> Result<CliOutcome, CliError<P::Error>> {
    let payload: TelemetryPayload = serde_json::from_str(payload).map_err(|_| CliError::Input)?;
    validate_usage(&payload.usage).map_err(|_| CliError::Input)?;
    let desired = render_comment(&payload).map_err(|_| CliError::Input)?;
    if dry_run {
        let comments = port.list().map_err(CliError::Transport)?;
        let decision = decide_upsert(&comments, owner, &desired).map_err(|_| CliError::Conflict)?;
        return Ok(match decision {
            UpsertDecision::Create { .. } => CliOutcome {
                action: Action::Create,
                id: None,
                dry_run,
            },
            UpsertDecision::Update { id, .. } => CliOutcome {
                action: Action::Update,
                id: Some(id),
                dry_run,
            },
            UpsertDecision::Unchanged { id } => CliOutcome {
                action: Action::Unchanged,
                id: Some(id),
                dry_run,
            },
        });
    }
    publish(port, owner, &desired)
        .map(|outcome| match outcome {
            PublishOutcome::Created { id } => (Action::Create, id),
            PublishOutcome::Updated { id } => (Action::Update, id),
            PublishOutcome::Unchanged { id } => (Action::Unchanged, id),
        })
        .map(|(action, id)| CliOutcome {
            action,
            id: Some(id),
            dry_run,
        })
        .map_err(|error| match error {
            PublishError::Selection(_) => CliError::Conflict,
            PublishError::Transport(error) => CliError::Transport(error),
        })
}
