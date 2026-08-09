import crypto from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';

import { compareCodeUnits } from './code-unit-order.mjs';

const TEXT_EXTENSIONS = new Set([
  '.json',
  '.markdown',
  '.md',
  '.toml',
  '.txt',
  '.yaml',
  '.yml',
]);
const WINDOW_SIZE = 10;
const WINDOW_STRIDE = 1;
const DEFAULT_LIMIT = 20;
const CHECKLIST_COLUMNS = [
  'kind',
  'key',
  'source',
  'before',
  'after',
  'delta',
  'location',
  'sample',
  'disposition',
];
const CHECKLIST_HEADER = CHECKLIST_COLUMNS.join('\t');
const CHECKLIST_KINDS = new Set([
  'file-added',
  'file-changed',
  'file-missing',
  'window-cluster',
  'word-shortfall',
]);
const CHECKLIST_ERROR_LIMIT = 20;

function usage() {
  return [
    'Usage:',
    '  node .claude/skills/refactor-skill/scripts/preservation-audit.mjs <before-dir> <after-dir> [summary [limit]]',
    '  node .claude/skills/refactor-skill/scripts/preservation-audit.mjs <before-dir> <after-dir> checklist',
    '  node .claude/skills/refactor-skill/scripts/preservation-audit.mjs verify-checklist <checklist.tsv>',
    '',
    'summary emits bounded JSON with an all-file inventory. checklist emits complete TSV with a blank disposition column.',
    'verify-checklist validates the filled TSV and fails when its schema or any disposition is incomplete.',
  ].join('\n');
}

function parseArguments(argv) {
  if (argv.length === 1 && (argv[0] === '--help' || argv[0] === '-h')) {
    return { help: true };
  }

  if (argv[0] === 'verify-checklist') {
    if (argv.length !== 2) {
      throw new Error(`verify-checklist requires exactly one checklist path\n\n${usage()}`);
    }
    return {
      help: false,
      mode: 'verify-checklist',
      checklistPath: path.resolve(argv[1]),
    };
  }

  if (argv.length < 2 || argv.length > 4) {
    throw new Error(usage());
  }

  const [beforeRoot, afterRoot, mode = 'summary', limitText] = argv;
  if (mode !== 'summary' && mode !== 'checklist') {
    throw new Error(`Unknown mode: ${mode}\n\n${usage()}`);
  }
  if (mode === 'checklist' && limitText !== undefined) {
    throw new Error(`checklist does not accept a limit\n\n${usage()}`);
  }

  const limit = limitText === undefined ? DEFAULT_LIMIT : Number.parseInt(limitText, 10);
  if (mode === 'summary' && (!Number.isInteger(limit) || limit < 1)) {
    throw new Error('summary limit must be a positive integer');
  }

  return {
    help: false,
    beforeRoot: path.resolve(beforeRoot),
    afterRoot: path.resolve(afterRoot),
    mode,
    limit,
  };
}

function assertDirectory(root, label) {
  let stat;
  try {
    stat = fs.statSync(root);
  } catch {
    throw new Error(`${label} directory does not exist: ${root}`);
  }
  if (!stat.isDirectory()) {
    throw new Error(`${label} path is not a directory: ${root}`);
  }
}

function walkEntries(root) {
  const files = [];

  function visit(directory) {
    for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
      const absolute = path.join(directory, entry.name);
      if (entry.isDirectory()) {
        visit(absolute);
      } else if (entry.isFile() || entry.isSymbolicLink()) {
        files.push(absolute);
      }
    }
  }

  visit(root);
  return files.sort(compareCodeUnits);
}

function walkTextFiles(root) {
  return walkEntries(root).filter((file) => (
    fs.lstatSync(file).isFile()
    && TEXT_EXTENSIONS.has(path.extname(file).toLowerCase())
  ));
}

function sha256(value) {
  return crypto.createHash('sha256').update(value).digest('hex');
}

function inventoryTree(root) {
  return walkEntries(root).map((file) => {
    const stat = fs.lstatSync(file);
    const entryType = stat.isSymbolicLink() ? 'symlink' : 'file';
    const supportedText = entryType === 'file'
      && TEXT_EXTENSIONS.has(path.extname(file).toLowerCase());
    return {
      relative: path.relative(root, file),
      entry_type: entryType,
      digest: sha256(entryType === 'symlink' ? fs.readlinkSync(file) : fs.readFileSync(file)),
      supported_text: supportedText,
    };
  });
}

function compareInventories(beforeFiles, afterFiles) {
  const beforeByPath = new Map(beforeFiles.map((file) => [file.relative, file]));
  const afterByPath = new Map(afterFiles.map((file) => [file.relative, file]));
  const paths = [...new Set([...beforeByPath.keys(), ...afterByPath.keys()])].sort();
  const changes = [];

  for (const relative of paths) {
    const before = beforeByPath.get(relative);
    const after = afterByPath.get(relative);
    let kind;
    if (before === undefined) {
      kind = 'file-added';
    } else if (after === undefined) {
      kind = 'file-missing';
    } else if (before.entry_type !== after.entry_type || before.digest !== after.digest) {
      kind = 'file-changed';
    } else {
      continue;
    }

    changes.push({
      kind,
      file: relative,
      before_digest: before?.digest ?? '',
      after_digest: after?.digest ?? '',
      before_type: before?.entry_type ?? '',
      after_type: after?.entry_type ?? '',
      text_analysis: before?.supported_text || after?.supported_text ? 'supported' : 'unsupported',
    });
  }

  return changes;
}

