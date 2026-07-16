//! Integration test for the MCP stdio server shell (ticket #2).
//!
//! Verifies the ADR-0004 stability disciplines end-to-end against the real
//! binary:
//! - the server responds to `initialize`, `tools/list`, and `tools/call`.
//! - stdout contains ONLY valid JSON-RPC frames (no stray log pollution).
//! - no network call blocks the handshake (it would show up as a stall).

use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Command, Stdio};

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

/// Read one JSON-RPC response line from stdout. Each MCP message over stdio
/// is a single line of JSON (newline-delimited).
fn read_response(reader: &mut BufReader<std::process::ChildStdout>) -> serde_json::Value {
    let mut line = String::new();
    reader
        .read_line(&mut line)
        .expect("expected a JSON-RPC line on stdout");
    // Skip any blank lines.
    while line.trim().is_empty() {
        line.clear();
        reader.read_line(&mut line).expect("expected another line");
    }
    serde_json::from_str(&line).unwrap_or_else(|e| {
        panic!("stdout line is not valid JSON-RPC (stdout pollution?):\n  line: {line:?}\n  error: {e}")
    })
}

#[test]
fn initialize_handshake_responds() {
    let mut child = spawn_server();
    let mut stdin = child.stdin.take().expect("no stdin");
    let mut stdout = BufReader::new(child.stdout.take().expect("no stdout"));

    // Send initialize — this is the handshake that must never block on network.
    let init = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"0.1"}}}"#;
    writeln!(stdin, "{init}").unwrap();
    stdin.flush().unwrap();

    let resp = read_response(&mut stdout);
    assert_eq!(resp["jsonrpc"], "2.0");
    assert_eq!(resp["id"], 1);
    assert!(
        resp["result"]["serverInfo"]["name"].as_str().unwrap()
            .contains("agent-web-search"),
        "server name in initialize response: {resp}"
    );
    assert!(
        resp["result"]["capabilities"]["tools"].is_object(),
        "tools capability advertised: {resp}"
    );

    // Send the initialized notification (no response expected), then list tools.
    let initialized =
        r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;
    writeln!(stdin, "{initialized}").unwrap();
    stdin.flush().unwrap();

    child.kill().ok();
    child.wait().ok();
}

#[test]
fn tools_list_advertises_web_search_prime() {
    let mut child = spawn_server();
    let mut stdin = child.stdin.take().expect("no stdin");
    let mut stdout = BufReader::new(child.stdout.take().expect("no stdout"));

    // initialize first
    let init = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"0.1"}}}"#;
    writeln!(stdin, "{init}").unwrap();
    stdin.flush().unwrap();
    let _ = read_response(&mut stdout);

    writeln!(
        stdin,
        r#"{{"jsonrpc":"2.0","method":"notifications/initialized"}}"#
    )
    .unwrap();
    stdin.flush().unwrap();

    // tools/list
    let list = r#"{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}"#;
    writeln!(stdin, "{list}").unwrap();
    stdin.flush().unwrap();

    let resp = read_response(&mut stdout);
    let tools = resp["result"]["tools"]
        .as_array()
        .expect("tools is an array");
    assert_eq!(tools.len(), 1, "exactly one tool advertised");

    let tool = &tools[0];
    assert_eq!(tool["name"], "web_search_prime");

    // The five-parameter schema must match the target tool.
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
    assert_eq!(required[0], "search_query", "only search_query is required");

    child.kill().ok();
    child.wait().ok();
}

#[test]
fn tools_call_returns_stub_without_error() {
    let mut child = spawn_server();
    let mut stdin = child.stdin.take().expect("no stdin");
    let mut stdout = BufReader::new(child.stdout.take().expect("no stdout"));

    let init = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"0.1"}}}"#;
    writeln!(stdin, "{init}").unwrap();
    stdin.flush().unwrap();
    let _ = read_response(&mut stdout);
    writeln!(
        stdin,
        r#"{{"jsonrpc":"2.0","method":"notifications/initialized"}}"#
    )
    .unwrap();
    stdin.flush().unwrap();

    let call = r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"web_search_prime","arguments":{"search_query":"rust async"}}}"#;
    writeln!(stdin, "{call}").unwrap();
    stdin.flush().unwrap();

    let resp = read_response(&mut stdout);
    assert_eq!(resp["id"], 3);
    // The stub returns "[]" as the tool content; the key point is no error.
    assert!(resp.get("error").is_none(), "no error on stub call: {resp}");

    child.kill().ok();
    child.wait().ok();
}

#[test]
fn stdout_is_clean_json_rpc_only() {
    // The ADR-0004 discipline: nothing but JSON-RPC on stdout.
    // We run a full sequence and assert every stdout line parses as JSON.
    let mut child = spawn_server();
    let mut stdin = child.stdin.take().expect("no stdin");
    let mut stdout = BufReader::new(child.stdout.take().expect("no stdout"));

    let seq = [
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"0.1"}}}"#,
        r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}"#,
        r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"web_search_prime","arguments":{"search_query":"test"}}}"#,
    ];

    // Write all requests up front, then read responses. We expect 3 responses
    // (initialize, tools/list, tools/call); the notification produces none.
    for line in &seq {
        writeln!(stdin, "{line}").unwrap();
    }
    stdin.flush().unwrap();

    // Read everything stdout emits until the server is killed.
    let mut raw = String::new();
    // Give the server a moment to emit its responses.
    stdin.flush().unwrap();
    std::thread::sleep(std::time::Duration::from_millis(200));
    child.kill().ok();
    stdout.read_to_string(&mut raw).ok();
    child.wait().ok();

    // Every non-blank line must be valid JSON.
    let mut parsed = 0;
    for line in raw.lines() {
        if line.trim().is_empty() {
            continue;
        }
        serde_json::from_str::<serde_json::Value>(line)
            .unwrap_or_else(|e| panic!("stdout pollution — line not JSON-RPC:\n  {line}\n  {e}"));
        parsed += 1;
    }
    assert!(
        parsed >= 3,
        "expected at least 3 JSON-RPC responses on stdout, got {parsed}; raw:\n{raw}"
    );
}
