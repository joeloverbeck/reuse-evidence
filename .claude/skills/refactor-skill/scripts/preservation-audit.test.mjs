import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import {
  mkdirSync,
  mkdtempSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import test from "node:test";

const AUDITOR = fileURLToPath(new URL("./preservation-audit.mjs", import.meta.url));

function withTempDirectory(run) {
  const root = mkdtempSync(join(tmpdir(), "refactor-preservation-audit-"));
  try {
    return run(root);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
}

function writeTree(root, files) {
  mkdirSync(root, { recursive: true });
  for (const [relativePath, contents] of Object.entries(files)) {
    const path = join(root, relativePath);
    mkdirSync(dirname(path), { recursive: true });
    writeFileSync(path, contents);
  }
}

function runAuditor(args, options = {}) {
  return spawnSync(process.execPath, [AUDITOR, ...args], {
    cwd: options.cwd,
    encoding: "utf8",
    env: options.env,
  });
}

function runGit(args, cwd) {
  const result = spawnSync("git", args, { cwd, encoding: "utf8" });
  assert.equal(result.status, 0, result.stderr);
  return result.stdout;
}

const PRESERVED_TEXT =
  "Preserve every public interface dependency source requirement pointer fragment and client field.\n";

test("reports identical trees as valid preservation", () => {
  withTempDirectory((root) => {
    const before = join(root, "before");
    const after = join(root, "after");
    writeTree(before, { "SKILL.md": PRESERVED_TEXT });
    writeTree(after, { "SKILL.md": PRESERVED_TEXT });

    const summary = runAuditor([before, after, "summary"]);
    assert.equal(summary.status, 0, summary.stderr);
    assert.equal(summary.stderr, "");
    const report = JSON.parse(summary.stdout);
    assert.equal(report.file_inventory.changes.total, 0);
    assert.equal(report.word_shortfalls.total, 0);
    assert.equal(report.missing_window_clusters.total, 0);

    const checklist = runAuditor([before, after, "checklist"]);
    assert.equal(checklist.status, 0, checklist.stderr);
    const checklistPath = join(root, "preservation-checklist.tsv");
    writeFileSync(checklistPath, checklist.stdout);
    const verified = runAuditor(["verify-checklist", checklistPath]);
    assert.equal(verified.status, 0, verified.stderr);
    assert.deepEqual(JSON.parse(verified.stdout), {
      checklist: checklistPath,
      rows: 0,
      filled_dispositions: 0,
      blank_dispositions: 0,
    });
  });
});

test("reports missing material and refuses an undispositioned checklist", () => {
  withTempDirectory((root) => {
    const before = join(root, "before");
    const after = join(root, "after");
    writeTree(before, {
      "SKILL.md": PRESERVED_TEXT,
      "references/lost.md":
        "This unique preservation witness must remain available to every future refactor run.\n",
    });
    writeTree(after, { "SKILL.md": PRESERVED_TEXT });

    const summary = runAuditor([before, after, "summary"]);
    assert.equal(summary.status, 0, summary.stderr);
    const report = JSON.parse(summary.stdout);
    assert.ok(
      report.file_inventory.changes.shown.some(
        (entry) => entry.kind === "file-missing" && entry.file === "references/lost.md",
      ),
    );
    assert.ok(report.word_shortfalls.total > 0);
    assert.ok(report.missing_window_clusters.total > 0);

    const checklist = runAuditor([before, after, "checklist"]);
    assert.equal(checklist.status, 0, checklist.stderr);
    const checklistPath = join(root, "preservation-checklist.tsv");
    writeFileSync(checklistPath, checklist.stdout);
    const verification = runAuditor(["verify-checklist", checklistPath]);
    assert.equal(verification.status, 2);
    assert.match(verification.stderr, /disposition is blank/);
  });
});

test("reports changed file identities with before and after digests", () => {
  withTempDirectory((root) => {
    const before = join(root, "before");
    const after = join(root, "after");
    writeTree(before, { "SKILL.md": "Original stable interface identity.\n" });
    writeTree(after, { "SKILL.md": "Changed stable interface identity.\n" });

    const result = runAuditor([before, after, "summary"]);
    assert.equal(result.status, 0, result.stderr);
    const report = JSON.parse(result.stdout);
    const changed = report.file_inventory.changes.shown.find(
      (entry) => entry.kind === "file-changed" && entry.file === "SKILL.md",
    );
    assert.ok(changed);
    assert.match(changed.before_digest, /^[a-f0-9]{64}$/);
    assert.match(changed.after_digest, /^[a-f0-9]{64}$/);
    assert.notEqual(changed.before_digest, changed.after_digest);
  });
});

test("rejects missing or unreadable inputs with the public error exit", () => {
  withTempDirectory((root) => {
    const after = join(root, "after");
    writeTree(after, { "SKILL.md": PRESERVED_TEXT });

    const missingTree = runAuditor([join(root, "missing"), after, "summary"]);
    assert.equal(missingTree.status, 2);
    assert.equal(missingTree.stdout, "");
    assert.match(missingTree.stderr, /before directory does not exist/);

    const missingChecklist = runAuditor([
      "verify-checklist",
      join(root, "missing-checklist.tsv"),
    ]);
    assert.equal(missingChecklist.status, 2);
    assert.equal(missingChecklist.stdout, "");
    assert.match(missingChecklist.stderr, /cannot read checklist/);
  });
});

test("emits byte-identical output for repeated explicit inputs", () => {
  withTempDirectory((root) => {
    const before = join(root, "before");
    const after = join(root, "after");
    writeTree(before, { "SKILL.md": PRESERVED_TEXT });
    writeTree(after, { "SKILL.md": PRESERVED_TEXT });

    const first = runAuditor([before, after, "summary"]);
    const second = runAuditor([before, after, "summary"]);
    assert.equal(first.status, 0, first.stderr);
    assert.equal(second.status, 0, second.stderr);
    assert.equal(second.stdout, first.stdout);
    assert.equal(second.stderr, first.stderr);
  });
});

test("summary ordering is independent of process locale", () => {
  withTempDirectory((root) => {
    const before = join(root, "before");
    const after = join(root, "after");
    writeTree(before, {
      "keep.md": "retained\n",
      "z.md": "zeta\n",
      "ä.md": "älg\n",
    });
    writeTree(after, { "keep.md": "retained\n" });
    const summaryFor = (locale) => runAuditor(
      [before, after, "summary"],
      { env: { ...process.env, LANG: locale, LC_ALL: locale } },
    );

    const english = summaryFor("en_US.UTF-8");
    const swedish = summaryFor("sv_SE.UTF-8");
    assert.equal(english.status, 0, english.stderr);
    assert.equal(swedish.status, 0, swedish.stderr);
    assert.equal(swedish.stdout, english.stdout);
    assert.deepEqual(
      JSON.parse(english.stdout).word_shortfalls.shown.map((entry) => entry.word),
      ["zeta", "älg"],
    );
  });
});

test("leaves a tracked fixture tree unmodified", () => {
  withTempDirectory((root) => {
    const repository = join(root, "repository");
    const before = join(repository, "before");
    const after = join(repository, "after");
    mkdirSync(repository);
    runGit(["init", "--quiet"], repository);
    writeTree(before, { "SKILL.md": PRESERVED_TEXT });
    writeTree(after, { "SKILL.md": PRESERVED_TEXT });
    runGit(["add", "."], repository);
    runGit(
      [
        "-c",
        "user.name=Playbench Test",
        "-c",
        "user.email=playbench@example.invalid",
        "commit",
        "--quiet",
        "-m",
        "fixture",
      ],
      repository,
    );
    assert.equal(runGit(["status", "--porcelain"], repository), "");

    const result = runAuditor(["before", "after", "summary"], { cwd: repository });
    assert.equal(result.status, 0, result.stderr);
    assert.equal(runGit(["status", "--porcelain"], repository), "");
  });
});
