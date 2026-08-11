mod support;

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use support::TempRoot;

struct Fixture {
    root: TempRoot,
}

impl Fixture {
    fn new(name: &str) -> Self {
        Self {
            root: TempRoot::new(name),
        }
    }

    fn repository(&self, portfolio_root: &Path, name: &str) -> PathBuf {
        assert!(
            portfolio_root.starts_with(&*self.root),
            "repository fixtures must stay beneath their fixture root"
        );
        support::git_repository(portfolio_root, name)
            .canonicalize()
            .expect("repository fixture path should be canonicalizable")
    }
}

fn run_in(working_directory: &Path, arguments: &[&str], config_home: &Path) -> Output {
    run_with_state_home(
        working_directory,
        arguments,
        config_home,
        &state_home(config_home),
    )
}

fn run_with_state_home(
    working_directory: &Path,
    arguments: &[&str],
    config_home: &Path,
    state_home: &Path,
) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_reuse-evidence"));
    command.args(arguments).current_dir(working_directory);
    #[cfg(target_os = "windows")]
    command
        .env("APPDATA", config_home)
        .env("LOCALAPPDATA", state_home)
        .env_remove("XDG_CONFIG_HOME")
        .env_remove("XDG_STATE_HOME")
        .env_remove("HOME");
    #[cfg(not(target_os = "windows"))]
    command
        .env("XDG_CONFIG_HOME", config_home)
        .env("XDG_STATE_HOME", state_home);
    command
        .output()
        .expect("compiled reuse-evidence binary should run")
}

fn run_without_config_environment(working_directory: &Path, arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_reuse-evidence"))
        .args(arguments)
        .current_dir(working_directory)
        .env_remove("APPDATA")
        .env_remove("LOCALAPPDATA")
        .env_remove("XDG_CONFIG_HOME")
        .env_remove("XDG_STATE_HOME")
        .env_remove("HOME")
        .output()
        .expect("compiled reuse-evidence binary should run")
}

fn state_home(config_home: &Path) -> PathBuf {
    config_home
        .parent()
        .expect("fixture config home should have a parent")
        .join("state")
}

fn portfolio_state_path(config_home: &Path) -> PathBuf {
    state_home(config_home)
        .join("reuse-evidence")
        .join("portfolio.toml")
}

fn toml_contains_string(value: &toml::Value, expected: &str) -> bool {
    match value {
        toml::Value::String(value) => value == expected,
        toml::Value::Array(values) => values
            .iter()
            .any(|value| toml_contains_string(value, expected)),
        toml::Value::Table(values) => values
            .values()
            .any(|value| toml_contains_string(value, expected)),
        _ => false,
    }
}

fn files_beneath(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn visit(root: &Path, directory: &Path, files: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for entry in fs::read_dir(directory).expect("fixture directory should be readable") {
            let entry = entry.expect("fixture entry should be readable");
            let path = entry.path();
            if path.is_dir() {
                visit(root, &path, files);
            } else {
                let relative = path
                    .strip_prefix(root)
                    .expect("fixture entry should be beneath its root")
                    .to_path_buf();
                files.insert(
                    relative,
                    fs::read(path).expect("fixture file should be readable"),
                );
            }
        }
    }

    let mut files = BTreeMap::new();
    visit(root, root, &mut files);
    files
}

fn write_marker(repository: &Path, repository_id: &str, ecosystem_id: &str, visibility: &str) {
    fs::write(
        repository.join("reuse-evidence.toml"),
        format!(
            "schema_version = 1\nrepository_id = \"{repository_id}\"\necosystem_id = \"{ecosystem_id}\"\nvisibility = \"{visibility}\"\n"
        ),
    )
    .expect("marker fixture should be writable");
}

fn write_config(config_home: &Path, portfolio_roots: &[&Path]) -> PathBuf {
    let config_path = config_home.join("reuse-evidence").join("config.toml");
    fs::create_dir_all(
        config_path
            .parent()
            .expect("config path should have a parent"),
    )
    .expect("config directory should be creatable");
    let roots = portfolio_roots
        .iter()
        .map(|root| toml::Value::String(root.to_string_lossy().into_owned()).to_string())
        .collect::<Vec<_>>()
        .join(", ");
    fs::write(&config_path, format!("portfolio_roots = [{roots}]\n"))
        .expect("config fixture should be writable");
    config_path
}

#[test]
fn portfolio_refuses_without_roots_and_names_expected_config() {
    let fixture = Fixture::new("portfolio-no-roots");
    let config_home = fixture.root.join("config");
    let expected_config = config_home.join("reuse-evidence").join("config.toml");
    fs::write(fixture.root.join("preserved.txt"), b"unchanged\n")
        .expect("preserved fixture should be writable");
    let before = files_beneath(&fixture.root);

    let output = run_in(&fixture.root, &["portfolio"], &config_home);

    assert_eq!(output.status.code(), Some(3), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stderr).expect("stderr should be UTF-8"),
        format!(
            "refusal: no portfolio roots were supplied and `{}` does not configure any\nresolution: add `portfolio_roots = [\"/path/to/root\"]` to that user-local configuration or rerun with `--root <PATH>`\n",
            expected_config.display()
        )
    );
    assert!(
        !expected_config.exists(),
        "refusal must not create user-local configuration"
    );
    assert_eq!(
        files_beneath(&fixture.root),
        before,
        "no-roots refusal must leave the target tree byte-identical"
    );
}

