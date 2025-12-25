#!/usr/bin/env node

/**
 * RiceCoder Documentation Link Validation Script
 *
 * Validates all markdown links in RiceCoder documentation:
 * - docs/ directory
 * - README.md, CONTRIBUTING.md, etc.
 * - Cross-references between documentation files
 *
 * Identifies:
 * - Broken links (files that don't exist)
 * - Circular references (A → B → C → A)
 * - Orphaned files (no incoming references)
 * - Malformed links
 */

const fs = require('fs');
const path = require('path');

const DOCUMENTATION_DIRS = [
  'docs',
  '.',  // For root documentation files
];

const WORKSPACE_ROOT = process.cwd();

const IGNORE_FILES = [
  'node_modules',
  'target',
  '.git',
  '.github',
  '.cargo',
  '.ai',
  'learning',
  'projects/automation',
  'projects/ricecoder.wiki',
];

/**
 * Find all markdown files in documentation directories
 */
function findMarkdownFiles() {
  const files = [];

  function shouldIgnore(filePath) {
    return IGNORE_FILES.some(ignore => filePath.includes(ignore));
  }

  function walkDir(currentPath) {
    if (shouldIgnore(currentPath)) {
      return;
    }

    const entries = fs.readdirSync(currentPath, { withFileTypes: true });

    for (const entry of entries) {
      const fullEntryPath = path.join(currentPath, entry.name);
      const relativePath = path.relative(WORKSPACE_ROOT, fullEntryPath);

      if (shouldIgnore(relativePath)) {
        continue;
      }

      if (entry.isDirectory()) {
        // Check if this directory should be included
        const dirName = path.basename(currentPath);
        if (DOCUMENTATION_DIRS.includes(dirName) || DOCUMENTATION_DIRS.includes(relativePath)) {
          walkDir(fullEntryPath);
        }
      } else if (entry.isFile() && entry.name.endsWith('.md')) {
        // Include root level documentation files
        if (DOCUMENTATION_DIRS.includes('.') && currentPath === WORKSPACE_ROOT) {
          files.push(relativePath);
        }
        // Include docs directory files
        else if (relativePath.startsWith('docs/')) {
          files.push(relativePath);
        }
      }
    }
  }

  walkDir(WORKSPACE_ROOT);
  return files.sort();
}

/**
 * Extract all markdown links from a file
 */
function extractLinks(filePath) {
  const content = fs.readFileSync(filePath, 'utf-8');
  const lines = content.split('\n');
  const links = [];

  // Regex to match markdown links: [text](url)
  const linkRegex = /\[([^\]]+)\]\(([^)]+)\)/g;

  for (let lineNum = 0; lineNum < lines.length; lineNum++) {
    const line = lines[lineNum];
    let match;

    while ((match = linkRegex.exec(line)) !== null) {
      const text = match[1];
      const target = match[2];

      // Determine link type
      let type = 'relative';
      if (target.startsWith('http://') || target.startsWith('https://')) {
        type = 'external';
      } else if (target.startsWith('#')) {
        type = 'anchor';
      } else if (target.startsWith('/')) {
        type = 'absolute';
      }

      links.push({
        file: filePath,
        line: lineNum + 1,
        text,
        target,
        type,
      });
    }
  }

  return links;
}

/**
 * Validate a single link
 */
function validateLink(link) {
  // Skip external links and anchor-only links
  if (link.type === 'external') {
    return { valid: true, reason: 'External link (not validated)' };
  }

  if (link.type === 'anchor') {
    return { valid: true, reason: 'Anchor link (not validated)' };
  }

  // Extract file path from target (remove anchor if present)
  const targetPath = link.target.split('#')[0];

  if (!targetPath) {
    return { valid: true, reason: 'Anchor-only link' };
  }

  // Resolve relative path
  let resolvedPath;
  if (link.type === 'absolute') {
    resolvedPath = path.join(WORKSPACE_ROOT, targetPath);
  } else {
    const linkDir = path.dirname(link.file);
    resolvedPath = path.resolve(path.join(WORKSPACE_ROOT, linkDir, targetPath));
  }

  // Normalize path
  resolvedPath = path.normalize(resolvedPath);

  // Check if file exists
  if (!fs.existsSync(resolvedPath)) {
    return {
      valid: false,
      reason: 'File not found',
      resolvedPath: path.relative(WORKSPACE_ROOT, resolvedPath),
    };
  }

  // Check if it's a file or directory
  const stats = fs.statSync(resolvedPath);
  if (!stats.isFile() && !stats.isDirectory()) {
    return {
      valid: false,
      reason: 'Not a file or directory',
      resolvedPath: path.relative(WORKSPACE_ROOT, resolvedPath),
    };
  }

  return {
    valid: true,
    resolvedPath: path.relative(WORKSPACE_ROOT, resolvedPath),
  };
}

