#!/usr/bin/env node
// Script to update Scoop manifest with latest release information

const https = require('https');
const crypto = require('crypto');
const fs = require('fs');
const path = require('path');

// Configuration
const REPO = 'moabualruz/ricecoder';
const MANIFEST_PATH = 'scoop/ricecoder.json';

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

// Update Scoop manifest
async function updateManifest(version, assets) {
  const manifest = JSON.parse(fs.readFileSync(MANIFEST_PATH, 'utf8'));

  // Update version
  manifest.version = version;

  // Find Windows x86_64 asset
  const windowsAsset = assets.find(asset => asset.name.includes('windows-x86_64.zip'));
  if (windowsAsset) {
    const sha256 = await calculateSHA256(windowsAsset.browser_download_url);
    manifest.architecture['64bit'].url = windowsAsset.browser_download_url;
    manifest.architecture['64bit'].hash = sha256;
  }

  // Update autoupdate URL
  manifest.autoupdate.architecture['64bit'].url = `https://github.com/${REPO}/releases/download/v$version/ricecoder-$version-windows-x86_64.zip`;

  fs.writeFileSync(MANIFEST_PATH, JSON.stringify(manifest, null, 4));
  console.log(`Updated Scoop manifest to version ${version}`);
}

// Main execution
async function main() {
  try {
    const release = await getLatestRelease();
    const version = release.tag_name.replace('v', '');
    const assets = release.assets;

    await updateManifest(version, assets);
  } catch (e) {
    console.error(`Error updating Scoop manifest: ${e.message}`);
    process.exit(1);
  }
}

main();