#!/usr/bin/env node

import { createHash } from "node:crypto";
import { existsSync, lstatSync, readFileSync, readdirSync } from "node:fs";
import { basename, dirname, extname, isAbsolute, relative, resolve, sep } from "node:path";
import { pathToFileURL } from "node:url";

const usage = `Usage:
  node <skill-dir>/scripts/validate-research-brief.mjs <brief-path> [--bundle <bundle-dir>]

The optional bundle must be the brief's sibling <brief-stem>.evidence directory.`;

const REQUIRED_NUMBERED_SECTIONS = [
  "## 0. Commission",
  "## 1. Objective",
  "## 2. Intended outcomes and internal improvement loop",
  "## 3. Operating envelope",
  "## 4. Evidence packet",
  "## 5. Adopted-lineage fence",
  "## 6. Settled-negative and run-scope fences",
  "## 7. Candidate gap areas",
  "## 8. External knowledge bases",
  "## 9. Research tasks",
  "## 10. Recommendation contract",
  "## 11. Citation rules",
  "## 12. Deliverable",
  "## 13. Authority and consumption boundary",
  "## 14. Completion condition"
];

const REQUIRED_COMMISSION_FIELDS = [
  "Audited skill",
  "Repository scope",
  "Source identity",
  "Evidence-packet identity",
  "Decision owner",
  "Commissioner",
  "Research executor",
  "Audit date",
  "Prior audit",
  "In-flight or unconsumed work",
  "Internal-source access",
  "External-source access",
  "Privacy and sanitization"
];

const PLACEHOLDER_PATTERN =
  /\[(?:SKILL NAME|stable identifier|name, requested path[^\]]*|resolved root[^\]]*|content hash[^\]]*|immutable ref[^\]]*|owner|person or skill|executor or mechanism|date(?:\/version[^\]]*)?|none or exact state|verified direct[^\]]*|allowed sources[^\]]*|rules|list|map|facts|limits|access|accessible[^\]]*|item or none|capability|location|version|limit|status|reason|trigger|candidate[^\]]*|field ->[^\]]*|brief path or inline|none or sibling[^\]]*|none or <brief-stem>[^\]]*|concrete trigger|one ready-to-run instruction[^\]]*)\]/gi;

