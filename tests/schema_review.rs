use agent_delivery_telemetry::schema::{
    Review, ReviewSpan, ReviewTrigger, Timestamp, Trace, TraceId, TraceOutcome,
};

#[test]
fn review_is_optional_and_conditional() {
    let untouched = Trace::new(TraceId::try_from("trace-10").unwrap());
    assert_eq!(untouched.review, Review::NotRequested);

    let mut reviewed = Trace::new(TraceId::try_from("trace-11").unwrap());
    reviewed.review = Review::Requested {
        trigger: ReviewTrigger::CorrectnessUncertainty,
        span: Some(ReviewSpan {
            requested_at: Timestamp(20),
            completed_at: None,
        }),
    };
    reviewed.outcome = Some(TraceOutcome::Abandoned);

    assert!(matches!(reviewed.review, Review::Requested { .. }));
}