#[test]
fn missing_platform_config_directory_names_supported_environment() {
    let fixture = Fixture::new("portfolio-no-config-directory");
    fs::write(fixture.root.join("preserved.txt"), b"unchanged\n")
        .expect("preserved fixture should be writable");
    let before = files_beneath(&fixture.root);

    let output = run_without_config_environment(&fixture.root, &["portfolio"]);

    assert_eq!(output.status.code(), Some(3), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stderr).expect("stderr should be UTF-8"),
        "refusal: the user-local configuration directory cannot be determined\nresolution: set `APPDATA` on Windows, or `XDG_CONFIG_HOME` or `HOME` on Unix-like systems; alternatively rerun with `--root <PATH>`\n"
    );
    assert_eq!(
        files_beneath(&fixture.root),
        before,
        "configuration-directory refusal must leave the target tree byte-identical"
    );
}

#[test]
fn portfolio_reads_roots_from_user_local_config_and_reports_enrollment() {
    let fixture = Fixture::new("portfolio-configured-root");
    let config_home = fixture.root.join("config");
    let portfolio_root = fixture.root.join("portfolio");
    fs::create_dir_all(&portfolio_root).expect("portfolio root should be creatable");
    let repository = fixture.repository(&portfolio_root, "npm-only-app");
    fs::write(
        repository.join("package.json"),
        b"{\"name\":\"@portfolio/npm-only-app\",\"workspaces\":[\"packages/*\"]}\n",
    )
    .expect("npm workspace fixture should be writable");
    let repository_id = "00000000-0000-4000-8000-000000000011";
    write_marker(&repository, repository_id, "products", "private");
    write_config(&config_home, &[&portfolio_root]);
    let repository_before = files_beneath(&repository);

    let output = run_in(&fixture.root, &["portfolio"], &config_home);

    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert_eq!(
        stdout,
        format!(
            "ecosystem: products\n- repository_id: {repository_id}\n  path: {}\n  visibility: private\nnew repositories:\n- repository_id: {repository_id}\n  path: {}\n  visibility: private\n",
            repository.display(),
            repository.display()
        )
    );
    assert!(
        portfolio_state_path(&config_home).is_file(),
        "a successful first observation must create derived user-local state"
    );
    let state_text = fs::read_to_string(portfolio_state_path(&config_home))
        .expect("derived state should be readable UTF-8");
    let state =
        toml::from_str::<toml::Value>(&state_text).expect("derived state should be valid TOML");
    assert!(
        toml_contains_string(&state, repository.to_string_lossy().as_ref()),
        "the absolute local path should be retained in derived user-local state"
    );
    assert_eq!(
        files_beneath(&repository),
        repository_before,
        "portfolio state must not be written into the enrolled repository"
    );
    assert!(
        !repository.join("Cargo.toml").exists(),
        "portfolio participation must not require Cargo"
    );

    fs::remove_file(portfolio_state_path(&config_home))
        .expect("the disposable state file should be deletable");
    let rebuilt = run_in(&fixture.root, &["portfolio"], &config_home);
    assert_eq!(rebuilt.status.code(), Some(0), "{rebuilt:?}");
    assert!(rebuilt.stderr.is_empty(), "{rebuilt:?}");
    assert_eq!(
        String::from_utf8(rebuilt.stdout).expect("stdout should be UTF-8"),
        stdout,
        "deleting derived state must not change the authoritative enrolled set"
    );
    assert!(
        portfolio_state_path(&config_home).is_file(),
        "the disposable state should be rebuilt from current markers"
    );
}

#[test]
fn portfolio_reports_repository_enrolled_between_runs_as_new() {
    let fixture = Fixture::new("portfolio-new-between-runs");
    let config_home = fixture.root.join("config");
    let portfolio_root = fixture.root.join("portfolio");
    fs::create_dir_all(&portfolio_root).expect("portfolio root should be creatable");
    let existing_repository = fixture.repository(&portfolio_root, "existing");
    let existing_repository_id = "00000000-0000-4000-8000-000000000061";
    write_marker(
        &existing_repository,
        existing_repository_id,
        "products",
        "private",
    );
    write_config(&config_home, &[&portfolio_root]);

    let first = run_in(&fixture.root, &["portfolio"], &config_home);
    assert_eq!(first.status.code(), Some(0), "{first:?}");

    let new_repository = fixture.repository(&portfolio_root, "newly-enrolled");
    let new_repository_id = "00000000-0000-4000-8000-000000000062";
    write_marker(&new_repository, new_repository_id, "products", "public");

    let second = run_in(&fixture.root, &["portfolio"], &config_home);

    assert_eq!(second.status.code(), Some(0), "{second:?}");
    assert!(second.stderr.is_empty(), "{second:?}");
    assert_eq!(
        String::from_utf8(second.stdout).expect("stdout should be UTF-8"),
        format!(
            "ecosystem: products\n- repository_id: {existing_repository_id}\n  path: {}\n  visibility: private\n- repository_id: {new_repository_id}\n  path: {}\n  visibility: public\nnew repositories:\n- repository_id: {new_repository_id}\n  path: {}\n  visibility: public\n",
            existing_repository.display(),
            new_repository.display(),
            new_repository.display()
        )
    );
}

