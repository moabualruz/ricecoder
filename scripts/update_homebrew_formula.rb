#!/usr/bin/env ruby
# Script to update Homebrew formula with latest release information

require 'net/http'
require 'json'
require 'digest'

# Configuration
REPO = 'moabualruz/ricecoder'
FORMULA_PATH = 'homebrew/ricecoder.rb'

# Get latest release info from GitHub API
def get_latest_release
  uri = URI("https://api.github.com/repos/#{REPO}/releases/latest")
  response = Net::HTTP.get(uri)
  JSON.parse(response)
end

# Download file and calculate SHA256
def calculate_sha256(url)
  uri = URI(url)
  response = Net::HTTP.get(uri)
  Digest::SHA256.hexdigest(response)
end

# Update Homebrew formula
def update_formula(version, assets)
  formula_content = File.read(FORMULA_PATH)

  # Update version
  formula_content.gsub!(/version "\d+\.\d+\.\d+"/, "version \"#{version}\"")

  # Update URLs and checksums for each platform
  assets.each do |asset|
    name = asset['name']
    url = asset['browser_download_url']

    if name.include?('macos-x86_64.tar.gz')
      sha256 = calculate_sha256(url)
      formula_content.gsub!(/(url "https:\/\/github\.com\/moabualruz\/ricecoder\/releases\/download\/v\d+\.\d+\.\d+\/ricecoder-\d+\.\d+\.\d+-macos-x86_64\.tar\.gz")/, "url \"#{url}\"")
      formula_content.gsub!(/(sha256 ")(.*?)(" # \{:x86_64_macos\})/, "\\1#{sha256}\\3")
    elsif name.include?('macos-arm64.tar.gz')
      sha256 = calculate_sha256(url)
      formula_content.gsub!(/(url "https:\/\/github\.com\/moabualruz\/ricecoder\/releases\/download\/v\d+\.\d+\.\d+\/ricecoder-\d+\.\d+\.\d+-macos-arm64\.tar\.gz")/, "url \"#{url}\"")
      formula_content.gsub!(/(sha256 ")(.*?)(" # \{:arm64_macos\})/, "\\1#{sha256}\\3")
    elsif name.include?('linux-x86_64.tar.gz')
      sha256 = calculate_sha256(url)
      formula_content.gsub!(/(url "https:\/\/github\.com\/moabualruz\/ricecoder\/releases\/download\/v\d+\.\d+\.\d+\/ricecoder-\d+\.\d+\.\d+-linux-x86_64\.tar\.gz")/, "url \"#{url}\"")
      formula_content.gsub!(/(sha256 ")(.*?)(" # \{:x86_64_linux\})/, "\\1#{sha256}\\3")
    elsif name.include?('linux-arm64.tar.gz')
      sha256 = calculate_sha256(url)
      formula_content.gsub!(/(url "https:\/\/github\.com\/moabualruz\/ricecoder\/releases\/download\/v\d+\.\d+\.\d+\/ricecoder-\d+\.\d+\.\d+-linux-arm64\.tar\.gz")/, "url \"#{url}\"")
      formula_content.gsub!(/(sha256 ")(.*?)(" # \{:arm64_linux\})/, "\\1#{sha256}\\3")
    end
  end

  File.write(FORMULA_PATH, formula_content)
  puts "Updated Homebrew formula to version #{version}"
end

# Main execution
begin
  release = get_latest_release
  version = release['tag_name'].gsub('v', '')
  assets = release['assets']

  update_formula(version, assets)
rescue => e
  puts "Error updating Homebrew formula: #{e.message}"
  exit 1
end