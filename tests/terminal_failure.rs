use reuse_evidence::{ExitMeaning, TerminalFailure};

#[test]
fn refusal_carries_exit_meaning_and_renders_condition_with_resolution() {
    let refusal = TerminalFailure::refusal(
        "required authority is absent",
        "obtain exact human acceptance",
    );

    assert_eq!(refusal.meaning(), ExitMeaning::Refusal);
    assert_eq!(
        refusal.to_string(),
        "refusal: required authority is absent\nresolution: obtain exact human acceptance"
    );
}

#[test]
fn unsafe_failure_carries_exit_meaning_and_renders_condition() {
    let failure = TerminalFailure::unsafe_failure("the write may be partial");

    assert_eq!(failure.meaning(), ExitMeaning::UnsafeFailure);
    assert_eq!(
        failure.to_string(),
        "unsafe failure: the write may be partial"
    );
}