#[test]
fn portfolio_reports_relocated_repository_as_moved_not_new() {
    let fixture = Fixture::new("portfolio-moved");
    let config_home = fixture.root.join("config");
    let portfolio_root = fixture.root.join("portfolio");
    fs::create_dir_all(&portfolio_root).expect("portfolio root should be creatable");
    let original_repository = fixture.repository(&portfolio_root, "before-move");
    let repository_id = "00000000-0000-4000-8000-000000000012";
    write_marker(&original_repository, repository_id, "products", "private");
    write_config(&config_home, &[&portfolio_root]);

    let first = run_in(&fixture.root, &["portfolio"], &config_home);
    assert_eq!(first.status.code(), Some(0), "{first:?}");

    let moved_repository = portfolio_root.join("after-move");
    fs::rename(&original_repository, &moved_repository)
        .expect("repository fixture should be movable");
    let moved_repository = moved_repository
        .canonicalize()
        .expect("moved repository should be canonicalizable");

    let second = run_in(&fixture.root, &["portfolio"], &config_home);

    assert_eq!(second.status.code(), Some(0), "{second:?}");
    assert!(second.stderr.is_empty(), "{second:?}");
    let stdout = String::from_utf8(second.stdout).expect("stdout should be UTF-8");
    assert_eq!(
        stdout,
        format!(
            "ecosystem: products\n- repository_id: {repository_id}\n  path: {}\n  visibility: private\nmoved repositories:\n- repository_id: {repository_id}\n  previous_path: {}\n  path: {}\n",
            moved_repository.display(),
            original_repository.display(),
            moved_repository.display()
        )
    );
    assert!(
        !stdout.contains("new repositories:"),
        "a path move must not manufacture a new participant: {stdout}"
    );
}

#[test]
fn portfolio_reports_mover_and_new_tenant_without_substitution() {
    let fixture = Fixture::new("portfolio-mover-new-tenant");
    let config_home = fixture.root.join("config");
    let portfolio_root = fixture.root.join("portfolio");
    fs::create_dir_all(&portfolio_root).expect("portfolio root should be creatable");
    let original_repository = fixture.repository(&portfolio_root, "original");
    let moved_repository_id = "00000000-0000-4000-8000-000000000070";
    let new_repository_id = "00000000-0000-4000-8000-000000000071";
    write_marker(
        &original_repository,
        moved_repository_id,
        "products",
        "private",
    );
    write_config(&config_home, &[&portfolio_root]);

    let first = run_in(&fixture.root, &["portfolio"], &config_home);
    assert_eq!(first.status.code(), Some(0), "{first:?}");

    let moved_repository = portfolio_root.join("moved");
    fs::rename(&original_repository, &moved_repository)
        .expect("repository fixture should be movable");
    let moved_repository = moved_repository
        .canonicalize()
        .expect("moved repository should be canonicalizable");
    let new_repository = fixture.repository(&portfolio_root, "original");
    write_marker(&new_repository, new_repository_id, "products", "public");

    let second = run_in(&fixture.root, &["portfolio"], &config_home);

    assert_eq!(second.status.code(), Some(0), "{second:?}");
    assert!(second.stderr.is_empty(), "{second:?}");
    let stdout = String::from_utf8(second.stdout).expect("stdout should be UTF-8");
    assert_eq!(
        stdout,
        format!(
            "ecosystem: products\n- repository_id: {moved_repository_id}\n  path: {}\n  visibility: private\n- repository_id: {new_repository_id}\n  path: {}\n  visibility: public\nnew repositories:\n- repository_id: {new_repository_id}\n  path: {}\n  visibility: public\nmoved repositories:\n- repository_id: {moved_repository_id}\n  previous_path: {}\n  path: {}\n",
            moved_repository.display(),
            new_repository.display(),
            new_repository.display(),
            original_repository.display(),
            moved_repository.display()
        )
    );
    assert!(
        !stdout.contains("substituted repositories:"),
        "a surviving moved identity must not be reported as substituted: {stdout}"
    );
}

#[test]
fn portfolio_reports_previously_observed_absent_repository_as_unavailable() {
    let fixture = Fixture::new("portfolio-unavailable");
    let config_home = fixture.root.join("config");
    let portfolio_root = fixture.root.join("portfolio");
    fs::create_dir_all(&portfolio_root).expect("portfolio root should be creatable");
    let repository = fixture.repository(&portfolio_root, "available-first");
    let repository_id = "00000000-0000-4000-8000-000000000013";
    write_marker(&repository, repository_id, "products", "private");
    write_config(&config_home, &[&portfolio_root]);

    let first = run_in(&fixture.root, &["portfolio"], &config_home);
    assert_eq!(first.status.code(), Some(0), "{first:?}");

    fs::remove_dir_all(&repository).expect("repository fixture should be removable");
    let expected = format!(
        "no enrolled repositories found\nunavailable repositories:\n- repository_id: {repository_id}\n  path: {}\n  visibility: private\n",
        repository.display()
    );

    let second = run_in(&fixture.root, &["portfolio"], &config_home);
    assert_eq!(second.status.code(), Some(0), "{second:?}");
    assert!(second.stderr.is_empty(), "{second:?}");
    assert_eq!(
        String::from_utf8(second.stdout).expect("stdout should be UTF-8"),
        expected
    );

    let third = run_in(&fixture.root, &["portfolio"], &config_home);
    assert_eq!(third.status.code(), Some(0), "{third:?}");
    assert!(third.stderr.is_empty(), "{third:?}");
    assert_eq!(
        String::from_utf8(third.stdout).expect("stdout should be UTF-8"),
        expected,
        "unavailability must remain visible until the repository returns"
    );
}

