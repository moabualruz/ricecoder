/**
 * Test: npm Package Structure Verification
 * 
 * Feature: ricecoder-installation, Property 1: Installation Completeness
 * Validates: Requirements 2.1, 2.2, 7.6
 * 
 * This test verifies that the npm package is correctly structured for publishing
 * and that all necessary files are included.
 */

const fs = require('fs');
const path = require('path');

describe('npm Package Structure', () => {
  const projectRoot = path.join(__dirname, '..');
  const packageJsonPath = path.join(projectRoot, 'package.json');
  const npmIgnorePath = path.join(projectRoot, '.npmignore');
  const scriptsDir = path.join(projectRoot, 'scripts');
  const binDir = path.join(projectRoot, 'bin');

  test('package.json exists and is valid JSON', () => {
    expect(fs.existsSync(packageJsonPath)).toBe(true);
    
    const content = fs.readFileSync(packageJsonPath, 'utf-8');
    const packageJson = JSON.parse(content);
    
    expect(packageJson).toBeDefined();
    expect(packageJson.name).toBe('ricecoder');
    expect(packageJson.version).toBeDefined();
  });

  test('package.json has required metadata fields', () => {
    const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf-8'));
    
    // Required fields
    expect(packageJson.name).toBeDefined();
    expect(packageJson.version).toBeDefined();
    expect(packageJson.description).toBeDefined();
    expect(packageJson.license).toBeDefined();
    expect(packageJson.repository).toBeDefined();
    expect(packageJson.author).toBeDefined();
    
    // npm-specific fields
    expect(packageJson.bin).toBeDefined();
    expect(packageJson.bin.ricecoder).toBeDefined();
    expect(packageJson.scripts).toBeDefined();
    expect(packageJson.scripts.postinstall).toBeDefined();
  });

  test('package.json has correct bin configuration', () => {
    const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf-8'));
    
    expect(packageJson.bin.ricecoder).toBe('./bin/ricecoder');
    expect(packageJson.bin.rice).toBe('./bin/ricecoder');
  });

  test('package.json has postinstall script configured', () => {
    const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf-8'));
    
    expect(packageJson.scripts.postinstall).toBe('node scripts/install.js');
  });

  test('package.json includes necessary files in files array', () => {
    const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf-8'));
    
    expect(packageJson.files).toContain('bin');
    expect(packageJson.files).toContain('scripts');
    expect(packageJson.files).toContain('README.md');
    expect(packageJson.files).toContain('LICENSE.md');
  });

  test('package.json has platform and CPU specifications', () => {
    const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf-8'));
    
    expect(packageJson.os).toContain('linux');
    expect(packageJson.os).toContain('darwin');
    expect(packageJson.os).toContain('win32');
    
    expect(packageJson.cpu).toContain('x64');
    expect(packageJson.cpu).toContain('arm64');
  });

  test('package.json has publishConfig set to public', () => {
    const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf-8'));
    
    expect(packageJson.publishConfig).toBeDefined();
    expect(packageJson.publishConfig.access).toBe('public');
  });

  test('package.json has adm-zip dependency for Windows support', () => {
    const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf-8'));
    
    expect(packageJson.dependencies).toBeDefined();
    expect(packageJson.dependencies['adm-zip']).toBeDefined();
  });

  test('.npmignore file exists', () => {
    expect(fs.existsSync(npmIgnorePath)).toBe(true);
  });

  test('.npmignore excludes unnecessary files', () => {
    const content = fs.readFileSync(npmIgnorePath, 'utf-8');
    
    // Should exclude build artifacts
    expect(content).toContain('target/');
    expect(content).toContain('.git/');
    
    // Should exclude Rust files
    expect(content).toContain('Cargo.toml');
    expect(content).toContain('*.rs');
    
    // Should exclude test files
    expect(content).toContain('*.log');
  });

  test('scripts/install.js exists and is executable', () => {
    const installScriptPath = path.join(scriptsDir, 'install.js');
    expect(fs.existsSync(installScriptPath)).toBe(true);
    
    const content = fs.readFileSync(installScriptPath, 'utf-8');
    expect(content).toContain('#!/usr/bin/env node');
  });

  test('scripts/install.js has platform detection', () => {
    const installScriptPath = path.join(scriptsDir, 'install.js');
    const content = fs.readFileSync(installScriptPath, 'utf-8');
    
    expect(content).toContain('process.platform');
    expect(content).toContain('process.arch');
    expect(content).toContain('targetMap');
  });

  test('scripts/install.js has checksum verification', () => {
    const installScriptPath = path.join(scriptsDir, 'install.js');
    const content = fs.readFileSync(installScriptPath, 'utf-8');
    
    expect(content).toContain('sha256');
    expect(content).toContain('checksum');
    expect(content).toContain('crypto');
  });

  test('scripts/install.js handles all supported platforms', () => {
    const installScriptPath = path.join(scriptsDir, 'install.js');
    const content = fs.readFileSync(installScriptPath, 'utf-8');
    
    // Check for all target mappings
    expect(content).toContain('x86_64-unknown-linux-musl');
    expect(content).toContain('aarch64-unknown-linux-musl');
    expect(content).toContain('x86_64-apple-darwin');
    expect(content).toContain('aarch64-apple-darwin');
    expect(content).toContain('x86_64-pc-windows-msvc');
    expect(content).toContain('aarch64-pc-windows-msvc');
  });

  test('bin directory exists', () => {
    expect(fs.existsSync(binDir)).toBe(true);
  });

  test('README.md exists', () => {
    const readmePath = path.join(projectRoot, 'README.md');
    expect(fs.existsSync(readmePath)).toBe(true);
  });

  test('LICENSE.md exists', () => {
    const licensePath = path.join(projectRoot, 'LICENSE.md');
    expect(fs.existsSync(licensePath)).toBe(true);
  });

  test('package.json version matches expected format', () => {
    const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf-8'));
    
    // Version should be in semver format (e.g., 0.1.6)
    expect(packageJson.version).toMatch(/^\d+\.\d+\.\d+/);
  });

  test('package.json has keywords for npm discoverability', () => {
    const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf-8'));
    
    expect(packageJson.keywords).toBeDefined();
    expect(packageJson.keywords.length).toBeGreaterThan(0);
    expect(packageJson.keywords).toContain('ai');
    expect(packageJson.keywords).toContain('cli');
  });

  test('package.json has repository URL', () => {
    const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf-8'));
    
    expect(packageJson.repository).toBeDefined();
    expect(packageJson.repository.type).toBe('git');
    expect(packageJson.repository.url).toContain('github.com');
    expect(packageJson.repository.url).toContain('ricecoder');
  });

  test('package.json has homepage and bugs URLs', () => {
    const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf-8'));
    
    expect(packageJson.homepage).toBeDefined();
    expect(packageJson.bugs).toBeDefined();
    expect(packageJson.bugs.url).toBeDefined();
  });
});
