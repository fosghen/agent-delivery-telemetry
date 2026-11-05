use crate::comment::COMMENT_MARKER;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommentSnapshot {
    pub id: u64,
    pub author: String,
    pub body: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UpsertDecision {
    Create { body: String },
    Update { id: u64, body: String },
    Unchanged { id: u64 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UpsertError {
    MultipleEligibleComments,
}

pub trait CommentPort {
    type Error;

    /// # Errors
    ///
    /// Returns a provider-specific transport error.
    fn list(&mut self) -> Result<Vec<CommentSnapshot>, Self::Error>;
    /// # Errors
    ///
    /// Returns a provider-specific transport error.
    fn create(&mut self, body: &str) -> Result<u64, Self::Error>;
    /// # Errors
    ///
    /// Returns a provider-specific transport error.
    fn update(&mut self, id: u64, body: &str) -> Result<(), Self::Error>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PublishOutcome {
    Created { id: u64 },
    Updated { id: u64 },
    Unchanged { id: u64 },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PublishError<E> {
    Transport(E),
    Selection(UpsertError),
}

/// # Errors
///
/// Returns an error when more than one owned telemetry comment exists.
pub fn decide_upsert(
    comments: &[CommentSnapshot],
    owner: &str,
    desired: &str,
) -> Result<UpsertDecision, UpsertError> {
    let eligible: Vec<_> = comments
        .iter()
        .filter(|comment| comment.author == owner && comment.body.contains(COMMENT_MARKER))
        .collect();
    match eligible.as_slice() {
        [] => Ok(UpsertDecision::Create {
            body: desired.to_owned(),
        }),
        [comment] if comment.body == desired => Ok(UpsertDecision::Unchanged { id: comment.id }),
        [comment] => Ok(UpsertDecision::Update {
            id: comment.id,
            body: desired.to_owned(),
        }),
        _ => Err(UpsertError::MultipleEligibleComments),
    }
}

/// # Errors
///
/// Returns a selection conflict or provider-specific transport error.
pub fn publish<P: CommentPort>(
    port: &mut P,
    owner: &str,
    desired: &str,
) -> Result<PublishOutcome, PublishError<P::Error>> {
    let comments = port.list().map_err(PublishError::Transport)?;
    let decision = decide_upsert(&comments, owner, desired).map_err(PublishError::Selection)?;
    match decision {
        UpsertDecision::Create { body } => {
            let id = port.create(&body).map_err(PublishError::Transport)?;
            Ok(PublishOutcome::Created { id })
        }
        UpsertDecision::Update { id, body } => {
            port.update(id, &body).map_err(PublishError::Transport)?;
            Ok(PublishOutcome::Updated { id })
        }
        UpsertDecision::Unchanged { id } => Ok(PublishOutcome::Unchanged { id }),
    }
}
