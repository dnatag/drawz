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
        description: Some("Render a structured diagram as perfectly-aligned ASCII/Unicode art for terminal display. Accepts JSON with a 'type' field: freeform, mermaid, flow, table, tree, sequence, state, or dag. Pass all diagram-specific fields directly (e.g. steps, headers, rows, indent, code, actors, messages, transitions, edges, nodes, content). The response includes 'rendered_width' showing the diagram's character width. If the user says output is too wide, re-render with width set to half of rendered_width. The width is remembered for the session. IMPORTANT: Always display the 'output' field in a code block — tool results are not shown to the user automatically. Call introspect_drawz for per-type field documentation.".into()),
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

use std::sync::atomic::{AtomicU16, Ordering};

// -- Handler --

#[derive(Default)]
struct DrawzHandler {
    /// Session width: once set by the agent, persists for all subsequent calls
    session_width: AtomicU16,
}

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
            "render_diagram" => Ok(call_render(params.arguments, &self.session_width)),
            "introspect_drawz" => Ok(call_introspect()),
            _ => Err(CallToolError::unknown_tool(params.name)),
        }
    }
}

fn call_render(args: Option<serde_json::Map<String, Value>>, session_width: &AtomicU16) -> CallToolResult {
    let args = Value::Object(args.unwrap_or_default());

    let input: DiagramInput = match serde_json::from_value(args.clone()) {
        Ok(d) => d,
        Err(e) => {
            let resp = RenderResponse {
                output: None,
                fit: false,
                rendered_width: 0,
                errors: vec![format!("invalid input: {e}")],
                warnings: vec![],
            };
            let mut result = CallToolResult::text_content(vec![
                to_json(&resp).into(),
            ]);
            result.is_error = Some(true);
            return result;
        }
    };

    // Detect if agent explicitly passed a width field (vs relying on default)
    let explicit_width = args.as_object().and_then(|m| m.get("width")).and_then(|v| v.as_u64()).map(|w| w as u16);

    let width = if let Some(w) = explicit_width {
        // Agent passed explicit width — remember it for the session
        session_width.store(w, Ordering::Relaxed);
        w
    } else {
        // No explicit width — use session width if set, else default
        let sw = session_width.load(Ordering::Relaxed);
        if sw > 0 { sw } else { input.width }
    };

    let result = drawz_core::render(&input.diagram, width);
    let has_errors = result.output.is_none() && !result.errors.is_empty();

    let resp = RenderResponse {
        output: result.output.clone(),
        fit: result.fit,
        rendered_width: result.output.as_ref()
            .and_then(|o| o.lines().next())
            .map(drawz_core::measure::display_width)
            .unwrap_or(0),
        errors: result.errors,
        warnings: result.warnings,
    };
    let mut call_result = CallToolResult::text_content(vec![to_json(&resp).into()]);
    if has_errors {
        call_result.is_error = Some(true);
    }
    call_result
}

fn call_introspect() -> CallToolResult {
    let resp = json!({
        "types": [
            {"name": "freeform", "use_when": "Pre-formatted text or fix alignment of hand-drawn ASCII. Escape hatch: paste any misaligned diagram to get uniform-width padding", "minimal": r#"{"type":"freeform","content":"line1\nline2"}"#, "fields": "content: string (newline-separated) | lines: string[]"},
            {"name": "mermaid", "use_when": "Agent already has Mermaid code", "minimal": r#"{"type":"mermaid","code":"graph LR; A-->B"}"#},
            {"name": "flow", "use_when": "Pipelines, request flows", "minimal": r#"{"type":"flow","steps":["A","B","C"]}"#, "fields": "steps: string[] | steps: [{label,steps}] (nested) | nodes: string[] or [{id?,label}] + edges: [{from,to,label?}]. Optional: direction: \"LR\" for horizontal"},
            {"name": "table", "use_when": "Comparisons, option matrices", "minimal": r#"{"type":"table","headers":["A","B"],"rows":[["1","2"]]}"#, "fields": "headers: string[], rows: string[][]"},
            {"name": "tree", "use_when": "File structures, hierarchies", "minimal": r#"{"type":"tree","indent":"src\n  main.rs\n  lib.rs"}"#, "fields": "indent: string (2-space indented) | root: {label, children: [{label,children}]}"},
            {"name": "sequence", "use_when": "API interactions, protocols", "minimal": r#"{"type":"sequence","actors":["A","B"],"messages":[{"from":"A","to":"B","label":"hi"}]}"#, "fields": "actors: string[], messages: [{from,to,label}]"},
            {"name": "state", "use_when": "State machines, lifecycles", "minimal": r#"{"type":"state","transitions":[{"from":"A","to":"B","label":"go"}]}"#, "fields": "transitions: [{from,to,label?}], states?: string[] or [{id?,label}]"},
            {"name": "dag", "use_when": "Task dependencies, build graphs", "minimal": r#"{"type":"dag","edges":[{"from":"A","to":"B"}]}"#, "fields": "edges: [{from,to,label?}], nodes?: string[] or [{id?,label}] (inferred from edges if omitted)"}
        ],
        "common_fields": {"width": "integer, default 80", "title": "string, shown in frame header"},
        "version": env!("CARGO_PKG_VERSION")
    });
    CallToolResult::text_content(vec![to_json(&resp).into()])
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
    let handler = DrawzHandler::default().to_mcp_server_handler();
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
    /// The actual display width of the rendered diagram in characters.
    /// If this exceeds your display width, the diagram will wrap.
    rendered_width: usize,
    errors: Vec<String>,
    warnings: Vec<String>,
}

/// Serialize to JSON string. Cannot fail for our response types (String/Vec/bool fields).
fn to_json(value: &impl Serialize) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| r#"{"errors":["serialization failed"]}"#.into())
}