#[test]
fn portfolio_reports_visibility_change_and_corrects_derived_state() {
    let fixture = Fixture::new("portfolio-visibility-change");
    let config_home = fixture.root.join("config");
    let portfolio_root = fixture.root.join("portfolio");
    fs::create_dir_all(&portfolio_root).expect("portfolio root should be creatable");
    let repository = fixture.repository(&portfolio_root, "visibility-change");
    let repository_id = "00000000-0000-4000-8000-000000000014";
    write_marker(&repository, repository_id, "products", "private");
    write_config(&config_home, &[&portfolio_root]);

    let first = run_in(&fixture.root, &["portfolio"], &config_home);
    assert_eq!(first.status.code(), Some(0), "{first:?}");

    write_marker(&repository, repository_id, "products", "public");

    let second = run_in(&fixture.root, &["portfolio"], &config_home);
    assert_eq!(second.status.code(), Some(0), "{second:?}");
    assert!(second.stderr.is_empty(), "{second:?}");
    assert_eq!(
        String::from_utf8(second.stdout).expect("stdout should be UTF-8"),
        format!(
            "ecosystem: products\n- repository_id: {repository_id}\n  path: {}\n  visibility: public\nvisibility changed repositories:\n- repository_id: {repository_id}\n  path: {}\n  previous_visibility: private\n  visibility: public\n",
            repository.display(),
            repository.display()
        )
    );

    let third = run_in(&fixture.root, &["portfolio"], &config_home);
    assert_eq!(third.status.code(), Some(0), "{third:?}");
    assert!(third.stderr.is_empty(), "{third:?}");
    assert_eq!(
        String::from_utf8(third.stdout).expect("stdout should be UTF-8"),
        format!(
            "ecosystem: products\n- repository_id: {repository_id}\n  path: {}\n  visibility: public\n",
            repository.display()
        ),
        "the corrected derived state must not repeat an already-observed change"
    );
}

#[test]
fn portfolio_reports_identity_substitution_before_state_advances() {
    let fixture = Fixture::new("portfolio-stale-identity");
    let config_home = fixture.root.join("config");
    let portfolio_root = fixture.root.join("portfolio");
    fs::create_dir_all(&portfolio_root).expect("portfolio root should be creatable");
    let repository = fixture.repository(&portfolio_root, "identity-change");
    let old_repository_id = "00000000-0000-4000-8000-000000000016";
    let current_repository_id = "00000000-0000-4000-8000-000000000017";
    write_marker(&repository, old_repository_id, "products", "private");
    write_config(&config_home, &[&portfolio_root]);

    let first = run_in(&fixture.root, &["portfolio"], &config_home);
    assert_eq!(first.status.code(), Some(0), "{first:?}");

    write_marker(&repository, current_repository_id, "products", "public");

    let second = run_in(&fixture.root, &["portfolio"], &config_home);
    assert_eq!(second.status.code(), Some(0), "{second:?}");
    assert!(second.stderr.is_empty(), "{second:?}");
    let second_stdout = String::from_utf8(second.stdout).expect("stdout should be UTF-8");
    assert_eq!(
        second_stdout,
        format!(
            "ecosystem: products\n- repository_id: {current_repository_id}\n  path: {}\n  visibility: public\nsubstituted repositories:\n- repository_id: {current_repository_id}\n  path: {}\n  previous_repository_id: {old_repository_id}\n  previous_visibility: private\n  visibility: public\n",
            repository.display(),
            repository.display()
        )
    );
    assert!(
        !second_stdout.contains("new repositories:")
            && !second_stdout.contains("unavailable repositories:"),
        "a substitution must not be reported as new or unavailable: {second_stdout}"
    );
    let state_text = fs::read_to_string(portfolio_state_path(&config_home))
        .expect("derived state should be readable UTF-8");
    let state =
        toml::from_str::<toml::Value>(&state_text).expect("derived state should be valid TOML");
    assert!(toml_contains_string(&state, current_repository_id));
    assert!(!toml_contains_string(&state, old_repository_id));

    let third = run_in(&fixture.root, &["portfolio"], &config_home);
    assert_eq!(third.status.code(), Some(0), "{third:?}");
    assert!(third.stderr.is_empty(), "{third:?}");
    assert_eq!(
        String::from_utf8(third.stdout).expect("stdout should be UTF-8"),
        format!(
            "ecosystem: products\n- repository_id: {current_repository_id}\n  path: {}\n  visibility: public\n",
            repository.display()
        ),
        "an identity substitution must be reported only once before state advances"
    );
}

