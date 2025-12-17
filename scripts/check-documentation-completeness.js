#!/usr/bin/env node

/**
 * RiceCoder Documentation Completeness Checker
 *
 * Checks that all public APIs have proper documentation
 * and validates documentation standards compliance.
 */

const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

const WORKSPACE_ROOT = process.cwd();
const CRATES_DIR = path.join(WORKSPACE_ROOT, 'crates');

/**
 * Find all crate directories
 */
function findCrates() {
  const crates = [];

  if (!fs.existsSync(CRATES_DIR)) {
    console.error(`Crates directory not found: ${CRATES_DIR}`);
    return crates;
  }

  const entries = fs.readdirSync(CRATES_DIR, { withFileTypes: true });

  for (const entry of entries) {
    if (entry.isDirectory()) {
      const cratePath = path.join(CRATES_DIR, entry.name);
      const cargoToml = path.join(cratePath, 'Cargo.toml');

      if (fs.existsSync(cargoToml)) {
        crates.push({
          name: entry.name,
          path: cratePath,
          cargoToml: cargoToml
        });
      }
    }
  }

  return crates;
}

/**
 * Check documentation completeness for a crate
 */
function checkCrateDocumentation(crate) {
  console.log(`\n🔍 Checking documentation for crate: ${crate.name}`);

  const issues = [];

  try {
    // Run cargo doc with warnings
    const output = execSync(`cd "${crate.path}" && cargo doc --no-deps --all-features --document-private-items 2>&1`, {
      encoding: 'utf8',
      timeout: 30000
    });

    // Check for documentation warnings
    const warnings = output.split('\n').filter(line =>
      line.includes('warning:') &&
      (line.includes('missing documentation') ||
       line.includes('rustdoc') ||
       line.includes('unresolved link'))
    );

    if (warnings.length > 0) {
      issues.push({
        type: 'documentation_warnings',
        crate: crate.name,
        warnings: warnings
      });
    }

    // Check for undocumented public items
    const undocumented = output.split('\n').filter(line =>
      line.includes('missing documentation for')
    );

    if (undocumented.length > 0) {
      issues.push({
        type: 'undocumented_public_items',
        crate: crate.name,
        items: undocumented
      });
    }

  } catch (error) {
    issues.push({
      type: 'build_error',
      crate: crate.name,
      error: error.message
    });
  }

  return issues;
}

/**
 * Check README and CHANGELOG presence
 */
function checkDocumentationFiles(crate) {
  const issues = [];

  const readmePath = path.join(crate.path, 'README.md');
  const changelogPath = path.join(crate.path, 'CHANGELOG.md');

  if (!fs.existsSync(readmePath)) {
    issues.push({
      type: 'missing_readme',
      crate: crate.name,
      path: readmePath
    });
  }

  if (!fs.existsSync(changelogPath)) {
    issues.push({
      type: 'missing_changelog',
      crate: crate.name,
      path: changelogPath
    });
  }

  return issues;
}

/**
 * Validate documentation standards
 */
function validateDocumentationStandards(crate) {
  const issues = [];

  // Check README content if it exists
  const readmePath = path.join(crate.path, 'README.md');
  if (fs.existsSync(readmePath)) {
    try {
      const content = fs.readFileSync(readmePath, 'utf-8');

      // Check for required sections
      const requiredSections = ['Description', 'Usage', 'License'];
      for (const section of requiredSections) {
        if (!content.toLowerCase().includes(section.toLowerCase())) {
          issues.push({
            type: 'readme_missing_section',
            crate: crate.name,
            section: section
          });
        }
      }

      // Check for code examples
      if (!content.includes('```')) {
        issues.push({
          type: 'readme_no_code_examples',
          crate: crate.name
        });
      }

    } catch (error) {
      issues.push({
        type: 'readme_read_error',
        crate: crate.name,
        error: error.message
      });
    }
  }

  return issues;
}

/**
 * Generate report
 */
