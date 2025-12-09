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
    local crate_name=$2
    local cargo_toml="$crate_path/Cargo.toml"
    
    if [[ ! -f "$cargo_toml" ]]; then
        echo ""
        return
    fi
    
    # Extract ricecoder dependencies, excluding the crate's own name
    grep -oP 'ricecoder-[\w-]+' "$cargo_toml" 2>/dev/null | sort -u | grep -v "^$crate_name$" | tr '\n' ' ' || echo ""
}

# Get crate info
get_crate_info() {
    local crate_path=$1
    local cargo_toml="$crate_path/Cargo.toml"
    
    # Extract name
    local name=$(grep -oP 'name\s*=\s*"\K[^"]+' "$cargo_toml" 2>/dev/null | head -1)
    
    # Extract version
    local version=$(grep -oP 'version\s*=\s*"\K[^"]+' "$cargo_toml" 2>/dev/null | head -1)
    
    # Check if publishable
    local publishable=true
    if grep -q 'publish\s*=\s*false' "$cargo_toml"; then
        publishable=false
    fi
    
    echo "$name|$version|$publishable|$(get_crate_dependencies "$crate_path" "$name")"
}

# Get all published ricecoder crates with versions in one go
get_published_ricecoder_crates() {
    log_info "Checking published crates on crates.io..."
    
    # Search for all ricecoder crates in one go (increased limit to 200)
    local output=$(cargo search ricecoder --limit 200 2>/dev/null)
    
    # Parse output: each line is "crate-name = "version" # description"
    # Output format: "crate-name|version"
    while IFS= read -r line; do
        if [[ $line =~ ^(ricecoder-[a-z0-9-]+)[[:space:]]*=[[:space:]]*\"([^\"]+)\" ]]; then
            echo "${BASH_REMATCH[1]}|${BASH_REMATCH[2]}"
        fi
    done <<< "$output"
}

# Check if crate is published
is_crate_published() {
    local crate_name=$1
    shift
    local -a published_data=("$@")
    
    for entry in "${published_data[@]}"; do
        IFS='|' read -r pub_name pub_version <<< "$entry"
        if [[ "$pub_name" == "$crate_name" ]]; then
            return 0  # Found
        fi
    done
    
    return 1  # Not found
}

# Check if should publish (new crate or newer version)
should_publish_crate() {
    local crate_name=$1
    local local_version=$2
    shift 2
    local -a published_data=("$@")
    
    # Find the crate in published data
    for entry in "${published_data[@]}"; do
        IFS='|' read -r pub_name pub_version <<< "$entry"
        if [[ "$pub_name" == "$crate_name" ]]; then
            # Simple string comparison first
            if [[ "$local_version" == "$pub_version" ]]; then
                return 1  # Versions are equal, don't publish
            fi
            
            # Extract numeric parts for comparison
            local -a local_parts=()
            local -a pub_parts=()
            
            # Extract numeric parts from local version
            while IFS='.' read -ra parts; do
                for part in "${parts[@]}"; do
                    if [[ $part =~ ^([0-9]+) ]]; then
                        local_parts+=("${BASH_REMATCH[1]}")
                    fi
                done
            done <<< "$local_version"
            
            # Extract numeric parts from published version
            while IFS='.' read -ra parts; do
                for part in "${parts[@]}"; do
                    if [[ $part =~ ^([0-9]+) ]]; then
                        pub_parts+=("${BASH_REMATCH[1]}")
                    fi
                done
            done <<< "$pub_version"
            
            # Compare numeric parts
            local max_len=${#local_parts[@]}
            [[ ${#pub_parts[@]} -gt $max_len ]] && max_len=${#pub_parts[@]}
            
            for ((i=0; i<max_len; i++)); do
                local local_part=${local_parts[$i]:-0}
                local pub_part=${pub_parts[$i]:-0}
                
                if [[ $local_part -gt $pub_part ]]; then
                    return 0  # Local version is newer, should publish
                elif [[ $local_part -lt $pub_part ]]; then
                    return 1  # Published version is newer, don't publish
                fi
            done
            
            # Versions are equal
            return 1  # Don't publish
        fi
    done
    
    # Not found in published list, should publish
    return 0
}

# Parse rate limit time from error message
parse_rate_limit_time() {
    local error_msg=$1
    
    # Extract timestamp like "Tue, 09 Dec 2025 16:27:40 GMT"
    if echo "$error_msg" | grep -oP 'after\s+\K[A-Za-z]+,\s+\d+\s+[A-Za-z]+\s+\d+\s+\d+:\d+:\d+\s+GMT' > /tmp/rate_limit_time.txt 2>/dev/null; then
        cat /tmp/rate_limit_time.txt
        return 0
    fi
    return 1
}

# Wait for rate limit to expire
wait_for_rate_limit() {
    local gmt_time_str=$1
    
    # Convert GMT time to local time using date command
    local gmt_epoch=$(date -d "$gmt_time_str" +%s 2>/dev/null)
    if [[ -z "$gmt_epoch" ]]; then
        log_warning "Could not parse rate limit time: $gmt_time_str"
        return
    fi
    
    local now_epoch=$(date +%s)
    local wait_seconds=$((gmt_epoch - now_epoch))
    
    if [[ $wait_seconds -le 0 ]]; then
        return
    fi
    
    log_warning "Rate limit detected. Retry available at: $gmt_time_str (GMT)"
    log_info "Waiting $wait_seconds seconds before retrying..."
    
    # Show countdown every 30 seconds
    local elapsed=0
    while [[ $elapsed -lt $wait_seconds ]]; do
        local remaining=$((wait_seconds - elapsed))
        if [[ $remaining -le 0 ]]; then break; fi
        
        local sleep_time=$((remaining < 30 ? remaining : 30))
        sleep "$sleep_time"
        elapsed=$((elapsed + sleep_time))
        
        if [[ $remaining -gt 30 ]]; then
            local remaining_min=$(((remaining + 59) / 60))
            log_info "Still waiting... $remaining_min minutes remaining"
        fi
    done
    
    log_success "Rate limit period expired, resuming publishing..."
}

# Publish crate
publish_crate() {
    local crate_path=$1
    local crate_name=$2
    local local_version=$3
    local dry_run=$4
    shift 4
    local -a published_data=("$@")
    
    # Check if we should publish (new crate or newer version)
    if ! should_publish_crate "$crate_name" "$local_version" "${published_data[@]}"; then
        log_warning "Crate already published with same or newer version on crates.io - skipping"
        return 0  # Success (already published, no need to publish)
    fi
    
    log_info "Publishing $crate_name..."
    
    pushd "$crate_path" > /dev/null
    
    if [[ "$dry_run" == "true" ]]; then
        log_warning "DRY RUN: Skipping actual publish (would run: cargo publish)"
        popd > /dev/null
        return 0  # Success
    fi
    
    log_info "Running: cargo publish"
    local output=$(cargo publish 2>&1)
    local exit_code=$?
    
    if [[ $exit_code -ne 0 ]]; then
        log_error "ERROR: cargo publish failed with exit code $exit_code"
        log_error "Output: $output"
        
        # Check if it's a rate limit error
        if echo "$output" | grep -q "429 Too Many Requests\|published too many new crates"; then
            local rate_limit_time=$(parse_rate_limit_time "$output")
            if [[ -n "$rate_limit_time" ]]; then
                log_warning "Reason: Rate limit - waiting and retrying once"
                wait_for_rate_limit "$rate_limit_time"
                
                # Retry once after waiting
                log_info "Retrying after rate limit wait..."
                output=$(cargo publish 2>&1)
                exit_code=$?
                
                if [[ $exit_code -eq 0 ]]; then
                    log_success "SUCCESS: $crate_name published to crates.io (after rate limit wait)"
                    popd > /dev/null
                    return 0  # Success
                fi
                
                # Still failed after retry - check if it's another rate limit or different error
                log_error "ERROR: cargo publish still failed after rate limit wait"
                log_error "Output: $output"
                
                # Check if it's another rate limit
                if echo "$output" | grep -q "429 Too Many Requests\|published too many new crates"; then
                    log_warning "Reason: Rate limit again - will retry later"
                    popd > /dev/null
                    return 2  # Retry (add to retry list)
                fi
                
                # Check if it's a dependency error
                if echo "$output" | grep -q "no matching package named\|not found in registry"; then
                    log_warning "Reason: Dependency not yet published - will retry individually"
                    popd > /dev/null
                    return 2  # Retry
                fi
                
                # Other error - add to retry list
                log_warning "Reason: Unknown error - will retry"
                popd > /dev/null
                return 2  # Retry
            fi
        fi
        
        # Check if it's a dependency error
        if echo "$output" | grep -q "no matching package named\|not found in registry"; then
            log_warning "Reason: Dependency not yet published - will retry individually"
            popd > /dev/null
            return 2  # Retry
        fi
        
        # Check for already exists
        if echo "$output" | grep -q "already exists"; then
            log_warning "Reason: Crate version already published"
            popd > /dev/null
            return 0  # Success
        fi
        
        # Other errors - add to retry list
        log_warning "Reason: Unknown error - will retry"
        popd > /dev/null
        return 2  # Retry
    fi
    
    log_success "SUCCESS: $crate_name published to crates.io"
    popd > /dev/null
    return 0  # Success
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
    
    # Filter publishable crates
    log_info "Filtering publishable crates..."
    local -a publishable=()
    for info in "${crates[@]}"; do
        IFS='|' read -r name version publishable_flag deps <<< "$info"
        if [[ "$publishable_flag" == "true" ]]; then
            publishable+=("$info")
        fi
    done
    
    # Sort by number of internal dependencies (ascending)
    # Crates with 0 dependencies first, then 1, then 2, etc.
    log_info "Sorting crates by internal dependency count..."
    
    # Create array with dependency counts
    local -a crates_with_counts=()
    for info in "${publishable[@]}"; do
        IFS='|' read -r name version _ deps <<< "$info"
        local dep_count=0
        if [[ -n "$deps" ]]; then
            dep_count=$(echo "$deps" | wc -w)
        fi
        crates_with_counts+=("$dep_count|$info")
    done
    
    # Sort by dependency count (first field)
    local -a sorted
    mapfile -t sorted < <(printf '%s\n' "${crates_with_counts[@]}" | sort -t'|' -n)
    
    # Extract just the crate info (remove count prefix)
    local -a ordered=()
    for item in "${sorted[@]}"; do
        ordered+=("${item#*|}")
    done
    
    log_info "Crates to publish: ${#ordered[@]}"
    log_info "Publish order (by dependency count):"
    for info in "${ordered[@]}"; do
        IFS='|' read -r name version _ deps <<< "$info"
        local dep_count=0
        if [[ -n "$deps" ]]; then
            dep_count=$(echo "$deps" | wc -w)
            log_info "  $name v$version [$dep_count deps: $deps]"
        else
            log_info "  $name v$version [0 deps]"
        fi
    done
    
    # Get all published ricecoder crates with versions in one go
    local -a published_crates=()
    if [[ "$DRY_RUN" != "true" ]]; then
        mapfile -t published_crates < <(get_published_ricecoder_crates)
        log_info "Found ${#published_crates[@]} published ricecoder crates"
    fi
    
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
    local -a retry_list=()
    
    # First pass: try to publish all crates (only if all dependencies are published)
    log_info "First pass: attempting to publish all crates..."
    for info in "${ordered[@]}"; do
        IFS='|' read -r name version publishable deps <<< "$info"
        local crate_path="$crates_dir/$name"
        
        log_info "======================================"
        
        # On dry-run, skip dependency checks - just show what would be published
        if [[ "$DRY_RUN" != "true" ]]; then
            # Check if all dependencies are published BEFORE sending request
            local all_deps_published=true
            if [[ -n "$deps" ]]; then
                for dep in $deps; do
                    if ! is_crate_published "$dep" "${published_crates[@]}"; then
                        all_deps_published=false
                        log_warning "⏳ $name - skipping, waiting for dependency: $dep"
                        retry_list+=("$info")
                        break
                    fi
                done
            fi
            
            if [[ "$all_deps_published" != "true" ]]; then
                continue
            fi
        fi
        
        # Publish crate
        publish_crate "$crate_path" "$name" "$version" "$DRY_RUN" "${published_crates[@]}"
        local result=$?
        
        if [[ "$DRY_RUN" == "true" ]]; then
            # On dry-run, just mark as published
            log_success "✓ $name would be published"
            published+=("$name")
        elif [[ $result -eq 2 ]]; then
            log_warning "⟳ $name - dependency error, will retry"
            retry_list+=("$info")
        elif [[ $result -eq 0 ]]; then
            log_success "✓ $name - published successfully"
            published+=("$name")
            
            # Only wait if we actually published (not skipped)
            # Check if this was a skip or actual publish by checking if it was in published crates
            local was_published=false
            for entry in "${published_crates[@]}"; do
                IFS='|' read -r pub_name pub_version <<< "$entry"
                if [[ "$pub_name" == "$name" ]]; then
                    was_published=true
                    break
                fi
            done
            
            if [[ "$was_published" == "false" ]]; then
                # Was newly published, wait for crates.io to index
                log_info "Waiting $WAIT_SECONDS seconds for crates.io indexing..."
                sleep "$WAIT_SECONDS"
            fi
        else
            log_error "✗ $name - publish failed"
            failed+=("$name")
        fi
    done
    
    # Second pass: retry failed crates - only send request if ALL dependencies are published
    if [[ ${#retry_list[@]} -gt 0 ]]; then
        log_info "======================================"
        log_warning "Second pass: retrying crates when dependencies are published..."
        log_info "======================================"
        
        local max_retry_attempts=10
        local retry_attempt=0
        
        while [[ ${#retry_list[@]} -gt 0 ]] && [[ $retry_attempt -lt $max_retry_attempts ]]; do
            ((retry_attempt++))
            local -a new_retry_list=()
            
            for info in "${retry_list[@]}"; do
                IFS='|' read -r name version publishable deps <<< "$info"
                local crate_path="$crates_dir/$name"
                
                # Check if all dependencies are published BEFORE sending request
                local all_deps_published=true
                if [[ -n "$deps" ]]; then
                    for dep in $deps; do
                        if ! is_crate_published "$dep" "${published_crates[@]}"; then
                            all_deps_published=false
                            log_info "⏳ $name - waiting for dependency: $dep"
                            break
                        fi
                    done
                fi
                
                if [[ "$all_deps_published" != "true" ]]; then
                    # Keep in retry list, don't send request yet
                    new_retry_list+=("$info")
                    continue
                fi
                
                # All dependencies published, now try to publish
                log_warning "⟳ Retrying: $name"
                publish_crate "$crate_path" "$name" "$version" "$DRY_RUN" "${published_crates[@]}"
                local result=$?
                
                if [[ $result -eq 0 ]]; then
                    log_success "✓ $name - published on retry"
                    published+=("$name")
                    
                    # Wait for crates.io to index
                    log_info "Waiting $WAIT_SECONDS seconds for crates.io indexing..."
                    sleep "$WAIT_SECONDS"
                elif [[ $result -eq 2 ]]; then
                    # Still has dependency issues, keep in retry list
                    log_warning "⏳ $name - still has issues, will retry"
                    new_retry_list+=("$info")
                else
                    # Failed permanently
                    log_error "✗ $name - failed on retry"
                    failed+=("$name")
                fi
            done
            
            retry_list=("${new_retry_list[@]}")
            
            if [[ ${#retry_list[@]} -gt 0 ]]; then
                log_info "Waiting 30 seconds before next retry attempt ($retry_attempt/$max_retry_attempts)..."
                sleep 30
            fi
        done
        
        # Any remaining crates in retry list are failures
        for info in "${retry_list[@]}"; do
            IFS='|' read -r name version publishable deps <<< "$info"
            log_error "✗ $name - failed after $max_retry_attempts retry attempts"
            failed+=("$name")
        done
    fi
    
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
