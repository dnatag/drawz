# drawz

The rendering guarantee layer between AI agents and terminal display. Agents describe structure as JSON, drawz guarantees perfect alignment.

## What It Solves

AI agents produce misaligned ASCII diagrams because they can't compute Unicode character widths reliably. drawz separates content (what to show) from rendering (how it looks) — the same way Mermaid works for browsers, but for terminals.

## Quick Start

```sh
# Install
cargo install --path crates/drawz-cli

# Horizontal flow
echo '{"type":"flow","direction":"LR","steps":["Build","Test","Deploy"]}' | drawz

# Table with grid borders
echo '{"type":"table","headers":["Feature","Status"],"rows":[["Alignment","✓"],["Unicode","✓"]]}' | drawz

# Tree (auto-detects 2-space or 4-space indent)
echo '{"type":"tree","indent":"src\n  main.rs\n  lib.rs\n  utils/\n    helpers.rs"}' | drawz

# DAG with Sugiyama layout (diamond, fan-out)
echo '{"type":"dag","edges":[{"from":"Parse","to":"Lint"},{"from":"Parse","to":"Compile"},{"from":"Lint","to":"Link"},{"from":"Compile","to":"Link"}]}' | drawz

# Sequence diagram
echo '{"type":"sequence","actors":["Client","Server"],"messages":[{"from":"Client","to":"Server","label":"GET /api"}]}' | drawz

# Mermaid (LR = horizontal, TD = vertical, branching = DAG)
echo '{"type":"mermaid","code":"graph LR; A[Parse]-->B[Layout]-->C[Render]-->D[Output]"}' | drawz

# MCP server mode
drawz mcp
```

## Supported Diagram Types

| Type | Use Case | Minimal Input |
|------|----------|---------------|
| `flow` | Pipelines, request flows | `{"type":"flow","steps":["A","B","C"]}` |
| `flow` (LR) | Horizontal pipelines | `{"type":"flow","direction":"LR","steps":["A","B"]}` |
| `table` | Comparisons, option matrices | `{"type":"table","headers":[...],"rows":[...]}` |
| `tree` | File structures, hierarchies | `{"type":"tree","indent":"src/\n  main.rs"}` |
| `sequence` | API interactions, protocols | `{"type":"sequence","actors":[...],"messages":[...]}` |
| `state` | State machines, lifecycles | `{"type":"state","transitions":[...]}` |
| `dag` | Task dependencies, build graphs | `{"type":"dag","edges":[...]}` |
| `mermaid` | Agent already has Mermaid code | `{"type":"mermaid","code":"..."}` |
| `freeform` | Fix alignment of hand-drawn text | `{"type":"freeform","content":"..."}` |

## Integration

```
MCP:      drawz mcp        — JSON-RPC over stdio, 2 tools (render_diagram, introspect_drawz)
CLI pipe: echo '...' | drawz — stdin JSON → stdout diagram
Library:  drawz_core::render(&diagram, width) → RenderResult
```

## CLI Usage

```
Usage: drawz [OPTIONS] [COMMAND]

Commands:
  mcp   Start MCP server (JSON-RPC over stdio)
  help  Print this message or the help of the given subcommand(s)

Options:
  -w, --width <WIDTH>  Maximum output width in characters
  -h, --help           Print help
  -V, --version        Print version
```

## Response Contract

All render calls return:

```json
{
  "output": "<rendered diagram or null>",
  "fit": true,
  "errors": [],
  "warnings": []
}

- `fit: false` + warnings when content is truncated to fit width
- `output: null` + errors when input is invalid

## Agent Communication Style

The `render_diagram` tool description teaches agents when and how to call drawz automatically via MCP. No system prompt changes are required for basic usage.

To make agents **prefer diagrams over prose**, add these lines to your `AGENTS.md` or `CLAUDE.md`:

```
- Use render_diagram for all visual output — tables, trees, flows, sequences, state machines, DAGs, freeform, and mermaid. Never hand-draw them.
- Prefer diagrams over prose. If it can be a table or flow, render it.
- Keep prose to one-liners that annotate the rendered diagram.
- For complex graphs with many nodes/edges, prefer type: "mermaid" with a code string over type: "dag" with verbose node/edge arrays.
- Always display the rendered output in a code block — tool results may not be visible to the user automatically.
```

### Installation

```sh
cargo install --path crates/drawz-cli
```

### MCP Setup

Add to your MCP client configuration (e.g., Claude Desktop, Kiro, Cursor):

```json
{
  "mcpServers": {
    "drawz": {
      "command": "drawz",
      "args": ["mcp"]
    }
  }
}
```

### CLI Pipe Usage

```sh
echo '{"type":"flow","direction":"LR","steps":["Build","Test","Deploy"]}' | drawz
echo '{"type":"table","headers":["A","B"],"rows":[["1","2"]]}' | drawz -w 60
```

## Building

```sh
cargo build --release
cargo test              # all tests
cargo clippy --all-targets -- -D warnings
```

## Architecture

```
drawz-core/     Schema, rendering engine, alignment guarantee
drawz-cli/      Single binary: pipe mode + MCP server (rust-mcp-sdk)

Key crates:
  ascii-dag       Sugiyama DAG layout (diamond, fan-out, crossing reduction)
  unicode-width   CJK=2, combining=0 (THE alignment foundation)
  rust-mcp-sdk    MCP protocol handling (stdio transport)

Alignment invariant: every output line has display_width == total_width
Achieved by: measure.rs (display_width) → pad_right → frame_box
```

## Status

196 tests, clippy clean, all diagram types rendering correctly.
```
