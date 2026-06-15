use std::sync::Arc;

use async_trait::async_trait;
use rust_mcp_sdk::{
    error::SdkResult,
    macros,
    mcp_server::{server_runtime, McpServerOptions, ServerHandler},
    schema::*,
    *,
};
use serde::Serialize;
use serde_json::{json, Value};

use drawz_core::schema::DiagramInput;

// -- Tool definitions --

fn render_diagram_tool() -> Tool {
    // No properties declared — schema is open-ended.
    // Agents use the description and introspect_drawz for field guidance.
    // This prevents strict MCP clients from stripping diagram-specific fields.
    Tool {
        name: "render_diagram".into(),
        description: Some("Render a structured diagram as perfectly-aligned ASCII/Unicode art for terminal display. Accepts JSON with a 'type' field: freeform, mermaid, flow, table, tree, sequence, state, or dag. Pass all diagram-specific fields directly (e.g. steps, headers, rows, indent, code, actors, messages, transitions, edges, nodes, content). Call introspect_drawz for per-type field documentation and examples.".into()),
        input_schema: ToolInputSchema::new(
            vec!["type".into()],
            None,
            None,
        ),
        annotations: None,
        execution: None,
        icons: vec![],
        meta: None,
        output_schema: None,
        title: None,
    }
}

#[macros::mcp_tool(
    name = "introspect_drawz",
    description = "List supported diagram types, show examples, and return the diagram type mapping table."
)]
#[derive(Debug, serde::Deserialize, serde::Serialize, macros::JsonSchema)]
pub struct IntrospectTool {}

// -- Handler --

#[derive(Default)]
struct DrawzHandler;

#[async_trait]
impl ServerHandler for DrawzHandler {
    async fn handle_list_tools_request(
        &self,
        _request: Option<PaginatedRequestParams>,
        _runtime: Arc<dyn McpServer>,
    ) -> std::result::Result<ListToolsResult, RpcError> {
        Ok(ListToolsResult {
            tools: vec![render_diagram_tool(), IntrospectTool::tool()],
            meta: None,
            next_cursor: None,
        })
    }

    async fn handle_call_tool_request(
        &self,
        params: CallToolRequestParams,
        _runtime: Arc<dyn McpServer>,
    ) -> std::result::Result<CallToolResult, CallToolError> {
        match params.name.as_str() {
            "render_diagram" => Ok(call_render(params.arguments)),
            "introspect_drawz" => Ok(call_introspect()),
            _ => Err(CallToolError::unknown_tool(params.name)),
        }
    }
}

fn call_render(args: Option<serde_json::Map<String, Value>>) -> CallToolResult {
    let args = Value::Object(args.unwrap_or_default());

    let input: DiagramInput = match serde_json::from_value(args) {
        Ok(d) => d,
        Err(e) => {
            let resp = RenderResponse {
                output: None,
                fit: false,
                errors: vec![format!("invalid input: {e}")],
                warnings: vec![],
            };
            return CallToolResult::text_content(vec![
                serde_json::to_string(&resp).unwrap_or_default().into(),
            ]);
        }
    };

    let result = drawz_core::render(&input.diagram, input.width);
    let resp = RenderResponse {
        output: result.output,
        fit: result.fit,
        errors: result.errors,
        warnings: result.warnings,
    };
    CallToolResult::text_content(vec![
        serde_json::to_string(&resp).unwrap_or_default().into(),
    ])
}

fn call_introspect() -> CallToolResult {
    let resp = json!({
        "types": [
            {"name": "freeform", "use_when": "Pre-formatted text, fix alignment", "minimal": r#"{"type":"freeform","content":"line1\nline2"}"#},
            {"name": "mermaid", "use_when": "Agent already has Mermaid code", "minimal": r#"{"type":"mermaid","code":"graph LR; A-->B"}"#},
            {"name": "flow", "use_when": "Pipelines, request flows", "minimal": r#"{"type":"flow","steps":["A","B","C"]}"#, "fields": "steps: string[] | steps: [{label,steps}] (nested) | nodes: string[] or [{id?,label}] + edges: [{from,to,label?}]"},
            {"name": "table", "use_when": "Comparisons, option matrices", "minimal": r#"{"type":"table","headers":["A","B"],"rows":[["1","2"]]}"#, "fields": "headers: string[], rows: string[][]"},
            {"name": "tree", "use_when": "File structures, hierarchies", "minimal": r#"{"type":"tree","indent":"src\n  main.rs\n  lib.rs"}"#, "fields": "indent: string (2-space indented) | root: {label, children: [{label,children}]}"},
            {"name": "sequence", "use_when": "API interactions, protocols", "minimal": r#"{"type":"sequence","actors":["A","B"],"messages":[{"from":"A","to":"B","label":"hi"}]}"#, "fields": "actors: string[], messages: [{from,to,label}]"},
            {"name": "state", "use_when": "State machines, lifecycles", "minimal": r#"{"type":"state","transitions":[{"from":"A","to":"B","label":"go"}]}"#, "fields": "transitions: [{from,to,label?}], states?: string[] or [{id?,label}]"},
            {"name": "dag", "use_when": "Task dependencies, build graphs", "minimal": r#"{"type":"dag","edges":[{"from":"A","to":"B"}]}"#, "fields": "edges: [{from,to,label?}], nodes?: string[] or [{id?,label}] (inferred from edges if omitted)"}
        ],
        "common_fields": {"width": "integer, default 80", "title": "string, shown in frame header"},
        "version": env!("CARGO_PKG_VERSION")
    });
    CallToolResult::text_content(vec![
        serde_json::to_string(&resp).unwrap_or_default().into(),
    ])
}

pub async fn run() -> SdkResult<()> {
    let server_details = InitializeResult {
        server_info: Implementation {
            name: "drawz".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            title: None,
            description: Some("Rendering guarantee layer for AI agent terminal output".into()),
            icons: vec![],
            website_url: None,
        },
        capabilities: ServerCapabilities {
            tools: Some(ServerCapabilitiesTools { list_changed: None }),
            ..Default::default()
        },
        protocol_version: ProtocolVersion::V2025_11_25.into(),
        instructions: None,
        meta: None,
    };

    let transport = StdioTransport::new(TransportOptions::default())?;
    let handler = DrawzHandler.to_mcp_server_handler();
    let server = server_runtime::create_server(McpServerOptions {
        transport,
        handler,
        server_details,
        task_store: None,
        client_task_store: None,
        message_observer: None,
    });
    server.start().await
}

#[derive(Serialize)]
struct RenderResponse {
    output: Option<String>,
    fit: bool,
    errors: Vec<String>,
    warnings: Vec<String>,
}
