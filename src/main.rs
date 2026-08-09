#![forbid(unsafe_code)]

use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{CommandFactory, Parser, Subcommand, error::ErrorKind};
use reuse_evidence::{
    EnrollmentEffect, ExitMeaning, Visibility, enroll_with_expected_repository_id, set_visibility,
};
use uuid::Uuid;

mod portfolio;

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
    /// Rescan configured roots and report marker-enrolled repositories.
    Portfolio {
        /// Portfolio root for this run; overrides user-local configuration.
        #[arg(long)]
        root: Vec<PathBuf>,
    },
    /// Govern this repository's installed agent skills.
    Skills(skill_evidence::cli::SkillsArgs),
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
        Some(Command::Portfolio { root }) => run_portfolio(&root),
        Some(Command::Skills(args)) => return run_skills(args),
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

fn run_skills(args: skill_evidence::cli::SkillsArgs) -> ExitCode {
    let mut out = io::stdout().lock();
    let mut err = io::stderr().lock();
    let exit = skill_evidence::cli::run(args, &skill_evidence_host(), &mut out, &mut err);
    out.flush().ok();
    err.flush().ok();
    ExitCode::from(skill_exit_meaning(exit).status())
}

fn skill_evidence_host() -> skill_evidence::Host {
    skill_evidence::Host {
        namespace: "reuse-evidence".to_owned(),
        command: "reuse-evidence".to_owned(),
        cargo_package: "reuse-evidence".to_owned(),
        skills_directory: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".claude/skills"),
    }
}

const fn skill_exit_meaning(exit: skill_evidence::cli::Exit) -> ExitMeaning {
    match exit {
        skill_evidence::cli::Exit::Success => ExitMeaning::Success,
        skill_evidence::cli::Exit::UnsafeFailure => ExitMeaning::UnsafeFailure,
        skill_evidence::cli::Exit::Refusal => ExitMeaning::Refusal,
    }
}

fn run_portfolio(roots: &[PathBuf]) -> Result<(), (ExitMeaning, String)> {
    let report = portfolio::report(roots).map_err(|message| (ExitMeaning::Refusal, message))?;
    match report {
        portfolio::PortfolioReport::Complete(report) => {
            print!("{report}");
            Ok(())
        }
        portfolio::PortfolioReport::IdentityConflict(report) => Err((
            ExitMeaning::Refusal,
            format!(
                "duplicate repository identities make the portfolio ambiguous\n{report}resolution: restore a unique stable repository identity for every enrolled repository before rerunning the report"
            ),
        )),
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
