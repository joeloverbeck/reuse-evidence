#![forbid(unsafe_code)]

use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{CommandFactory, Parser, Subcommand, error::ErrorKind};
use reuse_evidence::{
    ExitMeaning, TerminalFailure, Visibility, case, case::RecordedInstant,
    enroll_with_expected_repository_id, install_skills, portfolio, set_visibility,
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
    /// Name the user-local staging directory for prepared proposals.
    StagingDirectory,
    /// Install this project's own skill packages into a target repository.
    InstallSkills {
        /// Target repository root that will receive the installed package.
        #[arg(long)]
        root: PathBuf,
        /// Explicitly replace installed paths that differ from the shipped package.
        #[arg(long)]
        force: bool,
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
    /// Record a human-authorized early-review override.
    Override {
        /// Opaque identity of the stewarded case to make review-ready.
        case_id: String,
        /// Revision the caller believes the case currently records.
        #[arg(long)]
        expected_revision: Option<i64>,
        /// TOML proposal containing the reason, evidence, and review appetite.
        #[arg(long)]
        proposal: Option<PathBuf>,
        /// Portfolio root for participant resolution; overrides configuration.
        #[arg(long)]
        root: Vec<PathBuf>,
        /// Render the exact event and privacy consequence without writing.
        #[arg(long)]
        preview: bool,
    },
    /// Record the exact reuse decision accepted for a review-ready case.
    Decide {
        /// Opaque identity of the stewarded case whose decision was accepted.
        case_id: String,
        /// Revision the accepted decision was reviewed against.
        #[arg(long)]
        expected_revision: Option<i64>,
        /// TOML proposal containing the exact accepted reuse decision.
        #[arg(long)]
        proposal: Option<PathBuf>,
        /// Portfolio root for participant resolution; overrides configuration.
        #[arg(long)]
        root: Vec<PathBuf>,
        /// Render the exact event and privacy consequence without writing.
        #[arg(long)]
        preview: bool,
    },
    /// Record verification and dispose of a case against its accepted decision.
    Verify {
        /// Opaque identity of the stewarded case whose consequence was verified.
        case_id: String,
        /// Revision the verification was prepared against.
        #[arg(long)]
        expected_revision: Option<i64>,
        /// TOML proposal containing every verification result and the disposition.
        #[arg(long)]
        proposal: Option<PathBuf>,
        /// Portfolio root for participant resolution; overrides configuration.
        #[arg(long)]
        root: Vec<PathBuf>,
        /// Render the exact event and privacy consequence without writing.
        #[arg(long)]
        preview: bool,
    },
    /// Project the implementation handoff from an accepted reuse decision.
    Brief {
        /// Opaque identity of the stewarded case whose brief should be projected.
        case_id: String,
        /// Portfolio root for current participant privacy; overrides configuration.
        #[arg(long)]
        root: Vec<PathBuf>,
    },
    /// Find every case stewarded beneath the selected portfolio roots.
    Find {
        /// Portfolio root for this query; overrides user-local configuration.
        #[arg(long)]
        root: Vec<PathBuf>,
    },
    /// List every case stewarded by the current repository.
    List {
        /// Portfolio root for current participant conditions; overrides configuration.
        #[arg(long)]
        root: Vec<PathBuf>,
    },
    /// Show one stewarded case and its complete recorded evidence history.
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
            terminal_exit(write_stdout(&error.to_string()))
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
        Some(Command::Portfolio { root }) => {
            run_portfolio(&portfolio::PortfolioLocation::from_environment(root))
        }
        Some(Command::StagingDirectory) => {
            let location = portfolio::PortfolioLocation::from_environment(Vec::new());
            run_staging_directory(&location)
        }
        Some(Command::InstallSkills { root, force }) => run_install_skills(&root, force),
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
        } => run_open(
            proposal.as_deref(),
            &portfolio::PortfolioLocation::from_environment(root),
            preview,
        ),
        CaseCommand::Append {
            case_id,
            expected_revision,
            proposal,
            root,
            preview,
        } => run_append(
            &case_id,
            expected_revision,
            proposal.as_deref(),
            &portfolio::PortfolioLocation::from_environment(root),
            preview,
        ),
        CaseCommand::Override {
            case_id,
            expected_revision,
            proposal,
            root,
            preview,
        } => run_override(
            &case_id,
            expected_revision,
            proposal.as_deref(),
            &portfolio::PortfolioLocation::from_environment(root),
            preview,
        ),
        CaseCommand::Decide {
            case_id,
            expected_revision,
            proposal,
            root,
            preview,
        } => run_decide(
            &case_id,
            expected_revision,
            proposal.as_deref(),
            &portfolio::PortfolioLocation::from_environment(root),
            preview,
        ),
        CaseCommand::Verify {
            case_id,
            expected_revision,
            proposal,
            root,
            preview,
        } => run_verify(
            &case_id,
            expected_revision,
            proposal.as_deref(),
            &portfolio::PortfolioLocation::from_environment(root),
            preview,
        ),
        CaseCommand::Brief { case_id, root } => {
            let location = portfolio::PortfolioLocation::from_environment(root);
            let outcome = case::brief(Path::new("."), &case_id, &location)?;
            write_stdout(&outcome.to_string())
        }
        CaseCommand::Find { root } => {
            let location = portfolio::PortfolioLocation::from_environment(root);
            let outcome = case::find(&location)?;
            write_stdout(&outcome.to_string())
        }
        CaseCommand::List { root } => {
            let location = portfolio::PortfolioLocation::from_environment(root);
            let outcome = case::list(Path::new("."), &location)?;
            write_stdout(&outcome.to_string())
        }
        CaseCommand::Show { case_id, root } => {
            let location = portfolio::PortfolioLocation::from_environment(root);
            let outcome = case::show(Path::new("."), &case_id, &location)?;
            write_stdout(&outcome.to_string())
        }
    }
}

