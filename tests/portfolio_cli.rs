use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

struct Fixture {
    root: PathBuf,
}

impl Fixture {
    fn new(name: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after the Unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "reuse-evidence-{name}-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("fixture root should be creatable");
        Self { root }
    }

    fn repository(&self, portfolio_root: &Path, name: &str) -> PathBuf {
        assert!(
            portfolio_root.starts_with(&self.root),
            "repository fixtures must stay beneath their fixture root"
        );
        let repository = portfolio_root.join(name);
        fs::create_dir_all(repository.join(".git"))
            .expect("repository fixture should be creatable");
        repository
            .canonicalize()
            .expect("repository fixture path should be canonicalizable")
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn run_in(working_directory: &Path, arguments: &[&str], config_home: &Path) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_reuse-evidence"));
    command.args(arguments).current_dir(working_directory);
    #[cfg(target_os = "windows")]
    command
        .env("APPDATA", config_home)
        .env_remove("XDG_CONFIG_HOME")
        .env_remove("HOME");
    #[cfg(not(target_os = "windows"))]
    command.env("XDG_CONFIG_HOME", config_home);
    command
        .output()
        .expect("compiled reuse-evidence binary should run")
}

fn run_without_config_environment(working_directory: &Path, arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_reuse-evidence"))
        .args(arguments)
        .current_dir(working_directory)
        .env_remove("APPDATA")
        .env_remove("XDG_CONFIG_HOME")
        .env_remove("HOME")
        .output()
        .expect("compiled reuse-evidence binary should run")
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

    let output = run_in(&fixture.root, &["portfolio"], &config_home);

    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert_eq!(
        stdout,
        format!(
            "ecosystem: products\n- repository_id: {repository_id}\n  path: {}\n  visibility: private\n",
            repository.display()
        )
    );
    assert!(
        !repository.join("Cargo.toml").exists(),
        "portfolio participation must not require Cargo"
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
            "ecosystem: games\n- repository_id: 00000000-0000-4000-8000-000000000021\n  path: {}\n  visibility: private\necosystem: tools\n- repository_id: 00000000-0000-4000-8000-000000000022\n  path: {}\n  visibility: public\n",
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