function generateReport(allIssues) {
  const report = {
    timestamp: new Date().toISOString(),
    totalCrates: 0,
    issues: allIssues,
    summary: {
      totalIssues: allIssues.length,
      byType: {},
      byCrate: {}
    }
  };

  // Calculate summaries
  for (const issue of allIssues) {
    report.summary.byType[issue.type] = (report.summary.byType[issue.type] || 0) + 1;
    report.summary.byCrate[issue.crate] = (report.summary.byCrate[issue.crate] || 0) + 1;
  }

  return report;
}

/**
 * Format report as markdown
 */
function formatReportAsMarkdown(report) {
  let markdown = `# RiceCoder Documentation Completeness Report

**Date**: ${report.timestamp}
**Total Crates Checked**: ${report.totalCrates}

## Summary

- Total issues found: ${report.summary.totalIssues}
- Issues by type: ${JSON.stringify(report.summary.byType, null, 2)}
- Issues by crate: ${JSON.stringify(report.summary.byCrate, null, 2)}

`;

  if (report.issues.length > 0) {
    markdown += `## Issues Found

`;

    for (const issue of report.issues) {
      markdown += `### ${issue.crate}: ${issue.type}

`;

      if (issue.warnings) {
        markdown += `Warnings:\n${issue.warnings.map(w => `- ${w}`).join('\n')}\n\n`;
      }

      if (issue.items) {
        markdown += `Undocumented items:\n${issue.items.map(i => `- ${i}`).join('\n')}\n\n`;
      }

      if (issue.error) {
        markdown += `Error: ${issue.error}\n\n`;
      }

      if (issue.path) {
        markdown += `Path: ${issue.path}\n\n`;
      }

      if (issue.section) {
        markdown += `Missing section: ${issue.section}\n\n`;
      }
    }
  } else {
    markdown += `## ✅ All Checks Passed

All crates have complete documentation that meets RiceCoder standards.

`;
  }

  markdown += `## Standards Checked

- ✅ Public API documentation completeness
- ✅ README.md presence and content
- ✅ CHANGELOG.md presence
- ✅ Code examples in documentation
- ✅ Required README sections

## Recommendations

`;

  if (report.summary.totalIssues > 0) {
    markdown += `- **High Priority**: Fix missing README.md and CHANGELOG.md files\n`;
    markdown += `- **Medium Priority**: Add documentation for undocumented public APIs\n`;
    markdown += `- **Low Priority**: Improve README content and add more examples\n`;
  } else {
    markdown += `- ✅ All documentation standards are met\n`;
  }

  return markdown;
}

/**
 * Main function
 */
async function checkDocumentationCompleteness() {
  console.log('Starting RiceCoder documentation completeness check...\n');

  // Find all crates
  const crates = findCrates();
  console.log(`Found ${crates.length} crates to check\n`);

  const allIssues = [];

  // Check each crate
  for (const crate of crates) {
    // Check documentation completeness
    const docIssues = checkCrateDocumentation(crate);
    allIssues.push(...docIssues);

    // Check documentation files
    const fileIssues = checkDocumentationFiles(crate);
    allIssues.push(...fileIssues);

    // Validate documentation standards
    const standardIssues = validateDocumentationStandards(crate);
    allIssues.push(...standardIssues);
  }

  // Generate report
  const report = generateReport(allIssues);
  report.totalCrates = crates.length;

  // Output report
  const reportMarkdown = formatReportAsMarkdown(report);
  console.log(reportMarkdown);

  // Save report
  const reportPath = path.join(WORKSPACE_ROOT, 'projects', 'ricecoder', '.kiro', 'docs-completeness-report.md');
  fs.mkdirSync(path.dirname(reportPath), { recursive: true });
  fs.writeFileSync(reportPath, reportMarkdown);

  const jsonReportPath = path.join(WORKSPACE_ROOT, 'projects', 'ricecoder', '.kiro', 'docs-completeness-report.json');
  fs.writeFileSync(jsonReportPath, JSON.stringify(report, null, 2));

  console.log(`\nReports saved to:`);
  console.log(`- ${reportPath}`);
  console.log(`- ${jsonReportPath}`);

  // Exit with appropriate code
  process.exit(report.summary.totalIssues === 0 ? 0 : 1);
}

// Run the check
checkDocumentationCompleteness().catch((error) => {
  console.error('Documentation completeness check failed:', error);
  process.exit(1);
});