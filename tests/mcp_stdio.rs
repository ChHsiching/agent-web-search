//! Integration test for the MCP stdio server.
//!
//! Verifies the ADR-0004 stability disciplines end-to-end against the real
//! binary:
//! - the server responds to `initialize` and `tools/list`.
//! - stdout contains ONLY valid JSON-RPC frames (no stray log pollution).
//! - `tools/call` returns a JSON-shaped response (results array or error
//!   object) without crashing. This call hits the network, so we assert only
//!   on response shape, not on whether live results came back (instances may
//!   be rate-limited or offline).

use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Command, Stdio};
use std::time::Duration;

/// Spawn the built binary as a stdio MCP server.
fn spawn_server() -> std::process::Child {
    let bin = env!("CARGO_BIN_EXE_agent-web-search");
    Command::new(bin)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn server")
}

/// Read one JSON-RPC response line from stdout.
fn read_response(reader: &mut BufReader<std::process::ChildStdout>) -> serde_json::Value {
    let mut line = String::new();
    reader
        .read_line(&mut line)
        .expect("expected a JSON-RPC line on stdout");
    while line.trim().is_empty() {
        line.clear();
        reader.read_line(&mut line).expect("expected another line");
    }
    serde_json::from_str(&line).unwrap_or_else(|e| {
        panic!(
            "stdout line is not valid JSON-RPC (stdout pollution?):\n  line: {line:?}\n  error: {e}"
        )
    })
}

const INIT: &str = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"0.1"}}}"#;
const INITIALIZED: &str = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;
const TOOLS_LIST: &str = r#"{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}"#;

#[test]
fn initialize_handshake_responds() {
    let mut child = spawn_server();
    let mut stdin = child.stdin.take().expect("no stdin");
    let mut stdout = BufReader::new(child.stdout.take().expect("no stdout"));

    writeln!(stdin, "{INIT}").unwrap();
    stdin.flush().unwrap();

    let resp = read_response(&mut stdout);
    assert_eq!(resp["jsonrpc"], "2.0");
    assert_eq!(resp["id"], 1);
    assert!(
        resp["result"]["serverInfo"]["name"]
            .as_str()
            .unwrap()
            .contains("agent-web-search"),
        "server name in initialize response: {resp}"
    );
    assert!(
        resp["result"]["capabilities"]["tools"].is_object(),
        "tools capability advertised: {resp}"
    );

    writeln!(stdin, "{INITIALIZED}").unwrap();
    stdin.flush().unwrap();

    child.kill().ok();
    child.wait().ok();
}

#[test]
fn tools_list_advertises_web_search_prime() {
    let mut child = spawn_server();
    let mut stdin = child.stdin.take().expect("no stdin");
    let mut stdout = BufReader::new(child.stdout.take().expect("no stdout"));

    writeln!(stdin, "{INIT}").unwrap();
    stdin.flush().unwrap();
    let _ = read_response(&mut stdout);

    writeln!(stdin, "{INITIALIZED}").unwrap();
    stdin.flush().unwrap();

    writeln!(stdin, "{TOOLS_LIST}").unwrap();
    stdin.flush().unwrap();

    let resp = read_response(&mut stdout);
    let tools = resp["result"]["tools"]
        .as_array()
        .expect("tools is an array");
    assert_eq!(tools.len(), 1, "exactly one tool advertised");

    let tool = &tools[0];
    assert_eq!(tool["name"], "web_search_prime");

    let props = &tool["inputSchema"]["properties"];
    assert!(props["search_query"]["type"] == "string");
    for opt in [
        "search_domain_filter",
        "search_recency_filter",
        "content_size",
        "location",
    ] {
        assert!(
            props[opt]["type"] == "string",
            "optional param {opt} present as string"
        );
    }
    let required = tool["inputSchema"]["required"]
        .as_array()
        .expect("required is an array");
    assert_eq!(required.len(), 1);
    assert_eq!(required[0], "search_query");

    child.kill().ok();
    child.wait().ok();
}

#[test]
fn stdout_is_clean_json_rpc_only() {
    // The ADR-0004 discipline: nothing but JSON-RPC on stdout. We run
    // initialize + tools/list (no tools/call, so no network dependency) and
    // assert every stdout line parses as JSON.
    let mut child = spawn_server();
    let mut stdin = child.stdin.take().expect("no stdin");
    let mut stdout = BufReader::new(child.stdout.take().expect("no stdout"));

    for line in &[INIT, INITIALIZED, TOOLS_LIST] {
        writeln!(stdin, "{line}").unwrap();
    }
    stdin.flush().unwrap();

    std::thread::sleep(Duration::from_millis(150));
    child.kill().ok();
    let mut raw = String::new();
    stdout.read_to_string(&mut raw).ok();
    child.wait().ok();

    let mut parsed = 0;
    for line in raw.lines() {
        if line.trim().is_empty() {
            continue;
        }
        serde_json::from_str::<serde_json::Value>(line).unwrap_or_else(|e| {
            panic!("stdout pollution — line not JSON-RPC:\n  {line}\n  {e}")
        });
        parsed += 1;
    }
    // initialize + tools/list produce 2 responses.
    assert!(
        parsed >= 2,
        "expected at least 2 JSON-RPC responses on stdout, got {parsed}; raw:\n{raw}"
    );
}

#[test]
fn tools_call_returns_json_shaped_response_without_crashing() {
    // tools/call now runs the real search pipeline (network). We only assert
    // the server doesn't crash and returns a well-formed JSON-RPC response
    // whose content is valid JSON — either a results array or an error object.
    // Live results aren't guaranteed (instances may rate-limit or be offline).
    let mut child = spawn_server();
    let mut stdin = child.stdin.take().expect("no stdin");
    let mut stdout = BufReader::new(child.stdout.take().expect("no stdout"));

    writeln!(stdin, "{INIT}").unwrap();
    stdin.flush().unwrap();
    let _ = read_response(&mut stdout);
    writeln!(stdin, "{INITIALIZED}").unwrap();
    stdin.flush().unwrap();

    let call = r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"web_search_prime","arguments":{"search_query":"rust programming language","location":"us"}}}"#;
    writeln!(stdin, "{call}").unwrap();
    stdin.flush().unwrap();

    let resp = read_response(&mut stdout);
    assert_eq!(resp["id"], 3);
    assert!(
        resp.get("error").is_none(),
        "no JSON-RPC level error on tools/call: {resp}"
    );

    // The tool result content should be valid JSON (results array or error obj).
    let content = resp["result"]["content"][0]["text"]
        .as_str()
        .expect("tool result has text content");
    let parsed: serde_json::Value =
        serde_json::from_str(content).expect("tool content is valid JSON");
    assert!(
        parsed.is_array() || parsed.get("error").is_some(),
        "content is a results array or an error object: {parsed}"
    );

    child.kill().ok();
    child.wait().ok();
}
