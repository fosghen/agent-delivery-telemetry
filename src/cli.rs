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