fn run_open(
    proposal: Option<&Path>,
    location: &portfolio::PortfolioLocation,
    preview: bool,
) -> Result<(), TerminalFailure> {
    let proposal = proposal.ok_or_else(|| {
        TerminalFailure::refusal(
            "missing required `--proposal`",
            "rerun with `case open --proposal <PATH>`",
        )
    })?;
    let outcome = case::open(
        Path::new("."),
        proposal,
        location,
        RecordedInstant::now()?,
        preview,
    )?;
    write_stdout(&outcome.to_string())
}

fn run_append(
    case_id: &str,
    expected_revision: Option<i64>,
    proposal: Option<&Path>,
    location: &portfolio::PortfolioLocation,
    preview: bool,
) -> Result<(), TerminalFailure> {
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
        case_id,
        expected_revision,
        proposal,
        location,
        RecordedInstant::now()?,
        preview,
    )?;
    write_stdout(&outcome.to_string())
}

fn run_override(
    case_id: &str,
    expected_revision: Option<i64>,
    proposal: Option<&Path>,
    location: &portfolio::PortfolioLocation,
    preview: bool,
) -> Result<(), TerminalFailure> {
    let expected_revision = expected_revision.ok_or_else(|| {
        TerminalFailure::refusal(
            "missing required `--expected-revision`",
            format!(
                "run `case show {case_id}` to recover the current revision, then rerun `case override {case_id} --expected-revision <REVISION>`"
            ),
        )
    })?;
    let proposal = proposal.ok_or_else(|| {
        TerminalFailure::refusal(
            "missing required `--proposal`",
            "rerun with `case override <CASE_ID> --proposal <PATH>`",
        )
    })?;
    let outcome = case::authorize_early_review(
        Path::new("."),
        case_id,
        expected_revision,
        proposal,
        location,
        RecordedInstant::now()?,
        preview,
    )?;
    write_stdout(&outcome.to_string())
}