#[test]
fn portfolio_sorts_identity_substitutions_by_current_identity() {
    let fixture = Fixture::new("portfolio-sorted-substitutions");
    let config_home = fixture.root.join("config");
    let portfolio_root = fixture.root.join("portfolio");
    fs::create_dir_all(&portfolio_root).expect("portfolio root should be creatable");
    let first_repository = fixture.repository(&portfolio_root, "first");
    let second_repository = fixture.repository(&portfolio_root, "second");
    let first_previous_id = "00000000-0000-4000-8000-000000000001";
    let second_previous_id = "00000000-0000-4000-8000-000000000099";
    let first_current_id = "00000000-0000-4000-8000-000000000081";
    let second_current_id = "00000000-0000-4000-8000-000000000080";
    write_marker(&first_repository, first_previous_id, "products", "private");
    write_marker(&second_repository, second_previous_id, "products", "public");
    write_config(&config_home, &[&portfolio_root]);

    let first = run_in(&fixture.root, &["portfolio"], &config_home);
    assert_eq!(first.status.code(), Some(0), "{first:?}");

    write_marker(&first_repository, first_current_id, "products", "public");
    write_marker(&second_repository, second_current_id, "products", "private");

    let second = run_in(&fixture.root, &["portfolio"], &config_home);

    assert_eq!(second.status.code(), Some(0), "{second:?}");
    assert!(second.stderr.is_empty(), "{second:?}");
    assert_eq!(
        String::from_utf8(second.stdout).expect("stdout should be UTF-8"),
        format!(
            "ecosystem: products\n- repository_id: {second_current_id}\n  path: {}\n  visibility: private\n- repository_id: {first_current_id}\n  path: {}\n  visibility: public\nsubstituted repositories:\n- repository_id: {second_current_id}\n  path: {}\n  previous_repository_id: {second_previous_id}\n  previous_visibility: public\n  visibility: private\n- repository_id: {first_current_id}\n  path: {}\n  previous_repository_id: {first_previous_id}\n  previous_visibility: private\n  visibility: public\n",
            second_repository.display(),
            first_repository.display(),
            second_repository.display(),
            first_repository.display()
        )
    );
}

#[test]
fn portfolio_refuses_state_path_inside_inspected_repository_without_writes() {
    let fixture = Fixture::new("portfolio-state-inside-repository");
    let config_home = fixture.root.join("config");
    let portfolio_root = fixture.root.join("portfolio");
    fs::create_dir_all(&portfolio_root).expect("portfolio root should be creatable");
    let repository = fixture.repository(&portfolio_root, "state-target");
    write_marker(
        &repository,
        "00000000-0000-4000-8000-000000000015",
        "products",
        "private",
    );
    write_config(&config_home, &[&portfolio_root]);
    let state_home = repository.join("derived-state");
    let state_path = state_home.join("reuse-evidence").join("portfolio.toml");
    let before = files_beneath(&repository);

    let output = run_with_state_home(&fixture.root, &["portfolio"], &config_home, &state_home);

    assert_eq!(output.status.code(), Some(3), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stderr).expect("stderr should be UTF-8"),
        format!(
            "refusal: user-local portfolio state `{}` would be stored inside inspected repository `{}`\nresolution: configure the platform state directory outside every inspected repository\n",
            state_path.display(),
            repository.display()
        )
    );
    assert_eq!(
        files_beneath(&repository),
        before,
        "the construction refusal must leave the repository byte-identical"
    );
    assert!(
        !state_path.exists(),
        "refusal must not create derived state"
    );
}

#[test]
fn unchanged_portfolio_scan_does_not_rewrite_derived_state() {
    let fixture = Fixture::new("portfolio-unchanged-state");
    let config_home = fixture.root.join("config");
    let portfolio_root = fixture.root.join("portfolio");
    fs::create_dir_all(&portfolio_root).expect("portfolio root should be creatable");
    let repository = fixture.repository(&portfolio_root, "unchanged");
    write_marker(
        &repository,
        "00000000-0000-4000-8000-000000000018",
        "products",
        "private",
    );
    write_config(&config_home, &[&portfolio_root]);

    let first = run_in(&fixture.root, &["portfolio"], &config_home);
    assert_eq!(first.status.code(), Some(0), "{first:?}");
    let state_path = portfolio_state_path(&config_home);
    let state_before = fs::read(&state_path).expect("derived state should be readable");
    let identity_alias = state_path.with_file_name("portfolio-identity-alias.toml");
    fs::hard_link(&state_path, &identity_alias)
        .expect("a hard link should expose whether the state file is replaced");

    let second = run_in(&fixture.root, &["portfolio"], &config_home);

    assert_eq!(second.status.code(), Some(0), "{second:?}");
    assert!(second.stderr.is_empty(), "{second:?}");
    assert_eq!(
        fs::read(&state_path).expect("derived state should remain readable"),
        state_before,
        "an unchanged scan must preserve already-current derived state bytes"
    );
    fs::write(&identity_alias, b"same-file-identity\n")
        .expect("the state identity alias should remain writable");
    assert_eq!(
        fs::read(&state_path).expect("derived state should remain readable"),
        b"same-file-identity\n",
        "an unchanged scan must preserve the existing state file identity"
    );
}

#[test]
fn portfolio_refuses_state_path_inside_uninspected_git_repository() {
    let fixture = Fixture::new("portfolio-state-inside-uninspected-repository");
    let config_home = fixture.root.join("config");
    let portfolio_root = fixture.root.join("portfolio");
    fs::create_dir_all(&portfolio_root).expect("portfolio root should be creatable");
    let enrolled_repository = fixture.repository(&portfolio_root, "enrolled");
    write_marker(
        &enrolled_repository,
        "00000000-0000-4000-8000-000000000019",
        "products",
        "private",
    );
    write_config(&config_home, &[&portfolio_root]);

    let unrelated_root = fixture.root.join("unrelated");
    fs::create_dir_all(&unrelated_root).expect("unrelated root should be creatable");
    let state_repository = fixture.repository(&unrelated_root, "state-repository");
    let state_home = state_repository.join("derived-state");
    let state_path = state_home.join("reuse-evidence").join("portfolio.toml");
    let before = files_beneath(&state_repository);

    let output = run_with_state_home(&fixture.root, &["portfolio"], &config_home, &state_home);

    assert_eq!(output.status.code(), Some(3), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stderr).expect("stderr should be UTF-8"),
        format!(
            "refusal: user-local portfolio state `{}` would be stored inside Git repository `{}`\nresolution: configure the platform state directory outside every Git repository\n",
            state_path.display(),
            state_repository.display()
        )
    );
    assert_eq!(
        files_beneath(&state_repository),
        before,
        "the uninspected state repository must remain byte-identical"
    );
}

