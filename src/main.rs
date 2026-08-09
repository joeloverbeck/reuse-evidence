#![forbid(unsafe_code)]

use std::path::Path;
use std::process::ExitCode;

use clap::{CommandFactory, Parser, Subcommand, error::ErrorKind};
use reuse_evidence::{
    EnrollmentEffect, ExitMeaning, Visibility, enroll_with_expected_repository_id, set_visibility,
};
use uuid::Uuid;

#[derive(Debug, Parser)]
#[command(name = "reuse-evidence")]
#[command(about = "Evidence-gated reuse decisions for repository portfolios")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Enroll the repository containing the current directory.
    Enroll {
        /// Label used to group this repository in portfolio reports.
        #[arg(long)]
        ecosystem_id: Option<String>,
        /// Declared repository visibility: public or private.
        #[arg(long)]
        visibility: Option<String>,
        /// Existing repository identity to verify; never assigned to a new enrollment.
        #[arg(long)]
        expected_repository_id: Option<String>,
    },
    /// Deliberately change an enrolled repository's declared visibility.
    SetVisibility {
        /// New declared repository visibility: public or private.
        #[arg(long)]
        visibility: Option<String>,
    },
}

fn main() -> ExitCode {
    match Cli::try_parse() {
        Ok(cli) => run(cli),
        Err(error)
            if matches!(
                error.kind(),
                ErrorKind::DisplayHelp | ErrorKind::DisplayVersion
            ) =>
        {
            print!("{error}");
            ExitCode::from(ExitMeaning::Success.status())
        }
        Err(error) => {
            eprintln!(
                "refusal: invalid command line\ncondition: {}\nresolution: rerun with a command and arguments shown in the usage",
                error.to_string().trim_end()
            );
            ExitCode::from(ExitMeaning::Refusal.status())
        }
    }
}

fn run(cli: Cli) -> ExitCode {
    let result = match cli.command {
        Some(Command::Enroll {
            ecosystem_id,
            visibility,
            expected_repository_id,
        }) => run_enroll(ecosystem_id, visibility, expected_repository_id),
        Some(Command::SetVisibility { visibility }) => run_set_visibility(visibility),
        None => {
            let mut command = Cli::command();
            let usage = command.render_usage();
            Err((
                ExitMeaning::Refusal,
                format!(
                    "no command was supplied\nresolution: rerun with a command shown in `{usage}`"
                ),
            ))
        }
    };

    match result {
        Ok(()) => ExitCode::from(ExitMeaning::Success.status()),
        Err((meaning, message)) => {
            eprintln!("{}: {message}", terminal_name(meaning));
            ExitCode::from(meaning.status())
        }
    }
}

fn run_enroll(
    ecosystem_id: Option<String>,
    visibility: Option<String>,
    expected_repository_id: Option<String>,
) -> Result<(), (ExitMeaning, String)> {
    let ecosystem_id = ecosystem_id.ok_or_else(|| {
        (
            ExitMeaning::Refusal,
            "missing required `--ecosystem-id`\nresolution: rerun with `--ecosystem-id <IDENTITY>`"
                .to_owned(),
        )
    })?;
    let visibility = visibility.ok_or_else(|| {
        (
            ExitMeaning::Refusal,
            "missing required `--visibility`\nresolution: rerun with `--visibility public` or `--visibility private`"
                .to_owned(),
        )
    })?;
    let visibility =
        Visibility::parse(&visibility).map_err(|error| (error.meaning(), error.to_string()))?;
    let expected_repository_id = expected_repository_id
        .map(|value| {
            Uuid::parse_str(&value).map_err(|error| {
                (
                    ExitMeaning::Refusal,
                    format!(
                        "expected repository identity `{value}` is invalid: {error}\nresolution: use the UUID recorded in the existing marker or omit the identity guard"
                    ),
                )
            })
        })
        .transpose()?;
    let enrollment = enroll_with_expected_repository_id(
        Path::new("."),
        &ecosystem_id,
        visibility,
        expected_repository_id,
    )
    .map_err(|error| (error.meaning(), error.to_string()))?;

    report_enrollment(&enrollment);
    Ok(())
}

fn run_set_visibility(visibility: Option<String>) -> Result<(), (ExitMeaning, String)> {
    let visibility = visibility.ok_or_else(|| {
        (
            ExitMeaning::Refusal,
            "missing required `--visibility`\nresolution: rerun with `--visibility public` or `--visibility private`"
                .to_owned(),
        )
    })?;
    let visibility =
        Visibility::parse(&visibility).map_err(|error| (error.meaning(), error.to_string()))?;
    let enrollment = set_visibility(Path::new("."), visibility)
        .map_err(|error| (error.meaning(), error.to_string()))?;
    report_enrollment(&enrollment);
    Ok(())
}

fn report_enrollment(enrollment: &reuse_evidence::Enrollment) {
    match enrollment.effect {
        EnrollmentEffect::Created => println!("enrolled repository"),
        EnrollmentEffect::Existing => println!("existing enrollment"),
        EnrollmentEffect::VisibilityChanged => println!("changed repository visibility"),
        EnrollmentEffect::VisibilityUnchanged => println!("repository visibility unchanged"),
    }
    println!("marker: {}", enrollment.marker_path.display());
    println!("repository_id: {}", enrollment.repository_id);
    println!("ecosystem_id: {}", enrollment.ecosystem_id);
    println!("visibility: {}", enrollment.visibility);
}

const fn terminal_name(meaning: ExitMeaning) -> &'static str {
    match meaning {
        ExitMeaning::Success => "success",
        ExitMeaning::UnsafeFailure => "unsafe failure",
        ExitMeaning::Refusal => "refusal",
    }
}
