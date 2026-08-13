//! In-process coverage for resolving prepared-proposal staging.
//!
//! GitHub #29 and ADR 0016 assign the behavior to the public module interface.
//! The process boundary receives one separate test for the terminal facts newly
//! introduced by the CLI command.

mod support;

use std::fs;

use reuse_evidence::ExitMeaning;
use reuse_evidence::portfolio::{self, PortfolioLocation};
use support::TempRoot;

#[test]
fn staging_directory_resolves_beneath_user_local_state_without_writing() {
    let fixture = TempRoot::new("prepared-proposal-staging-success");
    let state_directory = fixture.join("state");
    let expected = state_directory
        .join("reuse-evidence")
        .join("prepared-proposals");
    let location =
        PortfolioLocation::from_user_directories(Vec::new(), None, Some(&state_directory));
    let before = support::snapshot(&fixture);

    let staging_directory = portfolio::prepared_proposal_staging_directory(&location)
        .expect("a resolved user-local state directory should name staging");

    assert_eq!(staging_directory, expected);
    assert!(
        !staging_directory.exists(),
        "resolving staging must not create the directory it names"
    );
    assert_eq!(
        support::snapshot(&fixture),
        before,
        "resolving staging must preserve every existing byte"
    );
}

#[test]
fn staging_directory_refuses_when_user_local_state_is_undeterminable_without_writing() {
    let fixture = TempRoot::new("prepared-proposal-staging-no-state-home");
    let location = PortfolioLocation::from_user_directories(Vec::new(), None, None);
    let before = support::snapshot(&fixture);

    let Err(failure) = portfolio::prepared_proposal_staging_directory(&location) else {
        panic!("an undeterminable user-local state directory must refuse");
    };

    assert_eq!(failure.meaning(), ExitMeaning::Refusal);
    assert_eq!(
        failure.to_string(),
        "refusal: the user-local state directory cannot be determined\nresolution: set `LOCALAPPDATA` on Windows, or `XDG_STATE_HOME` or `HOME` on Unix-like systems"
    );
    assert_eq!(
        support::snapshot(&fixture),
        before,
        "the state-directory refusal must preserve every existing byte"
    );
}

#[test]
fn staging_directory_refuses_inside_an_inspected_repository_without_writing() {
    let fixture = TempRoot::new("prepared-proposal-staging-inspected-repository");
    let portfolio_root = fixture.join("portfolio");
    fs::create_dir_all(&portfolio_root).expect("portfolio root should be creatable");
    let repository = support::git_repository(&portfolio_root, "repository")
        .canonicalize()
        .expect("repository path should be canonical");
    support::enrollment_marker(
        &repository,
        "00000000-0000-4000-8000-000000000029",
        "private",
    );
    let state_directory = repository.join("state");
    let staging_directory = state_directory
        .join("reuse-evidence")
        .join("prepared-proposals");
    let location = PortfolioLocation::from_user_directories(
        vec![portfolio_root],
        None,
        Some(&state_directory),
    );
    let before = support::snapshot(&fixture);

    let Err(failure) = portfolio::prepared_proposal_staging_directory(&location) else {
        panic!("staging inside an inspected repository must refuse");
    };

    assert_eq!(failure.meaning(), ExitMeaning::Refusal);
    assert_eq!(
        failure.to_string(),
        format!(
            "refusal: prepared-proposal staging directory `{}` would be stored inside inspected repository `{}`\nresolution: configure the platform state directory outside every inspected repository",
            staging_directory.display(),
            repository.display()
        )
    );
    assert!(
        !staging_directory.exists(),
        "the refusal must not create the staging directory"
    );
    assert_eq!(
        support::snapshot(&fixture),
        before,
        "the inspected-repository refusal must preserve every existing byte"
    );
}

#[test]
fn staging_directory_refuses_inside_an_uninspected_git_repository_without_writing() {
    let fixture = TempRoot::new("prepared-proposal-staging-uninspected-repository");
    let portfolio_root = fixture.join("portfolio");
    fs::create_dir_all(&portfolio_root).expect("portfolio root should be creatable");
    let enrolled_repository = support::git_repository(&portfolio_root, "enrolled");
    support::enrollment_marker(
        &enrolled_repository,
        "00000000-0000-4000-8000-000000000030",
        "private",
    );
    let unrelated_root = fixture.join("unrelated");
    fs::create_dir_all(&unrelated_root).expect("unrelated root should be creatable");
    let state_repository = support::git_repository(&unrelated_root, "state-repository")
        .canonicalize()
        .expect("state repository path should be canonical");
    let state_directory = state_repository.join("state");
    let staging_directory = state_directory
        .join("reuse-evidence")
        .join("prepared-proposals");
    let location = PortfolioLocation::from_user_directories(
        vec![portfolio_root],
        None,
        Some(&state_directory),
    );
    let before = support::snapshot(&fixture);

    let Err(failure) = portfolio::prepared_proposal_staging_directory(&location) else {
        panic!("staging inside any Git repository must refuse");
    };

    assert_eq!(failure.meaning(), ExitMeaning::Refusal);
    assert_eq!(
        failure.to_string(),
        format!(
            "refusal: prepared-proposal staging directory `{}` would be stored inside Git repository `{}`\nresolution: configure the platform state directory outside every Git repository",
            staging_directory.display(),
            state_repository.display()
        )
    );
    assert!(
        !staging_directory.exists(),
        "the refusal must not create the staging directory"
    );
    assert_eq!(
        support::snapshot(&fixture),
        before,
        "the Git-repository refusal must preserve every existing byte"
    );
}