#[test]
fn portfolio_refuses_normalized_state_path_inside_uninspected_git_repository() {
    let fixture = Fixture::new("portfolio-normalized-state-inside-git");
    let config_home = fixture.root.join("config");
    let portfolio_root = fixture.root.join("portfolio");
    fs::create_dir_all(&portfolio_root).expect("portfolio root should be creatable");
    let enrolled_repository = fixture.repository(&portfolio_root, "enrolled");
    write_marker(
        &enrolled_repository,
        "00000000-0000-4000-8000-000000000020",
        "products",
        "private",
    );
    write_config(&config_home, &[&portfolio_root]);

    let outside = fixture.root.join("outside");
    fs::create_dir_all(&outside).expect("outside directory should be creatable");
    let unrelated_root = fixture.root.join("unrelated");
    fs::create_dir_all(&unrelated_root).expect("unrelated root should be creatable");
    let state_repository = fixture.repository(&unrelated_root, "state-repository");
    let state_home = outside
        .join("..")
        .join("unrelated")
        .join("state-repository")
        .join("derived-state");
    let state_path = state_home.join("reuse-evidence").join("portfolio.toml");
    let before = files_beneath(&state_repository);

    let output = run_with_state_home(&fixture.root, &["portfolio"], &config_home, &state_home);

    assert_eq!(output.status.code(), Some(3), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stderr).expect("stderr should be UTF-8"),
        format!(
            "refusal: user-local portfolio state `{}` would be stored inside Git repository `{}`\nresolution: configure the platform state directory outside every Git repository\n",
            state_path.display(),
            state_repository.display()
        )
    );
    assert_eq!(
        files_beneath(&state_repository),
        before,
        "normalization refusal must leave the state repository byte-identical"
    );
}

#[test]
fn repeatable_root_flags_override_config_and_ecosystems_only_group() {
    let fixture = Fixture::new("portfolio-root-overrides");
    let config_home = fixture.root.join("config");

    let configured_root = fixture.root.join("configured");
    fs::create_dir_all(&configured_root).expect("configured root should be creatable");
    let configured_repository = fixture.repository(&configured_root, "configured-only");
    write_marker(
        &configured_repository,
        "00000000-0000-4000-8000-000000000020",
        "configured",
        "private",
    );
    write_config(&config_home, &[&configured_root]);

    let first_override = fixture.root.join("first-override");
    let second_override = fixture.root.join("second-override");
    fs::create_dir_all(&first_override).expect("first override should be creatable");
    fs::create_dir_all(&second_override).expect("second override should be creatable");
    let tool_repository = fixture.repository(&first_override, "tool");
    let game_repository = fixture.repository(&second_override, "game");
    write_marker(
        &tool_repository,
        "00000000-0000-4000-8000-000000000022",
        "tools",
        "public",
    );
    write_marker(
        &game_repository,
        "00000000-0000-4000-8000-000000000021",
        "games",
        "private",
    );
    let first_override_text = first_override.to_string_lossy().into_owned();
    let second_override_text = second_override.to_string_lossy().into_owned();

    let output = run_in(
        &fixture.root,
        &[
            "portfolio",
            "--root",
            &first_override_text,
            "--root",
            &second_override_text,
        ],
        &config_home,
    );

    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert_eq!(
        stdout,
        format!(
            "ecosystem: games\n- repository_id: 00000000-0000-4000-8000-000000000021\n  path: {}\n  visibility: private\necosystem: tools\n- repository_id: 00000000-0000-4000-8000-000000000022\n  path: {}\n  visibility: public\nnew repositories:\n- repository_id: 00000000-0000-4000-8000-000000000021\n  path: {}\n  visibility: private\n- repository_id: 00000000-0000-4000-8000-000000000022\n  path: {}\n  visibility: public\n",
            game_repository.display(),
            tool_repository.display(),
            game_repository.display(),
            tool_repository.display()
        )
    );
    assert!(
        !stdout.contains("configured-only") && !stdout.contains("ecosystem: configured"),
        "command-line roots must replace the configured set: {stdout}"
    );
}

