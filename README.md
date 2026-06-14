# drawz

The rendering guarantee layer between AI agents and terminal display. Agents describe structure as JSON, drawz guarantees perfect alignment.

## What It Solves

AI agents produce misaligned ASCII diagrams because they can't compute Unicode character widths reliably. drawz separates content (what to show) from rendering (how it looks) — the same way Mermaid works for browsers, but for terminals.

## Quick Start

```sh
# Linear flow
echo '{"type":"flow","steps":["Build","Test","Deploy"]}' | drawz

# Table
echo '{"type":"table","headers":["Feature","Status"],"rows":[["Alignment","Guaranteed"],["Unicode","Handled"]]}' | drawz

# Tree
echo '{"type":"tree","indent":"src/\n  main.rs\n  lib.rs"}' | drawz

# Mermaid (agents already know this)
echo '{"type":"mermaid","code":"graph LR; A-->B-->C"}' | drawz
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
PRIMARY:   MCP tool (render_diagram) — structured response, no escaping issues
SECONDARY: CLI pipe — drawz <<'EOF' ... EOF
LIBRARY:   use drawz_core::render(...)
```

## Building

```sh
cargo build --release
```

## Status

Design complete. Implementation in progress. See `design/prd.md` for full requirements.
