#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const https = require('https');
const crypto = require('crypto');
const { execSync } = require('child_process');

// Determine platform and architecture
const platform = process.platform;
const arch = process.arch;

// Map Node.js platform/arch to Rust target
const targetMap = {
  'linux-x64': 'x86_64-unknown-linux-musl',
  'linux-arm64': 'aarch64-unknown-linux-musl',
  'darwin-x64': 'x86_64-apple-darwin',
  'darwin-arm64': 'aarch64-apple-darwin',
  'win32-x64': 'x86_64-pc-windows-msvc',
  'win32-arm64': 'aarch64-pc-windows-msvc',
};

const key = `${platform}-${arch}`;
const target = targetMap[key];

if (!target) {
  console.error(`Unsupported platform/architecture: ${key}`);
  process.exit(1);
}

// Determine file extension
const isWindows = platform === 'win32';
const ext = isWindows ? '.zip' : '.tar.gz';
const filename = `ricecoder-${target}${ext}`;

// Get version from package.json
const packageJson = require('../package.json');
const version = packageJson.version;

// GitHub release URL
const releaseUrl = `https://github.com/moabualruz/ricecoder/releases/download/v${version}/${filename}`;
const checksumUrl = `${releaseUrl}.sha256`;

// Create bin directory if it doesn't exist
const binDir = path.join(__dirname, '..', 'bin');
if (!fs.existsSync(binDir)) {
  fs.mkdirSync(binDir, { recursive: true });
}

const binaryPath = path.join(binDir, isWindows ? 'ricecoder.exe' : 'ricecoder');

// Download and extract binary
console.log(`Downloading ricecoder ${version} for ${platform}-${arch}...`);
console.log(`Target: ${target}`);
console.log(`URL: ${releaseUrl}`);

downloadAndExtract(releaseUrl, checksumUrl, binaryPath, isWindows)
  .then(() => {
    console.log(`Successfully installed ricecoder to ${binaryPath}`);
    if (!isWindows) {
      fs.chmodSync(binaryPath, 0o755);
    }
    console.log('Installation complete! Run "ricecoder --version" to verify.');
  })
  .catch((err) => {
    // During development, if the release doesn't exist yet, just warn
    if (err.message.includes('HTTP 404')) {
      console.warn(`Warning: ricecoder v${version} binary not yet available for ${platform}-${arch}`);
      console.warn('This is expected during development. The binary will be available after the release is published.');
      console.warn('For development, build from source: cargo build --release');
      process.exit(0);
    }
    console.error(`Failed to install ricecoder: ${err.message}`);
    process.exit(1);
  });

function downloadAndExtract(url, checksumUrl, outputPath, isWindows) {
  return new Promise((resolve, reject) => {
    const tempDir = path.join(__dirname, '..', '.tmp');
    if (!fs.existsSync(tempDir)) {
      fs.mkdirSync(tempDir, { recursive: true });
    }

    const tempFile = path.join(tempDir, path.basename(url));
    const checksumFile = path.join(tempDir, path.basename(checksumUrl));

    // Download checksum first
    downloadFile(checksumUrl, checksumFile)
      .then(() => {
        // Read expected checksum
        const checksumContent = fs.readFileSync(checksumFile, 'utf-8').trim();
        const expectedChecksum = checksumContent.split(/\s+/)[0];
        
        // Download binary
        return downloadFile(url, tempFile)
          .then(() => {
            // Verify checksum
            const fileContent = fs.readFileSync(tempFile);
            const actualChecksum = crypto.createHash('sha256').update(fileContent).digest('hex');
            
            if (actualChecksum !== expectedChecksum) {
              throw new Error(`Checksum verification failed. Expected: ${expectedChecksum}, Got: ${actualChecksum}`);
            }
            
            console.log('Checksum verified successfully');
            
            // Extract binary
            if (isWindows) {
              // Extract zip
              const AdmZip = require('adm-zip');
              const zip = new AdmZip(tempFile);
              zip.extractAllTo(tempDir, true);
              const extractedBinary = path.join(tempDir, 'ricecoder.exe');
              fs.copyFileSync(extractedBinary, outputPath);
            } else {
              // Extract tar.gz
              execSync(`tar -xzf "${tempFile}" -C "${tempDir}"`);
              const extractedBinary = path.join(tempDir, 'ricecoder');
              fs.copyFileSync(extractedBinary, outputPath);
            }

            // Clean up temp files
            fs.rmSync(tempDir, { recursive: true, force: true });
            resolve();
          });
      })
      .catch((err) => {
        // Clean up on error
        try {
          fs.rmSync(tempDir, { recursive: true, force: true });
        } catch (e) {
          // Ignore cleanup errors
        }
        reject(err);
      });
  });
}

function downloadFile(url, outputPath) {
  return new Promise((resolve, reject) => {
    https.get(url, { redirect: 'follow' }, (response) => {
      if (response.statusCode !== 200) {
        reject(new Error(`Failed to download: HTTP ${response.statusCode}`));
        return;
      }

      const file = fs.createWriteStream(outputPath);
      response.pipe(file);

      file.on('finish', () => {
        file.close();
        resolve();
      });

      file.on('error', (err) => {
        fs.unlink(outputPath, () => {});
        reject(err);
      });
    }).on('error', (err) => {
      fs.unlink(outputPath, () => {});
      reject(err);
    });
  });
}