/**
 * Build reference graph for circular reference detection
 */
function buildReferenceGraph(allLinks) {
  const graph = new Map();

  for (const link of allLinks) {
    if (link.type === 'external' || link.type === 'anchor') {
      continue;
    }

    const targetPath = link.target.split('#')[0];
    if (!targetPath) {
      continue;
    }

    // Resolve target path
    let resolvedTarget;
    if (link.type === 'absolute') {
      resolvedTarget = path.join(WORKSPACE_ROOT, targetPath);
    } else {
      const linkDir = path.dirname(link.file);
      resolvedTarget = path.resolve(path.join(WORKSPACE_ROOT, linkDir, targetPath));
    }

    resolvedTarget = path.normalize(resolvedTarget);
    const relativeTarget = path.relative(WORKSPACE_ROOT, resolvedTarget);

    if (!graph.has(link.file)) {
      graph.set(link.file, new Set());
    }

    graph.get(link.file).add(relativeTarget);
  }

  return graph;
}

/**
 * Detect circular references using DFS
 */
function detectCircularReferences(graph) {
  const cycles = [];
  const visited = new Set();
  const recursionStack = new Set();
  const pathArray = [];

  const dfs = (node) => {
    visited.add(node);
    recursionStack.add(node);
    pathArray.push(node);

    const neighbors = graph.get(node) || new Set();

    for (const neighbor of neighbors) {
      if (!visited.has(neighbor)) {
        dfs(neighbor);
      } else if (recursionStack.has(neighbor)) {
        // Found a cycle
        const cycleStart = pathArray.indexOf(neighbor);
        const cycle = pathArray.slice(cycleStart);
        cycle.push(neighbor);
        const chainStr = cycle.join(' → ');
        cycles.push({
          chain: chainStr,
          files: cycle,
        });
      }
    }

    pathArray.pop();
    recursionStack.delete(node);
  };

  for (const node of graph.keys()) {
    if (!visited.has(node)) {
      dfs(node);
    }
  }

  return cycles;
}

/**
 * Identify orphaned files (no incoming references)
 */
function findOrphanedFiles(allFiles, allLinks) {
  const referencedFiles = new Set();

  for (const link of allLinks) {
    if (link.type === 'external' || link.type === 'anchor') {
      continue;
    }

    const targetPath = link.target.split('#')[0];
    if (!targetPath) {
      continue;
    }

    // Resolve target path
    let resolvedTarget;
    if (link.type === 'absolute') {
      resolvedTarget = path.join(WORKSPACE_ROOT, targetPath);
    } else {
      const linkDir = path.dirname(link.file);
      resolvedTarget = path.resolve(path.join(WORKSPACE_ROOT, linkDir, targetPath));
    }

    resolvedTarget = path.normalize(resolvedTarget);
    const relativeTarget = path.relative(WORKSPACE_ROOT, resolvedTarget);

    referencedFiles.add(relativeTarget);
  }

  const orphaned = allFiles.filter((file) => !referencedFiles.has(file));

  // Filter out common non-documentation files
  return orphaned.filter((file) => {
    const fileName = path.basename(file);
    return !['README.md', 'CONTRIBUTING.md', 'CHANGELOG.md', 'LICENSE.md'].includes(fileName);
  });
}

/**
 * Generate validation report
 */
function generateReport(
  filesChecked,
  allLinks,
  brokenLinks,
  circularReferences,
  orphanedFiles
) {
  return {
    timestamp: new Date().toISOString(),
    totalFilesChecked: filesChecked.length,
    totalLinksChecked: allLinks.length,
    brokenLinks,
    circularReferences,
    orphanedFiles,
    summary: {
      brokenCount: brokenLinks.length,
      circularCount: circularReferences.length,
      orphanedCount: orphanedFiles.length,
      status:
        brokenLinks.length === 0 && circularReferences.length === 0
          ? 'PASS'
          : 'FAIL',
    },
  };
}

/**
 * Format report as markdown
 */