fn run_decide(
    case_id: &str,
    expected_revision: Option<i64>,
    proposal: Option<&Path>,
    location: &portfolio::PortfolioLocation,
    preview: bool,
) -> Result<(), TerminalFailure> {
    let expected_revision = expected_revision.ok_or_else(|| {
        TerminalFailure::refusal(
            "missing required `--expected-revision`",
            format!(
                "run `case show {case_id}` to recover the current revision, then rerun `case decide {case_id} --expected-revision <REVISION>`"
            ),
        )
    })?;
    let proposal = proposal.ok_or_else(|| {
        TerminalFailure::refusal(
            "missing required `--proposal`",
            "rerun with `case decide <CASE_ID> --proposal <PATH>`",
        )
    })?;
    let outcome = case::decide(
        Path::new("."),
        case_id,
        expected_revision,
        proposal,
        location,
        RecordedInstant::now()?,
        preview,
    )?;
    write_stdout(&outcome.to_string())
}

fn run_verify(
    case_id: &str,
    expected_revision: Option<i64>,
    proposal: Option<&Path>,
    location: &portfolio::PortfolioLocation,
    preview: bool,
) -> Result<(), TerminalFailure> {
    let expected_revision = expected_revision.ok_or_else(|| {
        TerminalFailure::refusal(
            "missing required `--expected-revision`",
            format!(
                "run `case show {case_id}` to recover the current revision, then rerun `case verify {case_id} --expected-revision <REVISION>`"
            ),
        )
    })?;
    let proposal = proposal.ok_or_else(|| {
        TerminalFailure::refusal(
            "missing required `--proposal`",
            "rerun with `case verify <CASE_ID> --proposal <PATH>`",
        )
    })?;
    let outcome = case::verify(
        Path::new("."),
        case_id,
        expected_revision,
        proposal,
        location,
        RecordedInstant::now()?,
        preview,
    )?;
    write_stdout(&outcome.to_string())
}

/// Writes one command's output to stdout, classifying a failure to write it.
///
/// `print!` panics when stdout cannot be written, which exits 101 and bypasses
/// `ExitMeaning` entirely — a status outside the terminal contract ADR 0016
/// makes the process boundary's whole subject. A full disk or a redirect to an
/// unwritable file is a real failure with no no-write guarantee, because a
/// write command may already have published its event before reaching here.
fn write_stdout(text: &str) -> Result<(), TerminalFailure> {
    let mut out = io::stdout().lock();
    match out.write_all(text.as_bytes()).and_then(|()| out.flush()) {
        Ok(()) => Ok(()),
        // A consumer that stopped reading, as `… | head` does, has not made the
        // command fail. Nothing more can be written, so the run simply ends.
        Err(error) if error.kind() == io::ErrorKind::BrokenPipe => Ok(()),
        Err(error) => Err(TerminalFailure::unsafe_failure(format!(
            "command output could not be written to stdout: {error}"
        ))),
    }
}

fn terminal_exit(result: Result<(), TerminalFailure>) -> ExitCode {
    match result {
        Ok(()) => ExitCode::from(ExitMeaning::Success.status()),
        Err(failure) => {
            // Best effort: a failure to report a failure cannot itself be
            // reported, and must not become a panic that loses the status too.
            let mut err = io::stderr().lock();
            let _ = writeln!(err, "{failure}");
            let _ = err.flush();
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

fn run_portfolio(location: &portfolio::PortfolioLocation) -> Result<(), TerminalFailure> {
    write_stdout(&portfolio::report(location)?)
}

fn run_staging_directory(location: &portfolio::PortfolioLocation) -> Result<(), TerminalFailure> {
    let directory = portfolio::prepared_proposal_staging_directory(location)?;
    write_stdout(&format!("{}\n", directory.display()))
}

fn run_install_skills(target_root: &Path, force: bool) -> Result<(), TerminalFailure> {
    write_stdout(&install_skills(target_root, force)?.to_string())
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

    write_stdout(&enrollment.to_string())
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
    write_stdout(&enrollment.to_string())
}
