use std::io::{self, BufRead, Write};

use serde::Deserialize;
use serde_json::{json, Value};

use drawz_core::schema::DiagramInput;

pub fn run() {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };

        if line.trim().is_empty() {
            continue;
        }

        let request: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                write_error(&mut stdout, None, &json!({"code": -32700, "message": format!("Parse error: {e}")}));
                continue;
            }
        };

        let result = handle_request(&request);
        match result {
            Some(Response::Success(v)) => write_success(&mut stdout, request.id.as_ref(), &v),
            Some(Response::RpcError(v)) => write_error(&mut stdout, request.id.as_ref(), &v),
            None => {} // notification — no response
        }
    }
}

enum Response {
    Success(Value),
    RpcError(Value),
}

fn handle_request(req: &JsonRpcRequest) -> Option<Response> {
    match req.method.as_str() {
        "initialize" => Some(Response::Success(handle_initialize())),
        "initialized" | "notifications/initialized" => None,
        "tools/list" => Some(Response::Success(handle_tools_list())),
        "tools/call" => Some(Response::Success(handle_tools_call(req.params.as_ref()))),
        _ => Some(Response::RpcError(json!({"code": -32601, "message": format!("Method not found: {}", req.method)}))),
    }
}

fn handle_initialize() -> Value {
    json!({
        "protocolVersion": "2024-11-05",
        "capabilities": { "tools": {} },
        "serverInfo": { "name": "drawz", "version": env!("CARGO_PKG_VERSION") }
    })
}

fn handle_tools_list() -> Value {
    json!({
        "tools": [
            {
                "name": "render_diagram",
                "description": "Render a structured diagram as perfectly-aligned ASCII/Unicode art for terminal display. Accepts JSON with a 'type' field: freeform, mermaid, flow, table, tree, sequence, state, or dag. Returns rendered output with fit/error metadata. Example: {\"type\":\"flow\",\"steps\":[\"Build\",\"Test\",\"Deploy\"]}",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "type": { "type": "string", "enum": ["freeform", "mermaid", "flow", "table", "tree", "sequence", "state", "dag"] },
                        "width": { "type": "integer", "default": 80 },
                        "title": { "type": "string" }
                    },
                    "required": ["type"],
                    "additionalProperties": true
                }
            },
            {
                "name": "introspect_drawz",
                "description": "List supported diagram types, show examples, and return the diagram type mapping table.",
                "inputSchema": { "type": "object", "properties": {}, "additionalProperties": false }
            }
        ]
    })
}

fn handle_tools_call(params: Option<&Value>) -> Value {
    let Some(params) = params else {
        return tool_error("missing params");
    };

    let name = params.get("name").and_then(Value::as_str).unwrap_or("");
    let arguments = params.get("arguments").cloned().unwrap_or(Value::Object(serde_json::Map::default()));

    match name {
        "render_diagram" => call_render_diagram(&arguments),
        "introspect_drawz" => call_introspect(),
        _ => tool_error(&format!("unknown tool: {name}")),
    }
}

fn call_render_diagram(args: &Value) -> Value {
    let input: DiagramInput = match serde_json::from_value(args.clone()) {
        Ok(d) => d,
        Err(e) => {
            return tool_result(&json!({
                "output": null, "fit": false,
                "errors": [format!("invalid input: {e}")], "warnings": []
            }));
        }
    };

    let result = drawz_core::render(&input.diagram, input.width);
    tool_result(&json!({
        "output": result.output, "fit": result.fit,
        "errors": result.errors, "warnings": result.warnings
    }))
}

fn call_introspect() -> Value {
    tool_result(&json!({
        "types": [
            {"name": "freeform", "use_when": "Pre-formatted text, fix alignment", "minimal": "{\"type\":\"freeform\",\"content\":\"...\"}"},
            {"name": "mermaid", "use_when": "Agent already has Mermaid code", "minimal": "{\"type\":\"mermaid\",\"code\":\"graph LR; A-->B\"}"},
            {"name": "flow", "use_when": "Pipelines, request flows", "minimal": "{\"type\":\"flow\",\"steps\":[\"A\",\"B\",\"C\"]}"},
            {"name": "table", "use_when": "Comparisons, option matrices", "minimal": "{\"type\":\"table\",\"headers\":[\"A\",\"B\"],\"rows\":[[\"1\",\"2\"]]}"},
            {"name": "tree", "use_when": "File structures, hierarchies", "minimal": "{\"type\":\"tree\",\"indent\":\"src\\n  main.rs\"}"},
            {"name": "sequence", "use_when": "API interactions, protocols", "minimal": "{\"type\":\"sequence\",\"actors\":[\"A\",\"B\"],\"messages\":[{\"from\":\"A\",\"to\":\"B\",\"label\":\"hi\"}]}"},
            {"name": "state", "use_when": "State machines, lifecycles", "minimal": "{\"type\":\"state\",\"transitions\":[{\"from\":\"A\",\"to\":\"B\"}]}"},
            {"name": "dag", "use_when": "Task dependencies, build graphs", "minimal": "{\"type\":\"dag\",\"edges\":[{\"from\":\"A\",\"to\":\"B\"}]}"}
        ],
        "version": env!("CARGO_PKG_VERSION")
    }))
}

fn tool_result(content: &Value) -> Value {
    json!({ "content": [{"type": "text", "text": serde_json::to_string(content).unwrap_or_default()}] })
}

fn tool_error(msg: &str) -> Value {
    json!({ "isError": true, "content": [{"type": "text", "text": msg}] })
}

fn write_success(out: &mut impl Write, id: Option<&Value>, result: &Value) {
    let response = json!({ "jsonrpc": "2.0", "id": id, "result": result });
    let serialized = serde_json::to_string(&response).unwrap_or_default();
    let _ = writeln!(out, "{serialized}");
    let _ = out.flush();
}

fn write_error(out: &mut impl Write, id: Option<&Value>, error: &Value) {
    let response = json!({ "jsonrpc": "2.0", "id": id, "error": error });
    let serialized = serde_json::to_string(&response).unwrap_or_default();
    let _ = writeln!(out, "{serialized}");
    let _ = out.flush();
}

#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    #[allow(dead_code)]
    jsonrpc: Option<String>,
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}
