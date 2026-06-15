# AGENTS.md

Guidelines for AI agents working on this codebase.

## Project Overview

drawz is the **rendering guarantee layer** between AI agents and terminal display. Agents describe *what* to show (structured JSON or Mermaid). drawz guarantees *how* it looks (perfect alignment, always).

## Crate Map

```
drawz-core/           Schema, rendering engine, alignment guarantee
├── src/lib.rs         Public API: render(Diagram, width) → RenderResult
├── src/schema.rs      DiagramInput, Diagram enum (8 types)
├── src/result.rs      RenderResult + RenderContext (pub(crate))
├── src/render.rs      Dispatch: match Diagram → renderer
├── src/measure.rs     Unicode display width (THE alignment foundation)
├── src/frame.rs       Box drawing (┌─┐│└─┘)
├── src/renderers/     Per-type rendering modules
│   ├── freeform.rs    Pad-to-width text blocks
│   ├── table.rs       Auto-sized columns, proportional shrink
│   ├── tree.rs        Indent parsing + TreeNode recursion
│   ├── flow.rs        Step boxes, dashed subflow frames, graph mode
│   ├── state.rs       ╭─╮ rounded state boxes, labeled transitions
│   ├── sequence.rs    Actor columns, lifeline arrows
│   └── dag.rs         Kahn's topological layering
└── src/mermaid/       Mermaid parser (subset → internal types)
    └── mod.rs         flowchart, sequenceDiagram, stateDiagram

drawz-cli/            Single binary: pipe mode + MCP server
├── src/main.rs        Clap CLI: drawz | drawz mcp
└── src/mcp.rs         JSON-RPC stdio server (render_diagram + introspect_drawz)
```

## Architecture Invariants

- **Alignment by construction.** All width decisions use `measure::display_width()`, never `.len()`. Padding via `measure::pad_right()` ensures every output line is exactly `total_width` chars wide.
- **No traits for rendering.** Match dispatch, one function per type. Compiler enforces exhaustiveness.
- **Renderers return `Result<Vec<String>, String>`.** Lines padded to `inner_width`. Caller applies frame.
- **No panics in library code.** No `unwrap()` in non-test code. Return errors in `RenderResult.errors`.
- **Deterministic.** Same input + same width = same output, always.
- **Minimum width = 4.** Rejected at top of `render()`.
- **ANSI-safe truncation.** Truncated strings get `\x1b[0m` reset appended.

## Agent Communication Style

```
- Use render_diagram for all visual output — tables, trees, flows, sequences, state machines, DAGs, freeform, and mermaid. Never hand-draw them.
- Prefer diagrams over prose. If it can be a table or flow, render it.
- Keep prose to one-liners that annotate the rendered diagram.
```

## Tech Stack

| Layer | Choice | Why |
|-------|--------|-----|
| Schema | serde + serde_json | JSON deserialization with tagged enums |
| Width | unicode-width 0.2 | CJK=2, combining=0, the alignment foundation |
| CLI | clap 4 (derive) | Declarative argument parsing |
| Rendering | Pure Rust | Predictable, fast compile, no runtime deps |
| MCP | JSON-RPC stdio | Standard for local MCP servers |

## Testing Convention

- **Unit tests:** inline `#[cfg(test)] mod tests` in source files
- **Integration tests:** `tests/` directory, one file per diagram type
- **Test names:** `should_<behavior>_when_<condition>`
- **Happy-path tests:** include `assert_and_print` for visual review via `just test-print`

## Key Design Docs

| Doc | What it covers |
|-----|---------------|
| `design/prd.md` | Full requirements, input/output format, MCP design |
| `design/high-level-design.md` | Module layout, data flow, alignment guarantee |
| `design/adr/001-rendering-constraints.md` | Width rules, degradation strategies |
| `design/adr/002-response-contract.md` | `{output, fit, errors, warnings}` contract |
| `design/adr/003-mermaid-scope.md` | Mermaid subset parser decision |

## Useful Commands

```sh
just test        # all 141 tests
just test-int    # integration tests only
just test-print  # visual output for human review
just lint        # clippy pedantic
just build       # release binary
just install     # install to ~/.cargo/bin
```