#[test]
fn portfolio_rescans_marker_state_and_reports_unsupported_versions() {
    let fixture = Fixture::new("portfolio-fresh-rescan");
    let config_home = fixture.root.join("config");
    let portfolio_root = fixture.root.join("portfolio");
    fs::create_dir_all(&portfolio_root).expect("portfolio root should be creatable");
    let withdrawn_repository = fixture.repository(&portfolio_root, "withdrawn-later");
    let upgraded_repository = fixture.repository(&portfolio_root, "newer-marker-later");
    let unmarked_repository = fixture.repository(&portfolio_root, "unmarked-archive");
    fs::write(
        unmarked_repository.join("archive.txt"),
        b"must remain invisible\n",
    )
    .expect("unmarked fixture should be writable");
    write_marker(
        &withdrawn_repository,
        "00000000-0000-4000-8000-000000000031",
        "products",
        "private",
    );
    write_marker(
        &upgraded_repository,
        "00000000-0000-4000-8000-000000000032",
        "products",
        "public",
    );
    write_config(&config_home, &[&portfolio_root]);

    let first = run_in(&fixture.root, &["portfolio"], &config_home);

    assert_eq!(first.status.code(), Some(0), "{first:?}");
    assert!(first.stderr.is_empty(), "{first:?}");
    let first_stdout = String::from_utf8(first.stdout).expect("stdout should be UTF-8");
    assert!(first_stdout.contains("00000000-0000-4000-8000-000000000031"));
    assert!(first_stdout.contains("00000000-0000-4000-8000-000000000032"));
    assert!(
        !first_stdout.contains("unmarked-archive"),
        "unmarked Git repositories must be absent: {first_stdout}"
    );

    fs::remove_file(withdrawn_repository.join("reuse-evidence.toml"))
        .expect("removing a marker should withdraw its repository");
    fs::write(
        upgraded_repository.join("reuse-evidence.toml"),
        b"schema_version = 2\nfuture_shape = { authority = \"newer-binary\" }\n",
    )
    .expect("unsupported marker fixture should be writable");

    let second = run_in(&fixture.root, &["portfolio"], &config_home);

    assert_eq!(second.status.code(), Some(0), "{second:?}");
    assert!(second.stderr.is_empty(), "{second:?}");
    assert_eq!(
        String::from_utf8(second.stdout).expect("stdout should be UTF-8"),
        format!(
            "unsupported marker versions:\n- marker: {}\n  schema_version: 2\n",
            upgraded_repository.join("reuse-evidence.toml").display()
        )
    );
}

#[test]
fn unreadable_marker_is_refused_by_enrollment_and_reported_by_portfolio() {
    let fixture = Fixture::new("portfolio-unreadable-marker");
    let config_home = fixture.root.join("config");
    let portfolio_root = fixture.root.join("portfolio");
    fs::create_dir_all(&portfolio_root).expect("portfolio root should be creatable");
    let repository = fixture.repository(&portfolio_root, "broken-enrollment");
    let marker_path = repository.join("reuse-evidence.toml");
    fs::write(
        &marker_path,
        b"schema_version = 1\nrepository_id = \"00000000-0000-4000-8000-000000000033\"\necosystem_id = \"products\"\nvisibility = \"private\"\nunknown_field = true\n",
    )
    .expect("unreadable marker fixture should be writable");
    write_config(&config_home, &[&portfolio_root]);
    let before = files_beneath(&portfolio_root);

    let enrollment = run_in(
        &repository,
        &[
            "enroll",
            "--ecosystem-id",
            "products",
            "--visibility",
            "private",
        ],
        &config_home,
    );

    assert_eq!(enrollment.status.code(), Some(3), "{enrollment:?}");
    assert!(enrollment.stdout.is_empty(), "{enrollment:?}");
    let enrollment_stderr =
        String::from_utf8(enrollment.stderr).expect("enrollment stderr should be UTF-8");
    assert_eq!(
        enrollment_stderr,
        format!(
            "refusal: existing marker `{}` is malformed: unknown field `unknown_field`, expected one of `schema_version`, `repository_id`, `ecosystem_id`, `visibility`\n\nresolution: restore a complete valid version 1 marker before rerunning enrollment\n",
            marker_path.display()
        ),
        "the shared marker reader must preserve enrollment's stderr byte-for-byte"
    );

    let portfolio = run_in(&fixture.root, &["portfolio"], &config_home);

    assert_eq!(portfolio.status.code(), Some(0), "{portfolio:?}");
    assert!(portfolio.stderr.is_empty(), "{portfolio:?}");
    let portfolio_stdout =
        String::from_utf8(portfolio.stdout).expect("portfolio stdout should be UTF-8");
    assert!(
        portfolio_stdout.contains("unreadable repository markers:\n"),
        "{portfolio_stdout}"
    );
    assert!(
        portfolio_stdout.contains(&format!("- marker: {}\n", marker_path.display())),
        "{portfolio_stdout}"
    );
    assert!(
        portfolio_stdout.contains("  reason: unknown field `unknown_field`"),
        "{portfolio_stdout}"
    );
    assert!(
        !portfolio_stdout.contains("no enrolled repositories found"),
        "a present unreadable marker must not be reported as no enrollment: {portfolio_stdout}"
    );
    for prohibited in ["score", "percentage", "ranking", "health metric"] {
        assert!(
            !portfolio_stdout.to_lowercase().contains(prohibited),
            "portfolio report must not contain `{prohibited}`: {portfolio_stdout}"
        );
    }
    assert_eq!(
        files_beneath(&portfolio_root),
        before,
        "enrollment refusal and portfolio reporting must preserve every inspected repository byte-for-byte"
    );
}

#[test]
fn duplicate_identity_conflict_returns_refusal_exit() {
    let fixture = Fixture::new("portfolio-duplicate-refusal-exit");
    let config_home = fixture.root.join("config");
    let portfolio_root = fixture.root.join("portfolio");
    fs::create_dir_all(&portfolio_root).expect("portfolio root should be creatable");
    let first_repository = fixture.repository(&portfolio_root, "first-copy");
    let second_repository = fixture.repository(&portfolio_root, "second-copy");
    let duplicate_id = "00000000-0000-4000-8000-000000000051";
    write_marker(&first_repository, duplicate_id, "products", "private");
    write_marker(&second_repository, duplicate_id, "products", "public");
    write_config(&config_home, &[&portfolio_root]);

    let output = run_in(&fixture.root, &["portfolio"], &config_home);

    assert_eq!(
        output.status.code(),
        Some(3),
        "duplicate identity must use the refusal exit: {output:?}"
    );
}

