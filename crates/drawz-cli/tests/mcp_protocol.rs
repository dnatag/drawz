use std::io::Write;
use std::process::{Command, Stdio};

use serde_json::{json, Value};

fn send_requests(requests: &[Value]) -> Vec<Value> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_drawz"))
        .arg("mcp")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to spawn drawz mcp");

    let stdin = child.stdin.as_mut().unwrap();
    for req in requests {
        writeln!(stdin, "{}", serde_json::to_string(req).unwrap()).unwrap();
    }
    drop(child.stdin.take());

    let output = child.wait_with_output().expect("failed to read output");
    let stdout = String::from_utf8_lossy(&output.stdout);

    stdout
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).expect("invalid JSON response"))
        .collect()
}

#[test]
fn initialize_returns_server_info() {
    let responses = send_requests(&[json!({
        "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {}
    })]);

    assert_eq!(responses.len(), 1);
    let r = &responses[0];
    assert_eq!(r["id"], 1);
    assert_eq!(r["result"]["serverInfo"]["name"], "drawz");
    assert_eq!(r["result"]["protocolVersion"], "2024-11-05");
    assert!(r["result"]["capabilities"]["tools"].is_object());
}

#[test]
fn tools_list_returns_two_tools() {
    let responses = send_requests(&[json!({
        "jsonrpc": "2.0", "id": 1, "method": "tools/list", "params": {}
    })]);

    let tools = responses[0]["result"]["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 2);
    assert_eq!(tools[0]["name"], "render_diagram");
    assert_eq!(tools[1]["name"], "introspect_drawz");
    assert!(tools[0]["inputSchema"]["properties"]["type"].is_object());
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

    let content = &responses[0]["result"]["content"][0]["text"];
    let inner: Value = serde_json::from_str(content.as_str().unwrap()).unwrap();
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

    let content = &responses[0]["result"]["content"][0]["text"];
    let inner: Value = serde_json::from_str(content.as_str().unwrap()).unwrap();
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

    let content = &responses[0]["result"]["content"][0]["text"];
    let inner: Value = serde_json::from_str(content.as_str().unwrap()).unwrap();
    assert!(!inner["errors"].as_array().unwrap().is_empty());
}

#[test]
fn introspect_returns_all_types() {
    let responses = send_requests(&[json!({
        "jsonrpc": "2.0", "id": 1, "method": "tools/call",
        "params": { "name": "introspect_drawz", "arguments": {} }
    })]);

    let content = &responses[0]["result"]["content"][0]["text"];
    let inner: Value = serde_json::from_str(content.as_str().unwrap()).unwrap();
    let types = inner["types"].as_array().unwrap();
    assert_eq!(types.len(), 8);
    assert_eq!(inner["version"], "0.1.0");
}

#[test]
fn unknown_tool_returns_error() {
    let responses = send_requests(&[json!({
        "jsonrpc": "2.0", "id": 1, "method": "tools/call",
        "params": { "name": "nonexistent", "arguments": {} }
    })]);

    assert_eq!(responses[0]["result"]["isError"], true);
}

#[test]
fn unknown_method_returns_rpc_error() {
    let responses = send_requests(&[json!({
        "jsonrpc": "2.0", "id": 1, "method": "foo/bar", "params": {}
    })]);

    assert!(responses[0]["error"].is_object());
    assert_eq!(responses[0]["error"]["code"], -32601);
}

#[test]
fn multiple_requests_in_sequence() {
    let responses = send_requests(&[
        json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {}}),
        json!({"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}}),
        json!({"jsonrpc": "2.0", "id": 3, "method": "tools/call", "params": {
            "name": "render_diagram",
            "arguments": {"type": "freeform", "content": "hello", "width": 20}
        }}),
    ]);

    assert_eq!(responses.len(), 3);
    assert_eq!(responses[0]["id"], 1);
    assert_eq!(responses[1]["id"], 2);
    assert_eq!(responses[2]["id"], 3);
}
