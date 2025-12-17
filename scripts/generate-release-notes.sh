#!/bin/bash

# RiceCoder Release Notes Generator
# Generates comprehensive release notes with enterprise features

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
TEMPLATE_FILE="$PROJECT_DIR/release-notes-template.md"
OUTPUT_DIR="$PROJECT_DIR/docs/release-notes"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

# Check if git-cliff is installed
check_dependencies() {
    if ! command -v git-cliff >/dev/null 2>&1; then
        log_error "git-cliff is not installed. Please install it first:"
        echo "cargo install git-cliff"
        exit 1
    fi

    if ! command -v jq >/dev/null 2>&1; then
        log_error "jq is not installed. Please install it first."
        exit 1
    fi
}

# Get version information
get_version_info() {
    local version="$1"
    local prev_version

    if [ -z "$version" ]; then
        # Get latest tag
        version=$(git describe --tags --abbrev=0 2>/dev/null || echo "v0.1.0")
    fi

    # Remove 'v' prefix if present
    version="${version#v}"

    # Get previous version
    prev_version=$(git describe --tags --abbrev=0 "${version}^" 2>/dev/null || echo "")

    echo "$version" "$prev_version"
}

# Check for enterprise features
check_enterprise_features() {
    local version="$1"

    # Check if this version has enterprise features
    if git log --oneline --grep="enterprise" "v${version}^..v${version}" | grep -q .; then
        echo "true"
    else
        echo "false"
    fi
}

# Check for breaking changes
check_breaking_changes() {
    local version="$1"

    if git-cliff --latest --strip header | grep -i "breaking\|break" >/dev/null 2>&1; then
        echo "true"
    else
        echo "false"
    fi
}

# Get security advisories
get_security_advisories() {
    local version="$1"

    # This would integrate with a security advisory system
    # For now, return empty array
    echo "[]"
}

# Get performance benchmarks
get_performance_benchmarks() {
    local benchmark_file="$PROJECT_DIR/performance-baselines.json"

    if [ -f "$benchmark_file" ]; then
        jq '.latest' "$benchmark_file" 2>/dev/null || echo "{}"
    else
        echo "{}"
    fi
}

# Get contributors
get_contributors() {
    local version="$1"
    local prev_version="$2"

    if [ -n "$prev_version" ]; then
        git log --format="%an" "v${prev_version}..v${version}" | sort | uniq | paste -sd "," -
    else
        git log --format="%an" | sort | uniq | paste -sd "," -
    fi
}

# Generate release notes
generate_release_notes() {
    local version="$1"
    local prev_version="$2"
    local template_file="$3"
    local output_file="$4"

    log_info "Generating release notes for version $version"

    # Get additional data
    local enterprise_features
    enterprise_features=$(check_enterprise_features "$version")

    local breaking_changes
    breaking_changes=$(check_breaking_changes "$version")

    local security_advisories
    security_advisories=$(get_security_advisories "$version")

    local performance_data
    performance_data=$(get_performance_benchmarks)

    local contributors
    contributors=$(get_contributors "$version" "$prev_version")

    # Extract performance metrics
    local startup_time
    startup_time=$(echo "$performance_data" | jq -r '.startup_time // 2.5')

    local response_time
    response_time=$(echo "$performance_data" | jq -r '.response_time_p95 // 450')

    local memory_usage
    memory_usage=$(echo "$performance_data" | jq -r '.memory_usage_mb // 250')

    local cpu_usage
    cpu_usage=$(echo "$performance_data" | jq -r '.cpu_usage_peak // 75')

    # Generate changelog content
    local changelog_content
    if [ -n "$prev_version" ]; then
        changelog_content=$(git-cliff "v${prev_version}..v${version}" --strip header)
    else
        changelog_content=$(git-cliff --latest --strip header)
    fi

    # Create temporary file with substituted values
    local temp_file
    temp_file=$(mktemp)

    # Copy template and substitute variables
    cp "$template_file" "$temp_file"

    # Replace template variables
    sed -i "s/{{ version }}/$version/g" "$temp_file"
    sed -i "s/{{ timestamp }}/$(date -u +"%Y-%m-%dT%H:%M:%SZ")/g" "$temp_file"

    # Replace enterprise features
    if [ "$enterprise_features" = "true" ]; then
        sed -i '/{% if enterprise_features %}/,/{% endif %}/s/{% if enterprise_features %}//' "$temp_file"
        sed -i '/{% endif %}/s/{% endif %}//' "$temp_file"
    else
        sed -i '/{% if enterprise_features %}/,/{% endif %}/d' "$temp_file"
    fi

    # Replace breaking changes
    if [ "$breaking_changes" = "true" ]; then
        sed -i '/{% if breaking_changes %}/,/{% endif %}/s/{% if breaking_changes %}//' "$temp_file"
        sed -i '/{% endif %}/s/{% endif %}//' "$temp_file"
    else
        sed -i '/{% if breaking_changes %}/,/{% endif %}/d' "$temp_file"
    fi

    # Replace security advisories
    if [ "$security_advisories" != "[]" ]; then
        sed -i '/{% if security_advisories %}/,/{% endif %}/s/{% if security_advisories %}//' "$temp_file"
        sed -i '/{% endif %}/s/{% endif %}//' "$temp_file"
        # Note: Full security advisory replacement would need more complex logic
    else
        sed -i '/{% if security_advisories %}/,/{% endif %}/d' "$temp_file"
    fi

    # Replace performance metrics
    sed -i "s/{{ startup_time }}/$startup_time/g" "$temp_file"
    sed -i "s/{{ response_time }}/$response_time/g" "$temp_file"
    sed -i "s/{{ memory_usage }}/$memory_usage/g" "$temp_file"
    sed -i "s/{{ cpu_usage }}/$cpu_usage/g" "$temp_file"

    # Replace contributors
    sed -i "s/{{ contributors }}/$contributors/g" "$temp_file"

    # Generate the final release notes using git-cliff with custom template
    local final_content
    final_content=$(git-cliff --config cliff.toml --template-file "$temp_file" --strip header)

    # Write to output file
    echo "$final_content" > "$output_file"

    # Cleanup
    rm "$temp_file"

    log_success "Release notes generated: $output_file"
}

# Main function
main() {
    local version=""
    local output_file=""

    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --version)
                version="$2"
                shift 2
                ;;
            --output)
                output_file="$2"
                shift 2
                ;;
            --help)
                echo "Usage: $0 [--version VERSION] [--output FILE]"
                echo ""
                echo "Generate comprehensive release notes for RiceCoder"
                echo ""
                echo "Options:"
                echo "  --version VERSION    Version to generate notes for (default: latest tag)"
                echo "  --output FILE        Output file (default: RELEASE_NOTES_v{VERSION}.md)"
                echo "  --help               Show this help message"
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                exit 1
                ;;
        esac
    done

    check_dependencies

    # Get version info
    local version_info
    version_info=$(get_version_info "$version")
    version=$(echo "$version_info" | cut -d' ' -f1)
    local prev_version
    prev_version=$(echo "$version_info" | cut -d' ' -f2)

    # Set default output file
    if [ -z "$output_file" ]; then
        output_file="$OUTPUT_DIR/RELEASE_NOTES_v${version}.md"
        mkdir -p "$OUTPUT_DIR"
    fi

    log_info "Generating release notes for version $version"
    log_info "Previous version: ${prev_version:-none}"
    log_info "Output file: $output_file"

    generate_release_notes "$version" "$prev_version" "$TEMPLATE_FILE" "$output_file"

    log_success "Release notes generation completed"
}

# Run main function
main "$@"