function unsupportedEntries(files) {
  return files
    .filter((file) => !file.supported_text)
    .map((file) => ({ file: file.relative, entry_type: file.entry_type }));
}

function tokenize(file) {
  const tokens = [];
  const lines = fs.readFileSync(file, 'utf8').split('\n');
  const pattern = /[\p{L}\p{N}_]+(?:[-'’][\p{L}\p{N}_]+)*/gu;

  for (let lineIndex = 0; lineIndex < lines.length; lineIndex += 1) {
    for (const match of lines[lineIndex].matchAll(pattern)) {
      tokens.push({
        word: match[0].toLowerCase(),
        line: lineIndex + 1,
      });
    }
  }

  return tokens;
}

function readTree(root, label) {
  const files = walkTextFiles(root).map((file) => ({
    absolute: file,
    relative: path.relative(root, file),
    tokens: tokenize(file),
  }));

  if (files.length === 0) {
    throw new Error(`${label} directory contains no supported text files: ${root}`);
  }

  return files;
}

function countWords(files) {
  const counts = new Map();
  for (const file of files) {
    for (const { word } of file.tokens) {
      counts.set(word, (counts.get(word) ?? 0) + 1);
    }
  }
  return counts;
}

function totalWords(counts) {
  return [...counts.values()].reduce((total, count) => total + count, 0);
}

function windowKey(tokens, start) {
  return tokens
    .slice(start, start + WINDOW_SIZE)
    .map(({ word }) => word)
    .join(' ');
}

function countWindows(files) {
  const counts = new Map();
  for (const file of files) {
    for (let start = 0; start + WINDOW_SIZE <= file.tokens.length; start += WINDOW_STRIDE) {
      const key = windowKey(file.tokens, start);
      counts.set(key, (counts.get(key) ?? 0) + 1);
    }
  }
  return counts;
}

function findWordShortfalls(beforeCounts, afterCounts) {
  return [...beforeCounts.entries()]
    .filter(([word, before]) => (afterCounts.get(word) ?? 0) < before)
    .map(([word, before]) => ({
      word,
      before,
      after: afterCounts.get(word) ?? 0,
      shortfall: before - (afterCounts.get(word) ?? 0),
    }))
    .sort((left, right) => (
      right.shortfall - left.shortfall || compareCodeUnits(left.word, right.word)
    ));
}

function findMissingWindowClusters(beforeFiles, afterWindowCounts) {
  const consumed = new Map();
  const clusters = [];

  for (const file of beforeFiles) {
    const missingStarts = [];
    for (let start = 0; start + WINDOW_SIZE <= file.tokens.length; start += WINDOW_STRIDE) {
      const key = windowKey(file.tokens, start);
      const occurrence = (consumed.get(key) ?? 0) + 1;
      consumed.set(key, occurrence);
      if (occurrence > (afterWindowCounts.get(key) ?? 0)) {
        missingStarts.push(start);
      }
    }

    for (let cursor = 0; cursor < missingStarts.length;) {
      let end = cursor;
      while (
        end + 1 < missingStarts.length
        && missingStarts[end + 1] === missingStarts[end] + WINDOW_STRIDE
      ) {
        end += 1;
      }

      const startIndex = missingStarts[cursor];
      const endIndex = missingStarts[end];
      clusters.push({
        file: file.relative,
        start_line: file.tokens[startIndex].line,
        end_line: file.tokens[Math.min(endIndex + WINDOW_SIZE - 1, file.tokens.length - 1)].line,
        missing_windows: end - cursor + 1,
        sample: file.tokens
          .slice(startIndex, Math.min(endIndex + WINDOW_SIZE + 5, file.tokens.length))
          .map(({ word }) => word)
          .join(' '),
      });
      cursor = end + 1;
    }
  }

  return clusters;
}

function bounded(items, limit) {
  return {
    total: items.length,
    shown: items.slice(0, limit),
    omitted: Math.max(0, items.length - limit),
  };
}

function escapeTsv(value) {
  return String(value ?? '').replaceAll('\t', ' ').replaceAll('\n', ' ');
}

function printChecklist(inventoryChanges, wordShortfalls, windowClusters) {
  console.log(CHECKLIST_HEADER);

  for (const item of inventoryChanges) {
    console.log([
      item.kind,
      item.file,
      item.file,
      item.before_digest,
      item.after_digest,
      1,
      '',
      `${item.before_type || 'absent'} -> ${item.after_type || 'absent'}; text-analysis=${item.text_analysis}`,
      '',
    ].map(escapeTsv).join('\t'));
  }

  for (const item of wordShortfalls) {
    console.log([
      'word-shortfall',
      item.word,
      'aggregate',
      item.before,
      item.after,
      item.shortfall,
      '',
      '',
      '',
    ].map(escapeTsv).join('\t'));
  }

  for (const item of windowClusters) {
    console.log([
      'window-cluster',
      `${item.file}:${item.start_line}-${item.end_line}`,
      item.file,
      '',
      '',
      item.missing_windows,
      `${item.start_line}-${item.end_line}`,
      item.sample,
      '',
    ].map(escapeTsv).join('\t'));
  }
}

function verifyChecklist(checklistPath) {
  let contents;
  try {
    contents = fs.readFileSync(checklistPath, 'utf8');
  } catch (error) {
    throw new Error(`cannot read checklist ${checklistPath}: ${error.message}`);
  }

  const lines = contents.split(/\r?\n/);
  if (lines.at(-1) === '') {
    lines.pop();
  }
  if (lines.length === 0) {
    throw new Error(`checklist is empty: ${checklistPath}`);
  }

  const header = lines.shift();
  if (header !== CHECKLIST_HEADER) {
    throw new Error([
      `checklist header mismatch: ${checklistPath}`,
      `expected: ${JSON.stringify(CHECKLIST_HEADER)}`,
      `received: ${JSON.stringify(header)}`,
    ].join('\n'));
  }

  const errors = [];
  let filledDispositions = 0;
  for (let index = 0; index < lines.length; index += 1) {
    const lineNumber = index + 2;
    const fields = lines[index].split('\t');
    if (fields.length !== CHECKLIST_COLUMNS.length) {
      errors.push(`line ${lineNumber}: expected ${CHECKLIST_COLUMNS.length} columns, found ${fields.length}`);
      continue;
    }

    const kind = fields[0];
    if (!CHECKLIST_KINDS.has(kind)) {
      errors.push(`line ${lineNumber}: unknown kind ${JSON.stringify(kind)}`);
    }
    if (fields[CHECKLIST_COLUMNS.length - 1].trim() === '') {
      errors.push(`line ${lineNumber}: disposition is blank`);
    } else {
      filledDispositions += 1;
    }
  }

  if (errors.length > 0) {
    const shown = errors.slice(0, CHECKLIST_ERROR_LIMIT);
    const omitted = errors.length - shown.length;
    throw new Error([
      `checklist verification failed with ${errors.length} error(s): ${checklistPath}`,
      ...shown,
      ...(omitted > 0 ? [`... ${omitted} additional error(s) omitted`] : []),
    ].join('\n'));
  }

  console.log(JSON.stringify({
    checklist: checklistPath,
    rows: lines.length,
    filled_dispositions: filledDispositions,
    blank_dispositions: 0,
  }, null, 2));
}

function main() {
  const options = parseArguments(process.argv.slice(2));
  if (options.help) {
    console.log(usage());
    return;
  }

  if (options.mode === 'verify-checklist') {
    verifyChecklist(options.checklistPath);
    return;
  }

  assertDirectory(options.beforeRoot, 'before');
  assertDirectory(options.afterRoot, 'after');

  const beforeInventory = inventoryTree(options.beforeRoot);
  const afterInventory = inventoryTree(options.afterRoot);
  const inventoryChanges = compareInventories(beforeInventory, afterInventory);
  const beforeFiles = readTree(options.beforeRoot, 'before');
  const afterFiles = readTree(options.afterRoot, 'after');
  const beforeCounts = countWords(beforeFiles);
  const afterCounts = countWords(afterFiles);
  const wordShortfalls = findWordShortfalls(beforeCounts, afterCounts);
  const windowClusters = findMissingWindowClusters(beforeFiles, countWindows(afterFiles));

  if (options.mode === 'checklist') {
    printChecklist(inventoryChanges, wordShortfalls, windowClusters);
    return;
  }

  console.log(JSON.stringify({
    contract_version: 2,
    scanned_extensions: [...TEXT_EXTENSIONS].sort(),
    file_inventory: {
      before: {
        total: beforeInventory.length,
        unsupported_for_text_analysis: bounded(unsupportedEntries(beforeInventory), options.limit),
      },
      after: {
        total: afterInventory.length,
        unsupported_for_text_analysis: bounded(unsupportedEntries(afterInventory), options.limit),
      },
      changes: bounded(inventoryChanges, options.limit),
    },
    tokenization: 'Unicode letters, numbers, and underscore; internal hyphen or apostrophe; lowercase',
    window: {
      size: WINDOW_SIZE,
      stride: WINDOW_STRIDE,
      interpretation: 'A surviving seam can remove up to size - 1 overlapping windows.',
    },
    before: {
      root: options.beforeRoot,
      files: beforeFiles.length,
      words: totalWords(beforeCounts),
    },
    after: {
      root: options.afterRoot,
      files: afterFiles.length,
      words: totalWords(afterCounts),
    },
    word_shortfalls: bounded(wordShortfalls, options.limit),
    missing_window_clusters: bounded(windowClusters, options.limit),
    next_step: 'Run checklist mode into the session scratchpad and fill every inventory, word, and window disposition cell.',
  }, null, 2));
}

try {
  main();
} catch (error) {
  console.error(`preservation-audit: ${error.message}`);
  process.exitCode = 2;
}
