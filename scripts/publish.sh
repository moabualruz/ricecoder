#!/bin/bash

# RiceCoder Automated Publishing Script
# Analyzes dependencies and publishes crates in correct order

set -e

# Configuration
DRY_RUN=false
VERBOSE=false
WAIT_SECONDS=10

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        --wait)
            WAIT_SECONDS="$2"
            shift 2
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Logging functions
log_info() {
    echo -e "${CYAN}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

# Get crate dependencies
get_crate_dependencies() {
    local crate_path=$1
    local cargo_toml="$crate_path/Cargo.toml"
    
    if [[ ! -f "$cargo_toml" ]]; then
        echo ""
        return
    fi
    
    # Extract ricecoder dependencies
    grep -oP 'ricecoder-[\w-]+' "$cargo_toml" | sort -u | tr '\n' ' '
}

# Get crate info
get_crate_info() {
    local crate_path=$1
    local cargo_toml="$crate_path/Cargo.toml"
    
    # Extract name
    local name=$(grep -oP 'name\s*=\s*"\K[^"]+' "$cargo_toml" | head -1)
    
    # Extract version
    local version=$(grep -oP 'version\s*=\s*"\K[^"]+' "$cargo_toml" | head -1)
    
    # Check if publishable
    local publishable=true
    if grep -q 'publish\s*=\s*false' "$cargo_toml"; then
        publishable=false
    fi
    
    echo "$name|$version|$publishable|$(get_crate_dependencies "$crate_path")"
}

# Resolve publish order using topological sort
resolve_publish_order() {
    local -a crates=("$@")
    local -a ordered=()
    local -a processed=()
    local -a processing=()
    
    resolve_crate() {
        local crate_name=$1
        
        # Check if already processed
        if [[ " ${processed[@]} " =~ " ${crate_name} " ]]; then
            return
        fi
        
        # Check for circular dependency
        if [[ " ${processing[@]} " =~ " ${crate_name} " ]]; then
            log_warning "Circular dependency detected: $crate_name"
            return
        fi
        
        processing+=("$crate_name")
        
        # Find crate info
        local crate_info=""
        for info in "${crates[@]}"; do
            if [[ "$info" == "$crate_name|"* ]]; then
                crate_info="$info"
                break
            fi
        done
        
        if [[ -z "$crate_info" ]]; then
            processing=("${processing[@]/$crate_name}")
            return
        fi
        
        # Parse crate info
        IFS='|' read -r name version publishable deps <<< "$crate_info"
        
        # Resolve dependencies first
        for dep in $deps; do
            resolve_crate "$dep"
        done
        
        # Add to ordered list if publishable
        if [[ "$publishable" == "true" ]] && ! [[ " ${processed[@]} " =~ " ${crate_name} " ]]; then
            ordered+=("$crate_info")
            processed+=("$crate_name")
        fi
        
        # Remove from processing
        processing=("${processing[@]/$crate_name}")
    }
    
    # Process all crates
    for info in "${crates[@]}"; do
        IFS='|' read -r name version publishable deps <<< "$info"
        if [[ "$publishable" == "true" ]]; then
            resolve_crate "$name"
        fi
    done
    
    # Output ordered crates
    printf '%s\n' "${ordered[@]}"
}

# Test crate
test_crate() {
    local crate_path=$1
    local crate_name=$2
    
    log_info "Testing $crate_name..."
    
    pushd "$crate_path" > /dev/null
    
    # Run tests
    if ! cargo test --release > /dev/null 2>&1; then
        log_error "Tests failed for $crate_name"
        popd > /dev/null
        return 1
    fi
    
    # Check clippy
    if ! cargo clippy --release > /dev/null 2>&1; then
        log_warning "Clippy warnings for $crate_name"
    fi
    
    popd > /dev/null
    return 0
}

# Publish crate
publish_crate() {
    local crate_path=$1
    local crate_name=$2
    local dry_run=$3
    
    log_info "Publishing $crate_name..."
    
    pushd "$crate_path" > /dev/null
    
    if [[ "$dry_run" == "true" ]]; then
        log_warning "DRY RUN: cargo publish --dry-run"
        cargo publish --dry-run
    else
        log_info "Publishing to crates.io..."
        cargo publish
    fi
    
    if [[ $? -ne 0 ]]; then
        log_error "Failed to publish $crate_name"
        popd > /dev/null
        return 1
    fi
    
    log_success "Successfully published $crate_name"
    popd > /dev/null
    return 0
}

# Main function
main() {
    log_info "RiceCoder Automated Publishing Script"
    log_info "======================================"
    
    # Find all crates
    local crates_dir="crates"
    if [[ ! -d "$crates_dir" ]]; then
        log_error "crates directory not found"
        exit 1
    fi
    
    log_info "Scanning crates..."
    local -a crates=()
    for crate_path in "$crates_dir"/*; do
        if [[ -d "$crate_path" ]]; then
            local crate_info=$(get_crate_info "$crate_path")
            if [[ -n "$crate_info" ]]; then
                crates+=("$crate_info")
                IFS='|' read -r name version publishable deps <<< "$crate_info"
                log_info "Found: $name v$version"
            fi
        fi
    done
    
    # Resolve publish order
    log_info "Resolving publish order..."
    local -a ordered=($(resolve_publish_order "${crates[@]}"))
    
    log_info "Publish order:"
    for info in "${ordered[@]}"; do
        IFS='|' read -r name version publishable deps <<< "$info"
        log_info "  $name v$version"
    done
    
    # Confirm before publishing
    if [[ "$DRY_RUN" != "true" ]]; then
        read -p "Proceed with publishing? (yes/no): " confirm
        if [[ "$confirm" != "yes" ]]; then
            log_warning "Publishing cancelled"
            exit 0
        fi
    fi
    
    # Publish crates
    local -a failed=()
    local -a published=()
    
    for info in "${ordered[@]}"; do
        IFS='|' read -r name version publishable deps <<< "$info"
        local crate_path="$crates_dir/$name"
        
        log_info "======================================"
        
        # Test crate
        if ! test_crate "$crate_path" "$name"; then
            log_warning "Skipping $name due to test failures"
            failed+=("$name")
            continue
        fi
        
        # Publish crate
        if publish_crate "$crate_path" "$name" "$DRY_RUN"; then
            published+=("$name")
            
            # Wait for crates.io to index
            if [[ "$DRY_RUN" != "true" ]]; then
                log_info "Waiting $WAIT_SECONDS seconds for crates.io indexing..."
                sleep "$WAIT_SECONDS"
            fi
        else
            failed+=("$name")
        fi
    done
    
    # Summary
    log_info "======================================"
    log_info "Publishing Summary"
    log_info "======================================"
    log_success "Published: ${#published[@]}"
    for crate in "${published[@]}"; do
        log_success "  ✓ $crate"
    done
    
    if [[ ${#failed[@]} -gt 0 ]]; then
        log_error "Failed: ${#failed[@]}"
        for crate in "${failed[@]}"; do
            log_error "  ✗ $crate"
        done
        exit 1
    fi
    
    log_success "All crates published successfully!"
    exit 0
}

# Run main
main
