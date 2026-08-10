#![forbid(unsafe_code)]

use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{CommandFactory, Parser, Subcommand, error::ErrorKind};
use reuse_evidence::{
    EnrollmentEffect, ExitMeaning, TerminalFailure, Visibility, case,
    enroll_with_expected_repository_id, portfolio, set_visibility,
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
    /// Rescan configured roots and report marker-enrolled repositories.
    Portfolio {
        /// Portfolio root for this run; overrides user-local configuration.
        #[arg(long)]
        root: Vec<PathBuf>,
    },
    /// Record and inspect durable reuse cases.
    Case {
        #[command(subcommand)]
        command: CaseCommand,
    },
    /// Govern this repository's installed agent skills.
    Skills(skill_evidence::cli::SkillsArgs),
}

#[derive(Debug, Subcommand)]
enum CaseCommand {
    /// Open a case from a prepared two-occurrence proposal.
    Open {
        /// TOML proposal containing the case identity and occurrences.
        #[arg(long)]
        proposal: Option<PathBuf>,
        /// Portfolio root for participant resolution; overrides configuration.
        #[arg(long)]
        root: Vec<PathBuf>,
        /// Render the exact event and privacy consequence without writing.
        #[arg(long)]
        preview: bool,
    },
    /// Append one later occurrence against an expected case revision.
    Append {
        /// Opaque identity of the stewarded case to grow.
        case_id: String,
        /// Revision the caller believes the case currently records.
        #[arg(long)]
        expected_revision: Option<i64>,
        /// TOML proposal containing the occurrence to append.
        #[arg(long)]
        proposal: Option<PathBuf>,
        /// Portfolio root for participant resolution; overrides configuration.
        #[arg(long)]
        root: Vec<PathBuf>,
        /// Render the exact event and privacy consequence without writing.
        #[arg(long)]
        preview: bool,
    },
    /// List every case stewarded by the current repository.
    List {
        /// Portfolio root for current participant conditions; overrides configuration.
        #[arg(long)]
        root: Vec<PathBuf>,
    },
    /// Show one stewarded case and its complete recorded occurrences.
    Show {
        /// Opaque case identity to show.
        case_id: String,
        /// Portfolio root for current participant conditions; overrides configuration.
        #[arg(long)]
        root: Vec<PathBuf>,
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
        Err(error) => terminal_exit(Err(TerminalFailure::refusal(
            format!(
                "invalid command line\ncondition: {}",
                error.to_string().trim_end()
            ),
            "rerun with a command and arguments shown in the usage",
        ))),
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
        Some(Command::Case { command }) => run_case(command),
        Some(Command::Skills(args)) => return run_skills(args),
        None => {
            let mut command = Cli::command();
            let usage = command.render_usage();
            Err(TerminalFailure::refusal(
                "no command was supplied",
                format!("rerun with a command shown in `{usage}`"),
            ))
        }
    };

    terminal_exit(result)
}

fn run_case(command: CaseCommand) -> Result<(), TerminalFailure> {
    match command {
        CaseCommand::Open {
            proposal,
            root,
            preview,
        } => {
            let proposal = proposal.ok_or_else(|| {
                TerminalFailure::refusal(
                    "missing required `--proposal`",
                    "rerun with `case open --proposal <PATH>`",
                )
            })?;
            let outcome = case::open(Path::new("."), &proposal, &root, preview)?;
            print!("{}", outcome.render());
            Ok(())
        }
        CaseCommand::Append {
            case_id,
            expected_revision,
            proposal,
            root,
            preview,
        } => {
            let expected_revision = expected_revision.ok_or_else(|| {
                TerminalFailure::refusal(
                    "missing required `--expected-revision`",
                    "rerun with `case append <CASE_ID> --expected-revision <REVISION>`",
                )
            })?;
            let proposal = proposal.ok_or_else(|| {
                TerminalFailure::refusal(
                    "missing required `--proposal`",
                    "rerun with `case append <CASE_ID> --proposal <PATH>`",
                )
            })?;
            let outcome = case::append(
                Path::new("."),
                &case_id,
                expected_revision,
                &proposal,
                &root,
                preview,
            )?;
            print!("{}", outcome.render());
            Ok(())
        }
        CaseCommand::List { root } => {
            let outcome = case::list(Path::new("."), &root)?;
            print!("{}", outcome.render());
            Ok(())
        }
        CaseCommand::Show { case_id, root } => {
            let outcome = case::show(Path::new("."), &case_id, &root)?;
            print!("{}", outcome.render());
            Ok(())
        }
    }
}

fn terminal_exit(result: Result<(), TerminalFailure>) -> ExitCode {
    match result {
        Ok(()) => ExitCode::from(ExitMeaning::Success.status()),
        Err(failure) => {
            eprintln!("{failure}");
            ExitCode::from(failure.meaning().status())
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

fn run_portfolio(roots: &[PathBuf]) -> Result<(), TerminalFailure> {
    let report = portfolio::report(roots)?;
    match report {
        portfolio::PortfolioReport::Complete(report) => {
            print!("{report}");
            Ok(())
        }
        portfolio::PortfolioReport::IdentityConflict(report) => Err(TerminalFailure::refusal(
            format!(
                "duplicate repository identities make the portfolio ambiguous\n{}",
                report.trim_end()
            ),
            "restore a unique stable repository identity for every enrolled repository before rerunning the report",
        )),
    }
}

fn run_enroll(
    ecosystem_id: Option<String>,
    visibility: Option<String>,
    expected_repository_id: Option<String>,
) -> Result<(), TerminalFailure> {
    let ecosystem_id = ecosystem_id.ok_or_else(|| {
        TerminalFailure::refusal(
            "missing required `--ecosystem-id`",
            "rerun with `--ecosystem-id <IDENTITY>`",
        )
    })?;
    let visibility = visibility.ok_or_else(|| {
        TerminalFailure::refusal(
            "missing required `--visibility`",
            "rerun with `--visibility public` or `--visibility private`",
        )
    })?;
    let visibility = Visibility::parse(&visibility)?;
    let expected_repository_id = expected_repository_id
        .map(|value| {
            Uuid::parse_str(&value).map_err(|error| {
                TerminalFailure::refusal(
                    format!("expected repository identity `{value}` is invalid: {error}"),
                    "use the UUID recorded in the existing marker or omit the identity guard",
                )
            })
        })
        .transpose()?;
    let enrollment = enroll_with_expected_repository_id(
        Path::new("."),
        &ecosystem_id,
        visibility,
        expected_repository_id,
    )?;

    report_enrollment(&enrollment);
    Ok(())
}

fn run_set_visibility(visibility: Option<String>) -> Result<(), TerminalFailure> {
    let visibility = visibility.ok_or_else(|| {
        TerminalFailure::refusal(
            "missing required `--visibility`",
            "rerun with `--visibility public` or `--visibility private`",
        )
    })?;
    let visibility = Visibility::parse(&visibility)?;
    let enrollment = set_visibility(Path::new("."), visibility)?;
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
