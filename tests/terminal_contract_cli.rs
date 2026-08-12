//! Terminal-contract coverage for a stdout that cannot be written.
//!
//! ADR 0016 gives the CLI suites the exit status each `ExitMeaning` maps to.
//! `print!` panics when stdout rejects a write, which exits 101 — a status
//! outside the contract, produced without consulting `ExitMeaning` at all.
//! This asserts that the binary classifies the failure instead.
//!
//! Two neighbouring cases are deliberately not tested here. A closed
//! descriptor is unreachable: the Rust runtime reopens a closed fd 1 on
//! `/dev/null` at startup, so the write succeeds. A consumer that stops
//! reading, as `… | head` does, ends the run successfully, but `--help` fits in
//! the pipe buffer, so whether the write reaches a closed read end is a race
//! rather than a fact.

#![cfg(unix)]

use std::fs::OpenOptions;
use std::process::{Command, Stdio};

#[test]
fn an_unwritable_stdout_is_an_unsafe_failure_rather_than_a_panic() {
    // `/dev/full` accepts an open and rejects every write with ENOSPC, which is
    // the redirect-to-a-full-disk case without needing a full disk.
    if OpenOptions::new().write(true).open("/dev/full").is_err() {
        return;
    }
    let binary = env!("CARGO_BIN_EXE_reuse-evidence");
    let output = Command::new("sh")
        .args([
            "-c",
            "exec \"$1\" --help > /dev/full",
            "reuse-evidence-stdout-test",
            binary,
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .expect("the compiled binary should start under a shell");
    let stderr = String::from_utf8(output.stderr).expect("stderr should be UTF-8");

    assert_eq!(
        output.status.code(),
        Some(1),
        "an unwritable stdout is an unsafe failure; stderr was: {stderr}"
    );
    assert!(
        stderr.starts_with("unsafe failure: command output could not be written to stdout: "),
        "{stderr}"
    );
    assert!(
        !stderr.contains("panicked"),
        "the terminal contract must not be bypassed by a panic: {stderr}"
    );
}
