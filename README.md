# drawz

The rendering guarantee layer between AI agents and terminal display. Agents describe structure as JSON, drawz guarantees perfect alignment.

## What It Solves

AI agents produce misaligned ASCII diagrams because they can't compute Unicode character widths reliably. drawz separates content (what to show) from rendering (how it looks) — the same way Mermaid works for browsers, but for terminals.

## Quick Start

```sh
# Install
cargo install --path crates/drawz-cli

# Linear flow
echo '{"type":"flow","steps":["Build","Test","Deploy"]}' | drawz

# Table
echo '{"type":"table","headers":["Feature","Status"],"rows":[["Alignment","✓"],["Unicode","✓"]]}' | drawz

# Tree
echo '{"type":"tree","indent":"src\n  main.rs\n  lib.rs"}' | drawz

# Sequence diagram
echo '{"type":"sequence","actors":["Client","Server"],"messages":[{"from":"Client","to":"Server","label":"GET /api"}]}' | drawz

# DAG
echo '{"type":"dag","edges":[{"from":"Parse","to":"Compile"},{"from":"Compile","to":"Link"}]}' | drawz

# Mermaid (agents already know this)
echo '{"type":"mermaid","code":"graph LR; A-->B-->C"}' | drawz

# MCP server mode
drawz mcp
```

## Supported Diagram Types

| Type | Use Case | Minimal Input |
|------|----------|---------------|
| `freeform` | Pre-formatted text, fix alignment | `{"type":"freeform","content":"..."}` |
| `mermaid` | Agent already has Mermaid code | `{"type":"mermaid","code":"..."}` |
| `table` | Comparisons, option matrices | `{"type":"table","headers":[...],"rows":[...]}` |
| `tree` | File structures, hierarchies | `{"type":"tree","indent":"src/\n  main.rs"}` |
| `flow` | Pipelines, request flows | `{"type":"flow","steps":["A","B","C"]}` |
| `state` | State machines, lifecycles | `{"type":"state","transitions":[...]}` |
| `sequence` | API interactions, protocols | `{"type":"sequence","actors":[...],"messages":[...]}` |
| `dag` | Task dependencies, build graphs | `{"type":"dag","edges":[...]}` |

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
```

## Building

```sh
cargo build --release
just test       # all tests
just test-int   # integration tests only
just test-print # visual output for review
just lint       # clippy pedantic
```

## Architecture

```
drawz-core/     Schema, rendering engine, alignment guarantee
drawz-cli/      Single binary: pipe mode + MCP server

Alignment invariant: every output line has display_width == total_width
Achieved by: measure.rs (display_width) → pad_right → frame_box
```

## Status

All 4 phases complete. 141 tests, 96%+ coverage, clippy pedantic clean.