function escapeRegExp(value) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function sha256(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

function sectionBody(lines, heading) {
  const start = lines.findIndex((line) => line === heading);
  if (start < 0) return "";

  const following = lines.slice(start + 1);
  const next = following.findIndex((line) => line.startsWith("## "));
  return (next < 0 ? following : following.slice(0, next)).join("\n");
}

function validateSectionOrder(lines, errors) {
  let previousIndex = -1;
  for (const heading of REQUIRED_NUMBERED_SECTIONS) {
    const indexes = lines.flatMap((line, index) => (line === heading ? [index] : []));
    if (indexes.length !== 1) {
      errors.push(`${heading} must appear exactly once.`);
      continue;
    }
    if (indexes[0] <= previousIndex) {
      errors.push(`${heading} is out of order.`);
    }
    previousIndex = indexes[0];
  }
}

function validateCommissionFields(source, errors) {
  for (const field of REQUIRED_COMMISSION_FIELDS) {
    const pattern = new RegExp(`^- ${escapeRegExp(field)}:\\s*\\S`, "m");
    if (!pattern.test(source)) errors.push(`Commission field ${field} is missing or empty.`);
  }

  const briefIds = [...source.matchAll(/^Brief ID:\s*(\S.*)$/gm)];
  if (briefIds.length !== 1) errors.push("Brief ID must appear exactly once and be non-empty.");

  const recommendations = [
    ...source.matchAll(/^Recommendation:\s*\*\*(commission now|postpone|decline)\*\*\.\s*$/gm)
  ];
  if (recommendations.length !== 1) {
    errors.push("Exactly one bold cadence Recommendation line is required.");
  } else if (recommendations[0][1] !== "commission now") {
    errors.push("A research brief is permitted only for a commission now recommendation.");
  }

  const placeholders = [...new Set(source.match(PLACEHOLDER_PATTERN) ?? [])];
  if (placeholders.length > 0) {
    errors.push(`Unresolved template placeholder(s): ${placeholders.join(", ")}.`);
  }

  return briefIds[0]?.[1] ?? null;
}

function validateAuthorityBoundaries(source, lines, errors) {
  if (!/does not authorize[\s\S]{0,160}\bedits?\b[\s\S]{0,160}\badoption\b/i.test(source)) {
    errors.push("The commission must disclaim edit and recommendation-adoption authority.");
  }

  const authorityBody = sectionBody(lines, "## 13. Authority and consumption boundary");
  if (!/\bDo not edit\b/i.test(authorityBody) || !/\badopt/i.test(authorityBody)) {
    errors.push("Section 13 must prohibit edits and adoption.");
  }

  const completionBody = sectionBody(lines, "## 14. Completion condition");
  if (
    !/cannot establish that no further (?:method )?gaps exist/i.test(source) &&
    !/must not claim that bounded research proves[^.\n]*no method gaps/i.test(source)
  ) {
    errors.push("The brief must state that bounded research cannot prove no method gaps remain.");
  }
  if (!completionBody.trim()) errors.push("Section 14 must be non-empty.");
}

function validateHandoffAndChecklist(lines, source, errors) {
  const handoffHeadings = lines.filter((line) => line === "### Exact executor handoff");
  if (handoffHeadings.length !== 1) {
    errors.push("### Exact executor handoff must appear exactly once.");
  } else {
    const handoff = sectionBody(lines, "### Exact executor handoff");
    if (
      handoff.trim().length < 80 ||
      !/\bDo not edit\b/i.test(handoff) ||
      !/\badopt/i.test(handoff)
    ) {
      errors.push("The executor handoff must be substantive and prohibit edits and adoption.");
    }
  }

  const validationHeadings = lines.filter((line) => line === "## Commission validation");
  if (validationHeadings.length !== 1) {
    errors.push("## Commission validation must appear exactly once.");
    return;
  }

  const validation = sectionBody(lines, "## Commission validation");
  const checkedCount = (validation.match(/^- \[x\] /gim) ?? []).length;
  const uncheckedCount = (validation.match(/^- \[ \] /gm) ?? []).length;
  const requiredCheckedCount = source.includes("### Commission artifact set") ? 17 : 16;
  if (checkedCount < requiredCheckedCount) {
    errors.push(
      `Commission validation must contain at least ${requiredCheckedCount} checked rows.`
    );
  }
  if (uncheckedCount > 0) errors.push("Commission validation contains unchecked rows.");

  if (declaresBundle(source) && !/artifact set/i.test(validation)) {
    errors.push("A bundled commission must check the artifact-set validation row.");
  }
}

function validateInlineBase64(source, errors, paths) {
  const encodingCount = (source.match(/^Encoding:\s*base64[^\n]*$/gim) ?? []).length;
  const pattern =
    /^Path:\s*(.+)\r?\n(?:\r?\nBase-ref blob:\s*[a-f0-9]+\r?\n)?\r?\nCurrent SHA-256:\s*([a-f0-9]{64})\r?\n\r?\nEncoding:\s*base64[^\r\n]*\r?\n\r?\n(`{3,}|~{3,})text\r?\n([A-Za-z0-9+/=\r\n]+)\r?\n\3\s*$/gim;
  const matches = [...source.matchAll(pattern)];

  if (matches.length !== encodingCount) {
    errors.push(
      "Every declared base64 payload must use the canonical Path/hash/fenced-block shape."
    );
  }

  for (const match of matches) {
    const [, sourcePath, expectedHash, , rawPayload] = match;
    const payloadLines = rawPayload.split(/\r?\n/);
    const payload = payloadLines.join("");

    if (paths.has(sourcePath)) errors.push(`Inline evidence path is duplicated: ${sourcePath}.`);
    paths.add(sourcePath);
    if (payloadLines.length !== 1) {
      errors.push(`Inline base64 payload must not wrap: ${sourcePath}.`);
    }
    if (!/^[A-Za-z0-9+/]+={0,2}$/.test(payload) || payload.length % 4 !== 0) {
      errors.push(`Inline base64 payload is malformed: ${sourcePath}.`);
      continue;
    }

    const decoded = Buffer.from(payload, "base64");
    if (decoded.toString("base64") !== payload) {
      errors.push(`Inline base64 payload is not canonical: ${sourcePath}.`);
    }
    if (sha256(decoded) !== expectedHash.toLowerCase()) {
      errors.push(`Inline base64 SHA-256 mismatch: ${sourcePath}.`);
    }
  }
}

function validateInlineUtf8(source, errors, paths) {
  const encodingCount =
    (source.match(/^Encoding:\s*utf8-plaintext[^\n]*$/gim) ?? []).length;
  const pattern =
    /^Path:\s*(.+)\r?\n(?:\r?\nBase-ref blob:\s*[a-f0-9]+\r?\n)?\r?\nCurrent SHA-256:\s*([a-f0-9]{64})\r?\n\r?\nBytes:\s*(0|[1-9][0-9]*)\r?\n\r?\nEncoding:\s*utf8-plaintext,\s*LF line endings\.\r?\n\r?\nTerminal newline:\s*(present|absent)\r?\n\r?\n(`{3,}|~{3,})text\r?\n([\s\S]*?)\r?\n\5\s*$/gim;
  const matches = [...source.matchAll(pattern)];

  if (matches.length !== encodingCount) {
    errors.push(
      "Every declared UTF-8 plaintext payload must use the canonical Path/hash/bytes/terminal-newline/fenced-block shape."
    );
  }

  for (const match of matches) {
    const [, sourcePath, expectedHash, expectedBytes, terminalNewline, , rawPayload] = match;

    if (paths.has(sourcePath)) errors.push(`Inline evidence path is duplicated: ${sourcePath}.`);
    paths.add(sourcePath);
    if (rawPayload.includes("\r")) {
      errors.push(`Inline UTF-8 plaintext payload must use LF line endings: ${sourcePath}.`);
      continue;
    }

    const payload = terminalNewline.toLowerCase() === "present" ? `${rawPayload}\n` : rawPayload;
    const bytes = Buffer.from(payload, "utf8");
    if (bytes.length !== Number(expectedBytes)) {
      errors.push(`Inline UTF-8 plaintext byte count mismatch: ${sourcePath}.`);
    }
    if (sha256(bytes) !== expectedHash.toLowerCase()) {
      errors.push(`Inline UTF-8 plaintext SHA-256 mismatch: ${sourcePath}.`);
    }
  }
}

function declaresBundle(source) {
  const evidenceBundle = source.match(/^- Evidence bundle:\s*(.+)$/im)?.[1] ?? "";
  const packetIdentity = source.match(/^- Evidence-packet identity:\s*(.+)$/im)?.[1] ?? "";
  const internalAccess = source.match(/^- Internal-source access:\s*(.+)$/im)?.[1] ?? "";
  return (
    (evidenceBundle.length > 0 && !/^none\b/i.test(evidenceBundle)) ||
    /\b(?:attached|materialized)\s+(?:evidence\s+)?bundle\b/i.test(packetIdentity) ||
    /\b(?:attached|materialized)\s+(?:evidence\s+)?bundle\b/i.test(internalAccess) ||
    /\bcomplete commission artifact set\b/i.test(internalAccess)
  );
}

function isSafeRelativePath(value) {
  return (
    typeof value === "string" &&
    value.length > 0 &&
    !isAbsolute(value) &&
    !value.includes("\\") &&
    value.split("/").every((part) => part.length > 0 && part !== "." && part !== "..")
  );
}

function collectBundleFiles(directory, root, errors, files = []) {
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    const absolutePath = resolve(directory, entry.name);
    const relativePath = relative(root, absolutePath).split(sep).join("/");
    if (entry.isSymbolicLink()) {
      errors.push(`Evidence bundle must not contain symlinks: ${relativePath}.`);
    } else if (entry.isDirectory()) {
      collectBundleFiles(absolutePath, root, errors, files);
    } else if (entry.isFile()) {
      files.push(relativePath);
    } else {
      errors.push(`Evidence bundle contains an unsupported entry: ${relativePath}.`);
    }
  }
  return files;
}

