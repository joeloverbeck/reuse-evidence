#![forbid(unsafe_code)]

use std::path::Path;
use std::process::ExitCode;

use clap::{CommandFactory, Parser, Subcommand, error::ErrorKind};
use reuse_evidence::{ExitMeaning, Visibility, enroll};

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
        }) => run_enroll(ecosystem_id, visibility),
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
    let enrollment = enroll(Path::new("."), &ecosystem_id, visibility)
        .map_err(|error| (error.meaning(), error.to_string()))?;

    println!("enrolled repository");
    println!("marker: {}", enrollment.marker_path.display());
    println!("repository_id: {}", enrollment.repository_id);
    println!("ecosystem_id: {}", enrollment.ecosystem_id);
    println!("visibility: {}", enrollment.visibility);
    Ok(())
}

const fn terminal_name(meaning: ExitMeaning) -> &'static str {
    match meaning {
        ExitMeaning::Success => "success",
        ExitMeaning::UnsafeFailure => "unsafe failure",
        ExitMeaning::Refusal => "refusal",
    }
}
