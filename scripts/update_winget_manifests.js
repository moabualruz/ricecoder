#!/usr/bin/env node
// Script to update Winget manifests with latest release information

const https = require('https');
const crypto = require('crypto');
const fs = require('fs');
const path = require('path');

// Configuration
const REPO = 'moabualruz/ricecoder';
const INSTALLER_MANIFEST = 'winget/RiceCoder.RiceCoder.installer.yaml';
const LOCALE_MANIFEST = 'winget/RiceCoder.RiceCoder.locale.en-US.yaml';

// Get latest release info from GitHub API
function getLatestRelease() {
  return new Promise((resolve, reject) => {
    const options = {
      hostname: 'api.github.com',
      path: `/repos/${REPO}/releases/latest`,
      headers: {
        'User-Agent': 'ricecoder-update-script'
      }
    };

    https.get(options, (res) => {
      let data = '';
      res.on('data', (chunk) => data += chunk);
      res.on('end', () => {
        try {
          resolve(JSON.parse(data));
        } catch (e) {
          reject(e);
        }
      });
    }).on('error', reject);
  });
}

// Download file and calculate SHA256
function calculateSHA256(url) {
  return new Promise((resolve, reject) => {
    https.get(url, (res) => {
      const hash = crypto.createHash('sha256');
      res.on('data', (chunk) => hash.update(chunk));
      res.on('end', () => resolve(hash.digest('hex')));
    }).on('error', reject);
  });
}

// Update Winget installer manifest
async function updateInstallerManifest(version, assets) {
  let manifest = fs.readFileSync(INSTALLER_MANIFEST, 'utf8');

  // Update version
  manifest = manifest.replace(/PackageVersion: \d+\.\d+\.\d+/, `PackageVersion: ${version}`);

  // Find Windows x86_64 asset
  const windowsAsset = assets.find(asset => asset.name.includes('windows-x86_64.zip'));
  if (windowsAsset) {
    const sha256 = await calculateSHA256(windowsAsset.browser_download_url);
    manifest = manifest.replace(/InstallerUrl: .*/, `InstallerUrl: ${windowsAsset.browser_download_url}`);
    manifest = manifest.replace(/InstallerSha256: .*/, `InstallerSha256: ${sha256}`);
  }

  fs.writeFileSync(INSTALLER_MANIFEST, manifest);
  console.log(`Updated Winget installer manifest to version ${version}`);
}

// Update Winget locale manifest
function updateLocaleManifest(version) {
  let manifest = fs.readFileSync(LOCALE_MANIFEST, 'utf8');

  // Update version
  manifest = manifest.replace(/PackageVersion: \d+\.\d+\.\d+/, `PackageVersion: ${version}`);

  // Update release notes URL
  manifest = manifest.replace(/ReleaseNotesUrl: .*/, `ReleaseNotesUrl: https://github.com/${REPO}/releases/tag/v${version}`);

  fs.writeFileSync(LOCALE_MANIFEST, manifest);
  console.log(`Updated Winget locale manifest to version ${version}`);
}

// Main execution
async function main() {
  try {
    const release = await getLatestRelease();
    const version = release.tag_name.replace('v', '');
    const assets = release.assets;

    await updateInstallerManifest(version, assets);
    updateLocaleManifest(version);
  } catch (e) {
    console.error(`Error updating Winget manifests: ${e.message}`);
    process.exit(1);
  }
}

main();