function validateBundle({ briefId, briefPath, bundlePath, source }, errors) {
  const extension = extname(briefPath);
  const stem = basename(briefPath, extension);
  const expectedBundlePath = resolve(dirname(briefPath), `${stem}.evidence`);
  const resolvedBundlePath = bundlePath ? resolve(bundlePath) : expectedBundlePath;
  const bundleExists = existsSync(resolvedBundlePath);
  const declared = declaresBundle(source);

  if (!bundleExists) {
    if (declared) errors.push(`Declared evidence bundle is missing: ${expectedBundlePath}.`);
    return;
  }
  if (resolvedBundlePath !== expectedBundlePath) {
    errors.push(`Evidence bundle must use the canonical sibling path: ${expectedBundlePath}.`);
  }
  if (!declared) errors.push("An evidence bundle exists but the brief does not declare it.");

  const bundleStat = lstatSync(resolvedBundlePath);
  if (!bundleStat.isDirectory() || bundleStat.isSymbolicLink()) {
    errors.push("Evidence bundle must be a real directory, not a file or symlink.");
    return;
  }

  const manifestPath = resolve(resolvedBundlePath, "manifest.json");
  if (!existsSync(manifestPath) || !lstatSync(manifestPath).isFile()) {
    errors.push("Evidence bundle must contain a regular manifest.json file.");
    return;
  }

  let manifest;
  try {
    manifest = JSON.parse(readFileSync(manifestPath, "utf8"));
  } catch (error) {
    errors.push(`Evidence bundle manifest is invalid JSON: ${error.message}`);
    return;
  }

  if (manifest == null || typeof manifest !== "object" || Array.isArray(manifest)) {
    errors.push("Evidence bundle manifest must be a JSON object.");
    return;
  }
  const allowedManifestKeys = new Set(["schemaVersion", "briefId", "files"]);
  const unknownManifestKeys = Object.keys(manifest).filter((key) => !allowedManifestKeys.has(key));
  if (unknownManifestKeys.length > 0) {
    errors.push(`Unknown evidence bundle manifest field(s): ${unknownManifestKeys.join(", ")}.`);
  }
  if (manifest.schemaVersion !== 1) errors.push("Evidence bundle schemaVersion must equal 1.");
  if (manifest.briefId !== briefId) errors.push("Evidence bundle briefId must match the brief.");
  if (!Array.isArray(manifest.files) || manifest.files.length === 0) {
    errors.push("Evidence bundle manifest files must be a non-empty array.");
    return;
  }

  const manifestPaths = [];
  for (const [index, entry] of manifest.files.entries()) {
    if (entry == null || typeof entry !== "object" || Array.isArray(entry)) {
      errors.push(`Evidence bundle manifest files[${index}] must be an object.`);
      continue;
    }
    const allowedEntryKeys = new Set(["path", "sourcePath", "sha256", "bytes"]);
    const unknownEntryKeys = Object.keys(entry).filter((key) => !allowedEntryKeys.has(key));
    if (unknownEntryKeys.length > 0) {
      errors.push(`Unknown manifest files[${index}] field(s): ${unknownEntryKeys.join(", ")}.`);
    }
    if (!isSafeRelativePath(entry.path) || basename(entry.path) === "manifest.json") {
      errors.push(`Manifest files[${index}].path must be a safe bundled-file path.`);
      continue;
    }
    if (!isSafeRelativePath(entry.sourcePath)) {
      errors.push(`Manifest files[${index}].sourcePath must be a safe source-relative path.`);
    }
    if (!/^[a-f0-9]{64}$/.test(entry.sha256 ?? "")) {
      errors.push(`Manifest files[${index}].sha256 must be 64 lowercase hex characters.`);
    }
    if (!Number.isSafeInteger(entry.bytes) || entry.bytes < 0) {
      errors.push(`Manifest files[${index}].bytes must be a non-negative safe integer.`);
    }

    if (manifestPaths.includes(entry.path)) {
      errors.push(`Manifest path is duplicated: ${entry.path}.`);
      continue;
    }
    manifestPaths.push(entry.path);

    const absolutePath = resolve(resolvedBundlePath, entry.path);
    if (!absolutePath.startsWith(`${resolvedBundlePath}${sep}`)) {
      errors.push(`Manifest path escapes the evidence bundle: ${entry.path}.`);
      continue;
    }
    if (!existsSync(absolutePath)) {
      errors.push(`Manifest file is missing: ${entry.path}.`);
      continue;
    }
    const stat = lstatSync(absolutePath);
    if (!stat.isFile() || stat.isSymbolicLink()) {
      errors.push(`Manifest path must name a regular non-symlink file: ${entry.path}.`);
      continue;
    }
    const bytes = readFileSync(absolutePath);
    if (bytes.length !== entry.bytes) errors.push(`Manifest byte count mismatch: ${entry.path}.`);
    if (sha256(bytes) !== entry.sha256) errors.push(`Manifest SHA-256 mismatch: ${entry.path}.`);
  }

  const actualFiles = collectBundleFiles(resolvedBundlePath, resolvedBundlePath, errors)
    .filter((path) => path !== "manifest.json")
    .sort();
  const expectedFiles = [...manifestPaths].sort();
  const unmanifested = actualFiles.filter((path) => !expectedFiles.includes(path));
  const absent = expectedFiles.filter((path) => !actualFiles.includes(path));
  if (unmanifested.length > 0) {
    errors.push(`Evidence bundle contains unmanifested file(s): ${unmanifested.join(", ")}.`);
  }
  if (absent.length > 0) {
    errors.push(`Evidence bundle manifest names absent file(s): ${absent.join(", ")}.`);
  }
}

