import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import {
  mkdirSync,
  mkdtempSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import test from "node:test";

import {
  compareInterfaceSnapshots,
  createInterfaceSnapshot,
} from "./interface-audit.mjs";

const AUDITOR = fileURLToPath(new URL("./interface-audit.mjs", import.meta.url));

const initialSkill = `---
name: alpha
description: Preserve the alpha interface.
---

# Alpha

## Public Contract

Return \`BLOCKED_INPUT\` when \`RU-020\` is missing.

| Edge | Condition | Owner |
| --- | --- | --- |
| \`EDGE-001\` | Beta output is needed | Read [Beta outputs](../beta/SKILL.md#outputs). |
`;

const refactoredSkill = `---
name: alpha
description: Preserve the alpha interface.
---

# Alpha

## Public Contract

When the public-contract branch fires, read
[Public contract details](references/public-contract.md#public-contract-details)
in full before returning to the caller.
`;

const contractReference = `# Public Contract Details

Return \`BLOCKED_INPUT\` when \`RU-020\` is missing.

| Edge | Condition | Owner |
| --- | --- | --- |
| \`EDGE-001\` | Beta output is needed | Read [Beta outputs](../../beta/SKILL.md#outputs). |
`;

function withFixture(run) {
  const root = mkdtempSync(join(tmpdir(), "refactor-interface-audit-"));
  try {
    mkdirSync(join(root, ".git"));
    mkdirSync(join(root, "skills/alpha/references"), { recursive: true });
    mkdirSync(join(root, "skills/alpha/agents"), { recursive: true });
    mkdirSync(join(root, "skills/beta"), { recursive: true });
    mkdirSync(join(root, "docs/workflows"), { recursive: true });
    writeFileSync(join(root, "skills/alpha/SKILL.md"), initialSkill);
    writeFileSync(
      join(root, "skills/alpha/agents/openai.yaml"),
      "interface:\n  display_name: Alpha\npolicy:\n  allow_implicit_invocation: false\n",
    );
    writeFileSync(join(root, "skills/beta/SKILL.md"), "# Beta\n\n## Outputs\n\nStable output.\n");
    writeFileSync(
      join(root, "docs/workflows/caller.md"),
      "# Caller\n\nUse [Alpha](../../skills/alpha/SKILL.md#public-contract).\n",
    );
    writeFileSync(
      join(root, "docs/workflows/temporary-source.md"),
      [
        "# Temporary Source",
        "",
        "> Temporary workflow specification for later conversion into an agent skill.",
        "",
        "[Alpha]: ../../skills/alpha/SKILL.md#public-contract",
        "",
      ].join("\n"),
    );
    return run(root);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
}

function snapshot(root) {
  return createInterfaceSnapshot(join(root, "skills/alpha"), {
    repoRoot: root,
    scanRoots: ["skills", "docs"],
  });
}

function applyCompatibleRefactor(root) {
  writeFileSync(join(root, "skills/alpha/SKILL.md"), refactoredSkill);
  writeFileSync(
    join(root, "skills/alpha/references/public-contract.md"),
    contractReference,
  );
}

test("captures active callers, public anchors, contract tokens, requirements, and edges", () => {
  withFixture((root) => {
    const current = snapshot(root);
    assert.equal(current.problems.length, 0);
    assert.equal(current.inbound_links.length, 2);
    assert.deepEqual(current.inbound_public_fragments, ["public-contract"]);
    assert.ok(current.contract_tokens.includes("BLOCKED_INPUT"));
    assert.ok(current.requirement_ids.includes("RU-020"));
    assert.equal(current.dependency_edges.length, 1);
    assert.equal(current.client_manifests.length, 1);
  });
});

test("CLI preserves valid-package output and exit status", () => {
  withFixture((root) => {
    const result = spawnSync(
      process.execPath,
      [
        AUDITOR,
        "check",
        "skills/alpha",
        "--scan-root",
        "skills",
        "--scan-root",
        "docs",
      ],
      { cwd: root, encoding: "utf8" },
    );

    assert.equal(result.status, 0);
    assert.equal(result.stderr, "");
    assert.deepEqual(JSON.parse(result.stdout), {
      schema: "playbench.refactor-skill.interface-check/v1",
      ok: true,
      target_path: "skills/alpha",
      inbound_links: 2,
      public_fragments: 1,
      external_outbound_endpoints: 1,
      contract_tokens: 3,
      dependency_edges: 1,
      client_manifests: 1,
      problems: [],
    });
  });
});

test("CLI snapshot ordering is independent of process locale", () => {
  withFixture((root) => {
    writeFileSync(
      join(root, "skills/alpha/agents/z.yaml"),
      "interface:\n  display_name: Zed\n",
    );
    writeFileSync(
      join(root, "skills/alpha/agents/ä.yaml"),
      "interface:\n  display_name: A Umlaut\n",
    );
    const snapshotFor = (locale) => spawnSync(
      process.execPath,
      [
        AUDITOR,
        "snapshot",
        "skills/alpha",
        "--scan-root",
        "skills",
        "--scan-root",
        "docs",
      ],
      {
        cwd: root,
        encoding: "utf8",
        env: { ...process.env, LANG: locale, LC_ALL: locale },
      },
    );

    const english = snapshotFor("en_US.UTF-8");
    const swedish = snapshotFor("sv_SE.UTF-8");
    assert.equal(english.status, 0, english.stderr);
    assert.equal(swedish.status, 0, swedish.stderr);
    assert.equal(swedish.stdout, english.stdout);
    assert.deepEqual(
      JSON.parse(english.stdout).client_manifests.map((manifest) => manifest.path),
      [
        "skills/alpha/agents/openai.yaml",
        "skills/alpha/agents/z.yaml",
        "skills/alpha/agents/ä.yaml",
      ],
    );
  });
});

test("CLI preserves broken-link diagnostics and exit status", () => {
  withFixture((root) => {
    writeFileSync(
      join(root, "docs/workflows/caller.md"),
      "# Caller\n\nUse [Alpha](../../skills/alpha/SKILL.md#missing-contract).\n",
    );
    const result = spawnSync(
      process.execPath,
      [
        AUDITOR,
        "check",
        "skills/alpha",
        "--scan-root",
        "skills",
        "--scan-root",
        "docs",
      ],
      { cwd: root, encoding: "utf8" },
    );

    assert.equal(result.status, 2);
    assert.equal(result.stderr, "");
    const report = JSON.parse(result.stdout);
    assert.equal(report.ok, false);
    assert.deepEqual(report.problems, [
      {
        source: "docs/workflows/caller.md",
        raw: "../../skills/alpha/SKILL.md#missing-contract",
        endpoint: "skills/alpha/SKILL.md#missing-contract",
        problem: "missing-fragment",
      },
    ]);
  });
});

test("accepts extraction with a public stub and identity-preserving relative-link rebase", () => {
  withFixture((root) => {
    const before = snapshot(root);
    applyCompatibleRefactor(root);
    const after = snapshot(root);
    const result = compareInterfaceSnapshots(before, after, { mode: "strict" });

    assert.equal(after.problems.length, 0);
    assert.equal(result.ok, true);
    assert.deepEqual(result.changes, []);
  });
});

test("rejects removal of a public entrypoint anchor", () => {
  withFixture((root) => {
    const before = snapshot(root);
    applyCompatibleRefactor(root);
    writeFileSync(
      join(root, "skills/alpha/SKILL.md"),
      refactoredSkill.replace("## Public Contract", "## Contract Index"),
    );
    const after = snapshot(root);
    const result = compareInterfaceSnapshots(before, after, { mode: "strict" });

    assert.equal(result.ok, false);
    assert.ok(result.errors.some((error) => error.kind === "public-anchor-removed"));
    assert.ok(result.errors.some((error) => error.kind === "link-problem"));
  });
});

test("rejects a relative link that resolves to a different existing owner", () => {
  withFixture((root) => {
    const before = snapshot(root);
    applyCompatibleRefactor(root);
    mkdirSync(join(root, "skills/alpha/beta"), { recursive: true });
    writeFileSync(
      join(root, "skills/alpha/beta/SKILL.md"),
      "# Wrong Beta\n\n## Outputs\n\nWrong owner.\n",
    );
    writeFileSync(
      join(root, "skills/alpha/references/public-contract.md"),
      contractReference.replace("../../beta/SKILL.md", "../beta/SKILL.md"),
    );
    const after = snapshot(root);
    const result = compareInterfaceSnapshots(before, after, { mode: "strict" });

    assert.equal(after.problems.length, 0);
    assert.equal(result.ok, false);
    assert.ok(
      result.errors.some((error) => error.kind === "external-outbound-endpoint-removed"),
    );
  });
});

test("rejects removed contract tokens and changed skill identity", () => {
  withFixture((root) => {
    const before = snapshot(root);
    applyCompatibleRefactor(root);
    writeFileSync(
      join(root, "skills/alpha/SKILL.md"),
      refactoredSkill.replace("description: Preserve", "description: Replace"),
    );
    writeFileSync(
      join(root, "skills/alpha/references/public-contract.md"),
      contractReference
        .replace("`BLOCKED_INPUT`", "a blocked result")
        .concat("\nReturn `NEW_OUTCOME` only after an invented dependency.\n"),
    );
    writeFileSync(
      join(root, "skills/alpha/agents/openai.yaml"),
      "interface:\n  display_name: Changed Alpha\npolicy:\n  allow_implicit_invocation: false\n",
    );
    const result = compareInterfaceSnapshots(before, snapshot(root), { mode: "strict" });

    assert.equal(result.ok, false);
    assert.ok(result.errors.some((error) => error.kind === "identity-changed"));
    assert.ok(result.errors.some((error) => error.kind === "frontmatter-changed"));
    assert.ok(result.errors.some((error) => error.kind === "client-manifest-changed"));
    assert.ok(result.errors.some((error) => error.kind === "contract-token-removed"));
    assert.ok(result.errors.some((error) => error.kind === "contract-token-added"));
  });
});

test("migration mode reports authorized-shape changes after callers are retargeted", () => {
  withFixture((root) => {
    const before = snapshot(root);
    applyCompatibleRefactor(root);
    writeFileSync(
      join(root, "skills/alpha/SKILL.md"),
      refactoredSkill.replace("## Public Contract", "## Contract Index"),
    );
    writeFileSync(
      join(root, "docs/workflows/caller.md"),
      "# Caller\n\nUse [Alpha](../../skills/alpha/SKILL.md#contract-index).\n",
    );
    writeFileSync(
      join(root, "docs/workflows/temporary-source.md"),
      [
        "# Temporary Source",
        "",
        "> Temporary workflow specification for later conversion into an agent skill.",
        "",
        "[Alpha]: ../../skills/alpha/SKILL.md#contract-index",
        "",
      ].join("\n"),
    );
    const after = snapshot(root);
    const result = compareInterfaceSnapshots(before, after, { mode: "migration" });

    assert.equal(after.problems.length, 0);
    assert.equal(result.ok, true);
    assert.ok(result.changes.length >= 2);
    assert.equal(result.requires_migration_dispositions, result.changes.length);
  });
});
