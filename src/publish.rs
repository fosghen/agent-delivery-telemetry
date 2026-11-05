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
    if comments
        .iter()
        .any(|comment| comment.author == owner && comment.body.contains(COMMENT_MARKER))
    {
        Err(UpsertError::MultipleEligibleComments)
    } else {
        Ok(UpsertDecision::Create {
            body: desired.to_owned(),
        })
    }
}
