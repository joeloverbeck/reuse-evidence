import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { compareCodeUnits } from "./code-unit-order.mjs";

const CONTRACT_VERSION = 1;
const DEFAULT_SCAN_ROOTS = [".claude/skills", "docs"];
const MARKDOWN_EXTENSIONS = new Set([".md", ".markdown"]);

function usage() {
  return [
    "Usage:",
    "  node .claude/skills/refactor-skill/scripts/interface-audit.mjs snapshot <target-skill-dir> [--scan-root <path>]...",
    "  node .claude/skills/refactor-skill/scripts/interface-audit.mjs check <target-skill-dir> [--scan-root <path>]...",
    "  node .claude/skills/refactor-skill/scripts/interface-audit.mjs verify <before.json> <target-skill-dir> [--mode strict|migration]",
    "",
    "snapshot emits the pre-edit interface contract and fails on relevant broken local links.",
    "check audits the current package and active inbound callers without a comparison.",
    "verify compares against a snapshot. Strict mode rejects interface changes; migration mode reports them for ledger reconciliation.",
  ].join("\n");
}

function normalizePath(value) {
  return value.split(path.sep).join("/");
}

function isInside(parent, child) {
  const relative = path.relative(parent, child);
  return relative === "" || (!relative.startsWith(`..${path.sep}`) && relative !== "..");
}

function findRepoRoot(start) {
  let current = path.resolve(start);
  if (fs.existsSync(current) && !fs.statSync(current).isDirectory()) {
    current = path.dirname(current);
  }

  while (true) {
    if (fs.existsSync(path.join(current, ".git"))) {
      return current;
    }
    const parent = path.dirname(current);
    if (parent === current) {
      throw new Error(`no repository root found from ${start}`);
    }
    current = parent;
  }
}

function assertDirectory(directory, label) {
  if (!fs.existsSync(directory) || !fs.statSync(directory).isDirectory()) {
    throw new Error(`${label} directory does not exist: ${directory}`);
  }
}

function walkMarkdown(root) {
  const files = [];

  function visit(directory) {
    for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
      const absolute = path.join(directory, entry.name);
      if (entry.isDirectory()) {
        visit(absolute);
      } else if (
        entry.isFile()
        && MARKDOWN_EXTENSIONS.has(path.extname(entry.name).toLowerCase())
      ) {
        files.push(absolute);
      }
    }
  }

  visit(root);
  return files.sort();
}

function uniqueSorted(values) {
  return [...new Set(values)].sort();
}

function parseFrontmatter(entrypoint) {
  const source = fs.readFileSync(entrypoint, "utf8");
  const match = source.match(/^---\r?\n([\s\S]*?)\r?\n---(?:\r?\n|$)/);
  if (!match) {
    throw new Error(`SKILL.md frontmatter is missing or malformed: ${entrypoint}`);
  }

  const fields = {};
  for (const line of match[1].split(/\r?\n/)) {
    const field = line.match(/^([A-Za-z0-9_-]+):\s*(.*)$/);
    if (field) {
      fields[field[1]] = field[2].trim();
    }
  }

  return {
    name: fields.name ?? "",
    description: fields.description ?? "",
    sha256: crypto.createHash("sha256").update(match[1]).digest("hex"),
  };
}

function clientManifests(target, repoRoot) {
  const agentsDirectory = path.join(target, "agents");
  if (!fs.existsSync(agentsDirectory) || !fs.statSync(agentsDirectory).isDirectory()) {
    return [];
  }

  return fs.readdirSync(agentsDirectory, { withFileTypes: true })
    .filter((entry) => (
      entry.isFile()
      && new Set([".yaml", ".yml"]).has(path.extname(entry.name).toLowerCase())
    ))
    .map((entry) => {
      const absolute = path.join(agentsDirectory, entry.name);
      return {
        path: normalizePath(path.relative(repoRoot, absolute)),
        sha256: crypto.createHash("sha256").update(fs.readFileSync(absolute)).digest("hex"),
      };
    })
    .sort((left, right) => compareCodeUnits(left.path, right.path));
}

