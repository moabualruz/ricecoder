use std::fs;

use ricegrep::{
    chunking::{ChunkProducer, RepositorySource},
    lexical::{
        Bm25IndexBuilder, Bm25IndexHandle, LexicalError, LexicalIngestPipeline, LexicalSearcher,
    },
};
use tempfile::tempdir;

#[tokio::test]
async fn ingestion_pipeline_produces_queryable_index() {
    let repo_dir = tempdir().expect("create repo dir");
    let src = repo_dir.path().join("src");
    tokio::fs::create_dir_all(&src)
        .await
        .expect("create src dir");
    tokio::fs::write(src.join("lib.rs"), "pub fn pipeline_test() {}\n")
        .await
        .expect("write file");
    tokio::fs::write(src.join("main.rs"), "fn main() { pipeline_test(); }\n")
        .await
        .expect("write file");

    let chunk_producer = ChunkProducer::builder().build();
    let pipeline = LexicalIngestPipeline::new(chunk_producer)
        .with_batch_size(2)
        .with_progress_interval(1);

    let index_dir = tempdir().expect("index dir");
    let builder = Bm25IndexBuilder::create(index_dir.path()).expect("create builder");
    let mut writer = builder.writer(50_000_000).expect("create writer");

    let stats = pipeline
        .ingest_repository(RepositorySource::new(repo_dir.path()), &mut writer)
        .await
        .expect("ingest repository");

    assert!(
        stats.chunks_indexed > 0,
        "expected pipeline to index at least one chunk"
    );
    assert_eq!(stats.errors, 0, "pipeline should not emit chunking errors");

    let handle = writer.commit().expect("commit after ingestion");
    let searcher = LexicalSearcher::new(handle);
    let hits = searcher
        .search("pipeline_test", 5)
        .expect("search after ingestion");
    assert!(hits.iter().any(|hit| hit.file_path.ends_with("lib.rs")));
}

#[test]
fn lexical_handle_reports_structured_error_for_missing_index() {
    let missing_dir = tempdir().expect("missing dir");
    let path = missing_dir.path().join("bogus");
    fs::create_dir_all(&path).expect("create target");
    fs::remove_dir_all(&path).expect("remove to simulate missing files");

    let err = match Bm25IndexHandle::open(&path) {
        Ok(_) => panic!("opening missing index should fail"),
        Err(err) => err,
    };
    match err {
        LexicalError::Io(_) | LexicalError::Tantivy(_) => {}
        other => panic!("expected I/O or Tantivy error, got {other:?}"),
    }
}

#[test]
#[ignore = "Requires running RiceGrep gateway and MCP stdio server"]
fn mcp_stdio_nl_search_smoke() {
    let bin_path = env!("CARGO_BIN_EXE_ricegrep");
    let mut cmd = std::process::Command::new(bin_path);
    cmd.arg("mcp").arg("--no-watch");
    if let Ok(endpoint) = std::env::var("RICEGREP_GATEWAY_ENDPOINT") {
        cmd.arg("--endpoint").arg(endpoint);
    }

    cmd.stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::inherit());

    let mut child = cmd.spawn().expect("start ricegrep mcp");
    let mut stdin = child.stdin.take().expect("capture stdin");

    let initialize = "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2024-11-05\",\"clientInfo\":{\"name\":\"mcp-smoke\",\"version\":\"0.1\"},\"capabilities\":{}}}";
    let initialized = "{\"jsonrpc\":\"2.0\",\"method\":\"initialized\",\"params\":{}}";
    let list_tools = "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}";
    let call_nl = "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"nl_search\",\"arguments\":{\"query\":\"how does auth work\",\"path\":\".\"}}}";
    let payload = format!("{initialize}\n{initialized}\n{list_tools}\n{call_nl}\n");

    use std::io::Write;
    stdin
        .write_all(payload.as_bytes())
        .expect("send MCP requests");
    drop(stdin);

    let output = child.wait_with_output().expect("read MCP output");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"id\":2"), "tools/list response missing");
    assert!(stdout.contains("nl_search"), "nl_search not present in MCP output");
    assert!(stdout.contains("\"id\":3"), "nl_search response missing");
    assert!(
        !stdout.contains("failed to reach RiceGrep gateway"),
        "gateway error in nl_search response"
    );
}