#[test]
fn duplicate_identity_refusal_reports_paths_on_stderr_without_writes() {
    let fixture = Fixture::new("portfolio-duplicate-refusal-report");
    let config_home = fixture.root.join("config");
    let portfolio_root = fixture.root.join("portfolio");
    fs::create_dir_all(&portfolio_root).expect("portfolio root should be creatable");
    let first_repository = fixture.repository(&portfolio_root, "first-copy");
    let second_repository = fixture.repository(&portfolio_root, "second-copy");
    let duplicate_id = "00000000-0000-4000-8000-000000000052";
    write_marker(&first_repository, duplicate_id, "products", "private");
    write_marker(&second_repository, duplicate_id, "products", "public");
    write_config(&config_home, &[&portfolio_root]);
    let before = files_beneath(&portfolio_root);

    let output = run_in(&fixture.root, &["portfolio"], &config_home);

    assert_eq!(output.status.code(), Some(3), "{output:?}");
    assert!(
        output.stdout.is_empty(),
        "refusal details belong on stderr: {output:?}"
    );
    let stderr = String::from_utf8(output.stderr).expect("stderr should be UTF-8");
    assert!(
        stderr.starts_with("refusal: duplicate repository identities"),
        "{stderr}"
    );
    assert!(stderr.contains(duplicate_id), "{stderr}");
    assert!(
        stderr.contains(first_repository.to_string_lossy().as_ref()),
        "{stderr}"
    );
    assert!(
        stderr.contains(second_repository.to_string_lossy().as_ref()),
        "{stderr}"
    );
    assert!(stderr.contains("\nresolution:"), "{stderr}");
    assert_eq!(
        files_beneath(&portfolio_root),
        before,
        "duplicate-identity refusal must preserve repository bytes"
    );
}

#[test]
fn portfolio_reports_duplicate_repository_identity_as_conflict() {
    let fixture = Fixture::new("portfolio-duplicate-identity");
    let config_home = fixture.root.join("config");
    let portfolio_root = fixture.root.join("portfolio");
    fs::create_dir_all(&portfolio_root).expect("portfolio root should be creatable");
    let first_repository = fixture.repository(&portfolio_root, "first-copy");
    let second_repository = fixture.repository(&portfolio_root, "second-copy");
    let npm_repository = fixture.repository(&portfolio_root, "npm-workspace");
    let unmarked_repository = fixture.repository(&portfolio_root, "unmarked-vendor");
    let unsupported_repository = fixture.repository(&portfolio_root, "newer-marker");
    let duplicate_id = "00000000-0000-4000-8000-000000000041";
    write_marker(&first_repository, duplicate_id, "products", "private");
    write_marker(&second_repository, duplicate_id, "products", "public");
    write_marker(
        &npm_repository,
        "00000000-0000-4000-8000-000000000042",
        "web",
        "private",
    );
    fs::write(
        npm_repository.join("package.json"),
        b"{\"name\":\"@portfolio/npm-workspace\",\"workspaces\":[\"packages/*\"]}\n",
    )
    .expect("npm workspace fixture should be writable");
    fs::write(unmarked_repository.join("vendor.txt"), b"not enrolled\n")
        .expect("unmarked fixture should be writable");
    fs::write(
        unsupported_repository.join("reuse-evidence.toml"),
        b"schema_version = 2\nfuture_field = true\n",
    )
    .expect("unsupported marker fixture should be writable");
    write_config(&config_home, &[&portfolio_root]);
    let before = files_beneath(&portfolio_root);

    let output = run_in(&fixture.root, &["portfolio"], &config_home);

    assert_eq!(output.status.code(), Some(3), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    let stderr = String::from_utf8(output.stderr).expect("stderr should be UTF-8");
    assert_eq!(
        stderr,
        format!(
            "refusal: duplicate repository identities make the portfolio ambiguous\necosystem: products\n- repository_id: {duplicate_id}\n  path: {}\n  visibility: private\n- repository_id: {duplicate_id}\n  path: {}\n  visibility: public\necosystem: web\n- repository_id: 00000000-0000-4000-8000-000000000042\n  path: {}\n  visibility: private\nduplicate repository identity conflicts:\n- repository_id: {duplicate_id}\n  paths:\n  - {}\n  - {}\nunsupported marker versions:\n- marker: {}\n  schema_version: 2\nresolution: restore a unique stable repository identity for every enrolled repository before rerunning the report\n",
            first_repository.display(),
            second_repository.display(),
            npm_repository.display(),
            first_repository.display(),
            second_repository.display(),
            unsupported_repository.join("reuse-evidence.toml").display()
        )
    );
    assert!(
        !stderr.contains("unmarked-vendor"),
        "unmarked Git repositories must be absent: {stderr}"
    );
    for prohibited in ["score", "percentage", "ranking", "health metric"] {
        assert!(
            !stderr.to_lowercase().contains(prohibited),
            "portfolio report must not contain `{prohibited}`: {stderr}"
        );
    }
    assert_eq!(
        files_beneath(&portfolio_root),
        before,
        "portfolio reporting must preserve every inspected repository byte-for-byte"
    );
    assert!(
        !npm_repository.join("Cargo.toml").exists(),
        "the marked npm workspace must not require Cargo"
    );
}
