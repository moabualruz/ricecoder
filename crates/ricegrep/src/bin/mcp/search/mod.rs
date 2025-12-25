use anyhow::{Context, Result};
use glob::Pattern;
use ignore::WalkBuilder;
use regex::Regex;
use ricegrep::api::models::{SearchRequest, SearchResponse, SearchResult};
use ricegrep::chunking::{ChunkMetadata, LanguageDetector, LanguageKind};
use std::path::Path;
use std::time::Instant;
use uuid::Uuid;

pub enum PathMatcher {
    Any,
    Glob(Pattern),
    Substring(String),
}

impl PathMatcher {
    fn matches(&self, path: &Path) -> bool {
        match self {
            PathMatcher::Any => true,
            PathMatcher::Glob(pattern) => pattern.matches_path(path),
            PathMatcher::Substring(filter) => path.to_string_lossy().contains(filter),
        }
    }
}

pub enum ContentMatcher {
    Regex(Regex),
    Literal(String),
}

impl ContentMatcher {
    fn is_match(&self, line: &str) -> bool {
        match self {
            ContentMatcher::Regex(regex) => regex.is_match(line),
            ContentMatcher::Literal(text) => line.contains(text),
        }
    }
}

pub fn build_path_matcher(filter: Option<&str>) -> PathMatcher {
    let Some(filter) = filter else {
        return PathMatcher::Any;
    };
    Pattern::new(filter)
        .map(PathMatcher::Glob)
        .unwrap_or_else(|_| PathMatcher::Substring(filter.to_string()))
}

pub fn build_content_matcher(query: &str) -> ContentMatcher {
    Regex::new(query)
        .map(ContentMatcher::Regex)
        .unwrap_or_else(|_| ContentMatcher::Literal(query.to_string()))
}

fn collect_local_matches(
    root: &Path,
    query: &str,
    path_matcher: &PathMatcher,
    content_matcher: &ContentMatcher,
    limit: Option<usize>,
) -> Result<Vec<SearchResult>> {
    let detector = LanguageDetector::default();
    let mut results = Vec::new();
    let mut next_id: u64 = 1;
    for entry in WalkBuilder::new(root).build() {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if !path_matcher.matches(path) {
            continue;
        }
        if append_matches_for_file(
            &mut results,
            path,
            query,
            limit,
            &detector,
            &mut next_id,
            content_matcher,
        )? {
            return Ok(results);
        }
    }

    Ok(results)
}

fn append_matches_for_file(
    results: &mut Vec<SearchResult>,
    path: &Path,
    query: &str,
    limit: Option<usize>,
    detector: &LanguageDetector,
    next_id: &mut u64,
    content_matcher: &ContentMatcher,
) -> Result<bool> {
    let contents = match std::fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(_) => return Ok(false),
    };
    let language = detector
        .detect(path, &contents)
        .unwrap_or(LanguageKind::PlainText);
    for (index, line) in contents.lines().enumerate() {
        if !content_matcher.is_match(line) {
            continue;
        }
        let line_number = (index + 1) as u32;
        let metadata = ChunkMetadata {
            chunk_id: *next_id,
            repository_id: None,
            file_path: path.to_path_buf(),
            language,
            start_line: line_number,
            end_line: line_number,
            token_count: 0,
            checksum: String::new(),
        };
        results.push(SearchResult {
            chunk_id: *next_id,
            score: 1.0,
            content: line.to_string(),
            metadata,
            highlights: vec![query.to_string()],
        });
        *next_id = next_id.saturating_add(1);
        if limit.map_or(false, |max| results.len() >= max) {
            return Ok(true);
        }
    }
    Ok(false)
}

pub fn local_search_response(
    request: &SearchRequest,
    root: &Path,
    filter: Option<&str>,
) -> Result<SearchResponse> {
    let start = Instant::now();
    let path_matcher = build_path_matcher(filter);
    let content_matcher = build_content_matcher(&request.query);
    let results = collect_local_matches(
        root,
        &request.query,
        &path_matcher,
        &content_matcher,
        request.limit,
    )?;
    let total_found = results.len();
    let elapsed = start.elapsed().as_secs_f64() * 1000.0;
    Ok(SearchResponse {
        results,
        total_found,
        query_time_ms: elapsed,
        request_id: Uuid::new_v4().to_string(),
    })
}

#[cfg(feature = "server")]
pub async fn server_search_request(
    endpoint: &str,
    request: &SearchRequest,
) -> Result<SearchResponse> {
    let url = format!("{}/search", endpoint.trim_end_matches('/'));
    let response = reqwest::Client::new()
        .post(&url)
        .json(request)
        .send()
        .await
        .context("failed to send server search request")?;
    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "server search request failed: {}",
            response.status()
        ));
    }
    let search_response: SearchResponse = response
        .json()
        .await
        .context("failed to parse server search response")?;
    Ok(search_response)
}

#[cfg(not(feature = "server"))]
pub async fn server_search_request(
    _endpoint: &str,
    _request: &SearchRequest,
) -> Result<SearchResponse> {
    Err(anyhow::anyhow!(
        "Server mode is disabled. Rebuild with --features server to enable it."
    ))
}