export function validateResearchBrief(briefPath, { bundlePath = null } = {}) {
  const errors = [];
  const resolvedBriefPath = resolve(briefPath);
  let source;
  try {
    source = readFileSync(resolvedBriefPath, "utf8");
  } catch (error) {
    return [`Cannot read research brief ${resolvedBriefPath}: ${error.message}`];
  }
  if (!source.trim()) return ["Research brief is empty."];

  const lines = source.split(/\r?\n/);
  validateSectionOrder(lines, errors);
  const briefId = validateCommissionFields(source, errors);
  validateAuthorityBoundaries(source, lines, errors);
  validateHandoffAndChecklist(lines, source, errors);
  const inlinePaths = new Set();
  validateInlineBase64(source, errors, inlinePaths);
  validateInlineUtf8(source, errors, inlinePaths);
  validateBundle({ briefId, briefPath: resolvedBriefPath, bundlePath, source }, errors);
  return errors;
}

function failUsage(message) {
  if (message) console.error(message);
  console.error(usage);
  process.exit(2);
}

function parseArgs(argv) {
  let briefPath = null;
  let bundlePath = null;
  for (let index = 0; index < argv.length; index += 1) {
    const argument = argv[index];
    if (argument === "--help") {
      console.log(usage);
      process.exit(0);
    }
    if (argument === "--bundle") {
      const value = argv[index + 1];
      if (!value || value.startsWith("--")) failUsage("--bundle requires a directory path.");
      if (bundlePath) failUsage("Provide --bundle only once.");
      bundlePath = value;
      index += 1;
      continue;
    }
    if (argument.startsWith("--")) failUsage(`Unknown option: ${argument}`);
    if (briefPath) failUsage("Provide exactly one research brief path.");
    briefPath = argument;
  }
  if (!briefPath) failUsage("Provide one research brief path.");
  return { briefPath, bundlePath };
}

function isMain() {
  return process.argv[1] ? import.meta.url === pathToFileURL(resolve(process.argv[1])).href : false;
}

if (isMain()) {
  const options = parseArgs(process.argv.slice(2));
  const errors = validateResearchBrief(options.briefPath, { bundlePath: options.bundlePath });
  if (errors.length > 0) {
    for (const error of errors) console.error(`- ${error}`);
    process.exitCode = 1;
  } else {
    console.log(`Research brief valid: ${resolve(options.briefPath)}.`);
  }
}