function cleanHeadingText(value) {
  return value
    .replace(/\s+#+\s*$/, "")
    .replace(/<[^>]*>/g, "")
    .replace(/!?(?:\[([^\]]+)\]\([^)]*\))/g, "$1")
    .replace(/[`*_~]/g, "")
    .trim();
}

function baseHeadingSlug(value) {
  return cleanHeadingText(value)
    .toLowerCase()
    .replace(/[^\p{L}\p{N} _-]/gu, "")
    .replace(/\s+/g, "-");
}

function markdownAnchors(file) {
  const anchors = new Set();
  const slugCounts = new Map();
  const source = fs.readFileSync(file, "utf8");

  for (const line of source.split(/\r?\n/)) {
    const heading = line.match(/^#{1,6}\s+(.+?)\s*$/);
    if (heading) {
      const base = baseHeadingSlug(heading[1]);
      const count = slugCounts.get(base) ?? 0;
      slugCounts.set(base, count + 1);
      anchors.add(count === 0 ? base : `${base}-${count}`);
    }

    for (const explicit of line.matchAll(/<a\s+[^>]*id=["']([^"']+)["'][^>]*>/gi)) {
      anchors.add(explicit[1]);
    }
  }

  return anchors;
}

function parseDestination(raw) {
  const trimmed = raw.trim();
  let destination;
  if (trimmed.startsWith("<")) {
    const end = trimmed.indexOf(">");
    destination = end === -1 ? trimmed : trimmed.slice(1, end);
  } else {
    destination = trimmed.match(/^\S+/)?.[0] ?? "";
  }

  if (destination === "" || /^(?:[A-Za-z][A-Za-z0-9+.-]*:|\/\/)/.test(destination)) {
    return { local: false, destination };
  }

  const hashIndex = destination.indexOf("#");
  const pathname = hashIndex === -1 ? destination : destination.slice(0, hashIndex);
  const fragment = hashIndex === -1 ? "" : destination.slice(hashIndex + 1);
  let decodedPath = pathname;
  let decodedFragment = fragment;
  try {
    decodedPath = decodeURIComponent(pathname);
    decodedFragment = decodeURIComponent(fragment);
  } catch {
    // Keep the literal destination. A malformed escape will fail path or anchor resolution.
  }

  return {
    local: true,
    destination,
    pathname: decodedPath,
    fragment: decodedFragment,
  };
}

function extractMarkdownDestinations(source) {
  const destinations = [];
  for (const match of source.matchAll(/!?\[[^\]]*\]\(([^)\n]+)\)/g)) {
    destinations.push(match[1]);
  }
  for (const match of source.matchAll(/^\s*\[[^\]]+\]:\s*(\S.*)$/gm)) {
    destinations.push(match[1]);
  }
  return destinations;
}

function resolveLink(sourceFile, raw, repoRoot, anchorCache) {
  const parsed = parseDestination(raw);
  if (!parsed.local) {
    return {
      source: normalizePath(path.relative(repoRoot, sourceFile)),
      raw,
      local: false,
      endpoint: parsed.destination,
      path_exists: true,
      fragment_exists: true,
    };
  }

  const absolute = parsed.pathname === ""
    ? sourceFile
    : path.resolve(path.dirname(sourceFile), parsed.pathname);
  const exists = fs.existsSync(absolute);
  let fragmentExists = true;
  if (
    exists
    && parsed.fragment !== ""
    && MARKDOWN_EXTENSIONS.has(path.extname(absolute).toLowerCase())
  ) {
    if (!anchorCache.has(absolute)) {
      anchorCache.set(absolute, markdownAnchors(absolute));
    }
    fragmentExists = anchorCache.get(absolute).has(parsed.fragment);
  }

  const targetPath = isInside(repoRoot, absolute)
    ? normalizePath(path.relative(repoRoot, absolute))
    : normalizePath(absolute);
  return {
    source: normalizePath(path.relative(repoRoot, sourceFile)),
    raw,
    local: true,
    target_path: targetPath,
    fragment: parsed.fragment,
    endpoint: `${targetPath}${parsed.fragment === "" ? "" : `#${parsed.fragment}`}`,
    absolute_target: absolute,
    path_exists: exists,
    fragment_exists: fragmentExists,
  };
}

function contractTokens(files) {
  const tokens = [];
  const pattern = /`(EDGE-\d+|[A-Z]{2,}-\d+|[A-Z][A-Z0-9_]{2,})`/g;
  for (const file of files) {
    const source = fs.readFileSync(file, "utf8");
    for (const match of source.matchAll(pattern)) {
      tokens.push(match[1]);
    }
  }
  return uniqueSorted(tokens);
}

function semanticEdgeRows(files, repoRoot, anchorCache) {
  const rows = [];
  for (const file of files) {
    for (const line of fs.readFileSync(file, "utf8").split(/\r?\n/)) {
      const ids = [...line.matchAll(/`(EDGE-\d+)`/g)].map((match) => match[1]);
      if (ids.length === 0 || !line.includes("|")) {
        continue;
      }

      const normalized = line
        .replace(/(\[[^\]]*\]\()([^)\n]+)(\))/g, (_whole, open, raw, close) => {
          const resolved = resolveLink(file, raw, repoRoot, anchorCache);
          return `${open}${resolved.local ? resolved.endpoint : resolved.raw}${close}`;
        })
        .trim()
        .replace(/\s+/g, " ");

      for (const id of ids) {
        rows.push({ id, signature: normalized });
      }
    }
  }
  return rows.sort((left, right) => (
    compareCodeUnits(left.id, right.id) || compareCodeUnits(left.signature, right.signature)
  ));
}

function resolveScanRoots(repoRoot, requestedRoots) {
  const candidates = requestedRoots.length === 0 ? DEFAULT_SCAN_ROOTS : requestedRoots;
  const roots = [];
  for (const candidate of candidates) {
    const absolute = path.resolve(repoRoot, candidate);
    if (!isInside(repoRoot, absolute)) {
      throw new Error(`scan root is outside the repository: ${candidate}`);
    }
    if (!fs.existsSync(absolute)) {
      if (requestedRoots.length > 0) {
        throw new Error(`scan root does not exist: ${candidate}`);
      }
      continue;
    }
    assertDirectory(absolute, "scan root");
    roots.push(absolute);
  }
  return uniqueSorted(roots);
}

export function createInterfaceSnapshot(targetRoot, options = {}) {
  const target = path.resolve(targetRoot);
  assertDirectory(target, "target skill");
  const entrypoint = path.join(target, "SKILL.md");
  if (!fs.existsSync(entrypoint)) {
    throw new Error(`target SKILL.md does not exist: ${entrypoint}`);
  }

  const repoRoot = options.repoRoot
    ? path.resolve(options.repoRoot)
    : findRepoRoot(target);
  if (!isInside(repoRoot, target)) {
    throw new Error(`target skill is outside repository root: ${target}`);
  }

  const scanRoots = resolveScanRoots(repoRoot, options.scanRoots ?? []);
  const packageFiles = walkMarkdown(target);
  const scannedFiles = uniqueSorted([
    ...scanRoots.flatMap((root) => walkMarkdown(root)),
    ...packageFiles,
  ]);
  const anchorCache = new Map();
  const allLinks = [];

  for (const file of scannedFiles) {
    const source = fs.readFileSync(file, "utf8");
    for (const raw of extractMarkdownDestinations(source)) {
      allLinks.push(resolveLink(file, raw, repoRoot, anchorCache));
    }
  }

  const targetRelative = normalizePath(path.relative(repoRoot, target));
  const entryRelative = normalizePath(path.relative(repoRoot, entrypoint));
  const inbound = [];
  const externalOutbound = [];
  const problems = [];

  for (const link of allLinks) {
    if (!link.local) {
      continue;
    }
    const absoluteSource = path.resolve(repoRoot, link.source);
    const sourceInsideTarget = isInside(target, absoluteSource);
    const targetInsideTarget = isInside(target, link.absolute_target);

    if (!sourceInsideTarget && targetInsideTarget) {
      inbound.push({
        source: link.source,
        raw: link.raw,
        endpoint: link.endpoint,
        target_path: link.target_path,
        fragment: link.fragment,
      });
    }
    if (sourceInsideTarget && !targetInsideTarget) {
      externalOutbound.push({
        source: link.source,
        raw: link.raw,
        endpoint: link.endpoint,
        target_path: link.target_path,
        fragment: link.fragment,
      });
    }

    if ((sourceInsideTarget || targetInsideTarget) && (!link.path_exists || !link.fragment_exists)) {
      problems.push({
        source: link.source,
        raw: link.raw,
        endpoint: link.endpoint,
        problem: !link.path_exists ? "missing-path" : "missing-fragment",
      });
    }
  }

  const entrypointAnchors = uniqueSorted(markdownAnchors(entrypoint));
  const inboundPublicFragments = uniqueSorted(
    inbound
      .filter((link) => link.target_path === entryRelative && link.fragment !== "")
      .map((link) => link.fragment),
  );
  const tokens = contractTokens(packageFiles);

  return {
    schema: "playbench.refactor-skill.interface-audit/v1",
    contract_version: CONTRACT_VERSION,
    repo_root: normalizePath(repoRoot),
    target_path: targetRelative,
    entrypoint: entryRelative,
    scan_roots: scanRoots.map((root) => normalizePath(path.relative(repoRoot, root))),
    identity: parseFrontmatter(entrypoint),
    client_manifests: clientManifests(target, repoRoot),
    entrypoint_anchors: entrypointAnchors,
    inbound_public_fragments: inboundPublicFragments,
    inbound_links: inbound.sort((left, right) => (
      compareCodeUnits(left.source, right.source) || compareCodeUnits(left.endpoint, right.endpoint)
    )),
    external_outbound_links: externalOutbound.sort((left, right) => (
      compareCodeUnits(left.source, right.source) || compareCodeUnits(left.endpoint, right.endpoint)
    )),
    external_outbound_endpoints: uniqueSorted(externalOutbound.map((link) => link.endpoint)),
    contract_tokens: tokens,
    requirement_ids: tokens.filter((token) => /^[A-Z]{2,}-\d+$/.test(token)),
    dependency_edges: semanticEdgeRows(packageFiles, repoRoot, anchorCache),
    package_markdown_files: packageFiles.map((file) => (
      normalizePath(path.relative(repoRoot, file))
    )),
    problems: problems.sort((left, right) => (
      compareCodeUnits(left.source, right.source) || compareCodeUnits(left.endpoint, right.endpoint)
    )),
  };
}

function missingValues(before, after) {
  const afterSet = new Set(after);
  return before.filter((value) => !afterSet.has(value));
}

function inboundSignatures(snapshot) {
  return snapshot.inbound_links.map((link) => `${link.source} -> ${link.endpoint}`);
}

function edgeSignatures(snapshot) {
  return snapshot.dependency_edges.map((edge) => `${edge.id} -> ${edge.signature}`);
}

export function compareInterfaceSnapshots(before, after, options = {}) {
  const mode = options.mode ?? "strict";
  if (mode !== "strict" && mode !== "migration") {
    throw new Error(`unknown comparison mode: ${mode}`);
  }
  if (before.schema !== "playbench.refactor-skill.interface-audit/v1") {
    throw new Error(`unsupported baseline schema: ${before.schema ?? "missing"}`);
  }

  const changes = [];
  for (const field of ["name", "description"]) {
    if (before.identity[field] !== after.identity[field]) {
      changes.push({
        kind: "identity-changed",
        key: field,
        before: before.identity[field],
        after: after.identity[field],
      });
    }
  }
  if (before.identity.sha256 !== after.identity.sha256) {
    changes.push({
      kind: "frontmatter-changed",
      key: before.entrypoint,
      before: before.identity.sha256,
      after: after.identity.sha256,
    });
  }

  const beforeManifests = new Map(
    (before.client_manifests ?? []).map((manifest) => [manifest.path, manifest.sha256]),
  );
  const afterManifests = new Map(
    (after.client_manifests ?? []).map((manifest) => [manifest.path, manifest.sha256]),
  );
  for (const manifestPath of uniqueSorted([
    ...beforeManifests.keys(),
    ...afterManifests.keys(),
  ])) {
    if (beforeManifests.get(manifestPath) !== afterManifests.get(manifestPath)) {
      changes.push({
        kind: "client-manifest-changed",
        key: manifestPath,
        before: beforeManifests.get(manifestPath) ?? "absent",
        after: afterManifests.get(manifestPath) ?? "absent",
      });
    }
  }

  for (const fragment of before.inbound_public_fragments) {
    if (!after.entrypoint_anchors.includes(fragment)) {
      changes.push({
        kind: "public-anchor-removed",
        key: fragment,
        before: before.entrypoint,
        after: "missing from current entrypoint",
      });
    }
  }

  for (const signature of missingValues(inboundSignatures(before), inboundSignatures(after))) {
    changes.push({
      kind: "inbound-interface-changed",
      key: signature,
      before: signature,
      after: "missing or retargeted",
    });
  }
  for (const signature of missingValues(inboundSignatures(after), inboundSignatures(before))) {
    changes.push({
      kind: "inbound-interface-added",
      key: signature,
      before: "absent",
      after: signature,
    });
  }

  for (const endpoint of missingValues(
    before.external_outbound_endpoints,
    after.external_outbound_endpoints,
  )) {
    changes.push({
      kind: "external-outbound-endpoint-removed",
      key: endpoint,
      before: endpoint,
      after: "missing or retargeted",
    });
  }
  for (const endpoint of missingValues(
    after.external_outbound_endpoints,
    before.external_outbound_endpoints,
  )) {
    changes.push({
      kind: "external-outbound-endpoint-added",
      key: endpoint,
      before: "absent",
      after: endpoint,
    });
  }

  for (const token of missingValues(before.contract_tokens, after.contract_tokens)) {
    changes.push({
      kind: "contract-token-removed",
      key: token,
      before: token,
      after: "missing",
    });
  }
  for (const token of missingValues(after.contract_tokens, before.contract_tokens)) {
    changes.push({
      kind: "contract-token-added",
      key: token,
      before: "absent",
      after: token,
    });
  }

  for (const signature of missingValues(edgeSignatures(before), edgeSignatures(after))) {
    changes.push({
      kind: "dependency-edge-changed",
      key: signature,
      before: signature,
      after: "missing or changed",
    });
  }
  for (const signature of missingValues(edgeSignatures(after), edgeSignatures(before))) {
    changes.push({
      kind: "dependency-edge-added",
      key: signature,
      before: "absent",
      after: signature,
    });
  }

  const problems = after.problems.map((problem) => ({
    kind: "link-problem",
    key: `${problem.source} -> ${problem.raw}`,
    ...problem,
  }));
  const errors = mode === "strict" ? [...problems, ...changes] : problems;

  return {
    schema: "playbench.refactor-skill.interface-verify/v1",
    mode,
    ok: errors.length === 0,
    changes,
    problems,
    errors,
    requires_migration_dispositions: mode === "migration" ? changes.length : 0,
    baseline: {
      target_path: before.target_path,
      inbound_links: before.inbound_links.length,
      public_fragments: before.inbound_public_fragments.length,
      external_outbound_endpoints: before.external_outbound_endpoints.length,
      contract_tokens: before.contract_tokens.length,
      dependency_edges: before.dependency_edges.length,
      client_manifests: (before.client_manifests ?? []).length,
    },
    current: {
      target_path: after.target_path,
      inbound_links: after.inbound_links.length,
      public_fragments: after.inbound_public_fragments.length,
      external_outbound_endpoints: after.external_outbound_endpoints.length,
      contract_tokens: after.contract_tokens.length,
      dependency_edges: after.dependency_edges.length,
      client_manifests: (after.client_manifests ?? []).length,
    },
  };
}

function parseScanRoots(args) {
  const positional = [];
  const scanRoots = [];
  for (let index = 0; index < args.length; index += 1) {
    if (args[index] === "--scan-root") {
      if (index + 1 >= args.length) {
        throw new Error("--scan-root requires a path");
      }
      scanRoots.push(args[index + 1]);
      index += 1;
    } else {
      positional.push(args[index]);
    }
  }
  return { positional, scanRoots };
}

function parseVerify(args) {
  const positional = [];
  let mode = "strict";
  for (let index = 0; index < args.length; index += 1) {
    if (args[index] === "--mode") {
      if (index + 1 >= args.length) {
        throw new Error("--mode requires strict or migration");
      }
      mode = args[index + 1];
      index += 1;
    } else {
      positional.push(args[index]);
    }
  }
  return { positional, mode };
}

function readSnapshot(snapshotPath) {
  try {
    return JSON.parse(fs.readFileSync(snapshotPath, "utf8"));
  } catch (error) {
    throw new Error(`cannot read interface snapshot ${snapshotPath}: ${error.message}`);
  }
}

function printJson(value) {
  process.stdout.write(`${JSON.stringify(value, null, 2)}\n`);
}

function main(argv) {
  if (argv.length === 1 && (argv[0] === "--help" || argv[0] === "-h")) {
    console.log(usage());
    return;
  }

  const [command, ...rest] = argv;
  if (command === "snapshot" || command === "check") {
    const { positional, scanRoots } = parseScanRoots(rest);
    if (positional.length !== 1) {
      throw new Error(usage());
    }
    const snapshot = createInterfaceSnapshot(positional[0], { scanRoots });
    if (command === "snapshot") {
      printJson(snapshot);
    } else {
      printJson({
        schema: "playbench.refactor-skill.interface-check/v1",
        ok: snapshot.problems.length === 0,
        target_path: snapshot.target_path,
        inbound_links: snapshot.inbound_links.length,
        public_fragments: snapshot.inbound_public_fragments.length,
        external_outbound_endpoints: snapshot.external_outbound_endpoints.length,
        contract_tokens: snapshot.contract_tokens.length,
        dependency_edges: snapshot.dependency_edges.length,
        client_manifests: snapshot.client_manifests.length,
        problems: snapshot.problems,
      });
    }
    if (snapshot.problems.length > 0) {
      process.exitCode = 2;
    }
    return;
  }

  if (command === "verify") {
    const { positional, mode } = parseVerify(rest);
    if (positional.length !== 2) {
      throw new Error(usage());
    }
    const before = readSnapshot(path.resolve(positional[0]));
    const repoRoot = findRepoRoot(path.resolve(positional[1]));
    const after = createInterfaceSnapshot(positional[1], {
      repoRoot,
      scanRoots: before.scan_roots,
    });
    const result = compareInterfaceSnapshots(before, after, { mode });
    printJson(result);
    if (!result.ok) {
      process.exitCode = 2;
    }
    return;
  }

  throw new Error(usage());
}

const isMain = process.argv[1]
  && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url);
if (isMain) {
  try {
    main(process.argv.slice(2));
  } catch (error) {
    console.error(`interface-audit: ${error.message}`);
    process.exitCode = 2;
  }
}
