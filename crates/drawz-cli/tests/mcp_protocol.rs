use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::time::Duration;

use serde_json::{json, Value};

fn send_requests(requests: &[Value]) -> Vec<Value> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_drawz"))
        .arg("mcp")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to spawn drawz mcp");

    let mut stdin = child.stdin.take().unwrap();
    let stdout = child.stdout.take().unwrap();
    let mut reader = BufReader::new(stdout);

    // Send initialize and read response
    let init = json!({"jsonrpc": "2.0", "id": 0, "method": "initialize", "params": {
        "protocolVersion": "2024-11-05",
        "capabilities": {},
        "clientInfo": {"name": "test", "version": "0.1.0"}
    }});
    writeln!(stdin, "{}", serde_json::to_string(&init).unwrap()).unwrap();
    stdin.flush().unwrap();

    // Read initialize response
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();

    // Send initialized notification
    writeln!(
        stdin,
        "{}",
        serde_json::to_string(&json!({"jsonrpc": "2.0", "method": "notifications/initialized"}))
            .unwrap()
    )
    .unwrap();
    stdin.flush().unwrap();

    // Small delay to let server process the notification
    std::thread::sleep(Duration::from_millis(100));

    // Send actual requests
    let mut responses = Vec::new();
    for req in requests {
        writeln!(stdin, "{}", serde_json::to_string(req).unwrap()).unwrap();
        stdin.flush().unwrap();

        // Read response (skip notifications which have no id)
        loop {
            let mut resp_line = String::new();
            reader.read_line(&mut resp_line).unwrap();
            if resp_line.trim().is_empty() {
                continue;
            }
            let v: Value = serde_json::from_str(&resp_line).unwrap();
            if v.get("id").is_some() {
                responses.push(v);
                break;
            }
        }
    }

    // Close stdin to let the server exit
    drop(stdin);
    let _ = child.wait();

    responses
}

#[test]
fn initialize_returns_server_info() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_drawz"))
        .arg("mcp")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to spawn drawz mcp");

    let mut stdin = child.stdin.take().unwrap();
    let stdout = child.stdout.take().unwrap();
    let mut reader = BufReader::new(stdout);

    let init = json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {
        "protocolVersion": "2024-11-05",
        "capabilities": {},
        "clientInfo": {"name": "test", "version": "0.1.0"}
    }});
    writeln!(stdin, "{}", serde_json::to_string(&init).unwrap()).unwrap();
    stdin.flush().unwrap();

    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    drop(stdin);
    let _ = child.wait();

    let r: Value = serde_json::from_str(&line).unwrap();
    assert_eq!(r["id"], 1);
    assert_eq!(r["result"]["serverInfo"]["name"], "drawz");
    assert!(r["result"]["capabilities"]["tools"].is_object());
}

#[test]
fn tools_list_returns_two_tools() {
    let responses = send_requests(&[json!({
        "jsonrpc": "2.0", "id": 1, "method": "tools/list", "params": {}
    })]);

    assert_eq!(responses.len(), 1);
    let tools = responses[0]["result"]["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 2);

    let names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
    assert!(names.contains(&"render_diagram"));
    assert!(names.contains(&"introspect_drawz"));
}

#[test]
fn render_diagram_flow_returns_output() {
    let responses = send_requests(&[json!({
        "jsonrpc": "2.0", "id": 1, "method": "tools/call",
        "params": {
            "name": "render_diagram",
            "arguments": {"type": "flow", "steps": ["A", "B"], "width": 20}
        }
    })]);

    assert_eq!(responses.len(), 1);
    let content = responses[0]["result"]["content"][0]["text"]
        .as_str()
        .unwrap();
    let inner: Value = serde_json::from_str(content).unwrap();
    assert!(inner["output"].is_string());
    assert_eq!(inner["fit"], true);
    assert!(inner["errors"].as_array().unwrap().is_empty());
}

#[test]
fn render_diagram_invalid_input_returns_errors() {
    let responses = send_requests(&[json!({
        "jsonrpc": "2.0", "id": 1, "method": "tools/call",
        "params": {
            "name": "render_diagram",
            "arguments": {"type": "table", "headers": [], "rows": []}
        }
    })]);

    assert_eq!(responses.len(), 1);
    let content = responses[0]["result"]["content"][0]["text"]
        .as_str()
        .unwrap();
    let inner: Value = serde_json::from_str(content).unwrap();
    assert!(inner["output"].is_null());
    assert!(!inner["errors"].as_array().unwrap().is_empty());
}

#[test]
fn render_diagram_bad_json_returns_error() {
    let responses = send_requests(&[json!({
        "jsonrpc": "2.0", "id": 1, "method": "tools/call",
        "params": {
            "name": "render_diagram",
            "arguments": {"not_a_type": true}
        }
    })]);

    assert_eq!(responses.len(), 1);
    let r = &responses[0];
    // rmcp may return a protocol error or our handler returns errors in content
    let is_error = r["error"].is_object()
        || r["result"]["isError"] == true
        || r["result"]["content"][0]["text"]
            .as_str()
            .is_some_and(|t| t.contains("error"));
    assert!(is_error, "expected error, got: {r}");
}

#[test]
fn introspect_returns_all_types() {
    let responses = send_requests(&[json!({
        "jsonrpc": "2.0", "id": 1, "method": "tools/call",
        "params": { "name": "introspect_drawz", "arguments": {} }
    })]);

    assert_eq!(responses.len(), 1);
    let content = responses[0]["result"]["content"][0]["text"]
        .as_str()
        .unwrap();
    let inner: Value = serde_json::from_str(content).unwrap();
    let types = inner["types"].as_array().unwrap();
    assert_eq!(types.len(), 8);
    assert_eq!(inner["version"], "1.0.1");
}

#[test]
fn unknown_tool_returns_error() {
    let responses = send_requests(&[json!({
        "jsonrpc": "2.0", "id": 1, "method": "tools/call",
        "params": { "name": "nonexistent", "arguments": {} }
    })]);

    assert_eq!(responses.len(), 1);
    let r = &responses[0];
    // rmcp returns an error for unknown tools
    let has_error = r["error"].is_object()
        || r["result"]["isError"] == true
        || r["result"]["content"][0]["text"]
            .as_str()
            .is_some_and(|t| t.contains("error") || t.contains("not found"));
    assert!(has_error, "expected error response, got: {r}");
}

#[test]
fn unknown_method_returns_rpc_error() {
    let responses = send_requests(&[json!({
        "jsonrpc": "2.0", "id": 1, "method": "foo/bar", "params": {}
    })]);

    assert_eq!(responses.len(), 1);
    assert!(responses[0]["error"].is_object());
}

#[test]
fn multiple_requests_in_sequence() {
    let responses = send_requests(&[
        json!({"jsonrpc": "2.0", "id": 1, "method": "tools/list", "params": {}}),
        json!({"jsonrpc": "2.0", "id": 2, "method": "tools/call", "params": {
            "name": "render_diagram",
            "arguments": {"type": "freeform", "content": "hello", "width": 20}
        }}),
    ]);

    assert_eq!(responses.len(), 2);
    assert_eq!(responses[0]["id"], 1);
    assert_eq!(responses[1]["id"], 2);
}
