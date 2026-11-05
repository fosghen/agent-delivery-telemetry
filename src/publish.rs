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