function formatReportAsMarkdown(report) {
  let markdown = `# RiceCoder Documentation Validation Report

**Date**: ${report.timestamp}
**Validator**: RiceCoder Documentation Validation Script

## Summary

- Total files checked: ${report.totalFilesChecked}
- Total links checked: ${report.totalLinksChecked}
- Broken links found: ${report.summary.brokenCount}
- Circular references found: ${report.summary.circularCount}
- Orphaned files found: ${report.summary.orphanedCount}
- **Overall Status**: ${report.summary.status}

`;

  if (report.brokenLinks.length > 0) {
    markdown += `## Broken Links

| File | Line | Link Text | Target | Issue |
|------|------|-----------|--------|-------|
`;
    for (const link of report.brokenLinks) {
      markdown += `| ${link.file} | ${link.line} | ${link.text} | ${link.target} | ${link.issue} |\n`;
    }
    markdown += '\n';
  }

  if (report.circularReferences.length > 0) {
    markdown += `## Circular References

| Chain | Files |
|-------|-------|
`;
    for (const ref of report.circularReferences) {
      markdown += `| ${ref.chain} | ${ref.files.join(', ')} |\n`;
    }
    markdown += '\n';
  }

  if (report.orphanedFiles.length > 0) {
    markdown += `## Orphaned Files

| File |
|------|
`;
    for (const file of report.orphanedFiles) {
      markdown += `| ${file} |\n`;
    }
    markdown += '\n';
  }

  markdown += `## Recommendations

`;

  if (report.brokenLinks.length > 0) {
    markdown += `- **High Priority**: Fix all ${report.brokenLinks.length} broken links\n`;
  }

  if (report.circularReferences.length > 0) {
    markdown += `- **Medium Priority**: Review ${report.circularReferences.length} circular references\n`;
  }

  if (report.orphanedFiles.length > 0) {
    markdown += `- **Low Priority**: Review ${report.orphanedFiles.length} orphaned files\n`;
  }

  if (report.summary.status === 'PASS') {
    markdown += `- ✓ All documentation links are valid\n`;
  }

  return markdown;
}

/**
 * Main validation function
 */
async function validateDocumentation() {
  console.log('Starting RiceCoder documentation link validation...\n');

  // Step 1: Find all markdown files
  console.log('Step 1: Identifying all markdown files...');
  const markdownFiles = findMarkdownFiles();
  console.log(`Found ${markdownFiles.length} markdown files\n`);

  // Step 2: Extract all links
  console.log('Step 2: Extracting all markdown links...');
  const allLinks = [];
  for (const file of markdownFiles) {
    try {
      const links = extractLinks(file);
      allLinks.push(...links);
    } catch (error) {
      console.warn(`Warning: Could not read file ${file}: ${error.message}`);
    }
  }
  console.log(`Found ${allLinks.length} total links\n`);

  // Step 3: Validate each link
  console.log('Step 3: Validating each link...');
  const brokenLinks = [];

  for (const link of allLinks) {
    const result = validateLink(link);
    if (!result.valid) {
      brokenLinks.push({
        file: link.file,
        line: link.line,
        text: link.text,
        target: link.target,
        issue: result.reason || 'Unknown error',
      });
    }
  }
  console.log(`Found ${brokenLinks.length} broken links\n`);

  // Step 4: Detect circular references
  console.log('Step 4: Detecting circular references...');
  const graph = buildReferenceGraph(allLinks);
  const circularReferences = detectCircularReferences(graph);
  console.log(`Found ${circularReferences.length} circular references\n`);

  // Step 5: Identify orphaned files
  console.log('Step 5: Identifying orphaned files...');
  const orphanedFiles = findOrphanedFiles(markdownFiles, allLinks);
  console.log(`Found ${orphanedFiles.length} orphaned files\n`);

  // Step 6: Generate report
  console.log('Step 6: Generating validation report...');
  const report = generateReport(
    markdownFiles,
    allLinks,
    brokenLinks,
    circularReferences,
    orphanedFiles
  );

  // Output report
  const reportMarkdown = formatReportAsMarkdown(report);
  console.log(reportMarkdown);

  // Save report to file
  const reportPath = path.join(WORKSPACE_ROOT, 'projects', 'ricecoder', '.ai', 'docs-validation-report.md');
  fs.mkdirSync(path.dirname(reportPath), { recursive: true });
  fs.writeFileSync(reportPath, reportMarkdown);
  console.log(`\nValidation report saved to: ${reportPath}`);

  // Save JSON report
  const jsonReportPath = path.join(
    WORKSPACE_ROOT,
    'projects',
    'ricecoder',
    '.ai',
    'docs-validation-report.json'
  );
  fs.writeFileSync(jsonReportPath, JSON.stringify(report, null, 2));
  console.log(`JSON report saved to: ${jsonReportPath}`);

  // Exit with appropriate code
  process.exit(report.summary.status === 'PASS' ? 0 : 1);
}

// Run validation
validateDocumentation().catch((error) => {
  console.error('Validation failed:', error);
  process.exit(1);
});