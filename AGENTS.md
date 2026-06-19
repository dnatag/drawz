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
│   ├── table.rs       Grid-bordered tables (┌┬┐ ├┼┤ └┴┘)
│   ├── tree.rs        Indent parsing (auto-detect 2/4-space) + TreeNode
│   ├── flow.rs        Vertical + horizontal (LR) pipelines
│   ├── state.rs       ╭─╮ rounded state boxes, labeled transitions
│   ├── sequence.rs    Actor columns, lifeline arrows, truncation warnings
│   └── dag.rs         ascii-dag Sugiyama layout (diamond, fan-out)
└── src/mermaid/       Mermaid subset parser
    ├── mod.rs         Module declarations + re-export
    ├── parse.rs       Dispatch: flowchart → Flow/DAG, sequence, state
    ├── helpers.rs     Shared: skip_first_line, split_statements
    ├── flowchart.rs   graph/flowchart parser (LR→horizontal, branching→DAG)
    ├── sequence.rs    sequenceDiagram parser
    └── state.rs       stateDiagram parser

drawz-cli/            Single binary: pipe mode + MCP server
├── src/main.rs        Clap CLI: drawz | drawz mcp
└── src/mcp.rs         rust-mcp-sdk server (render_diagram + introspect_drawz)
```

## Architecture Invariants

- **Alignment by construction.** All width decisions use `measure::display_width()`, never `.len()`. Padding via `measure::pad_right()` ensures every output line is exactly `total_width` chars wide.
- **No traits for rendering.** Match dispatch, one function per type. Compiler enforces exhaustiveness.
- **Renderers return `Result<Vec<String>, String>`.** Lines padded to `inner_width`. Caller applies frame.
- **No panics in library code.** No `unwrap()` in non-test code. Return errors in `RenderResult.errors`.
- **Deterministic.** Same input + same width = same output, always.
- **Minimum width = 4.** Rejected at top of `render()`.
- **fit: false when truncated.** All renderers push warnings and set fit=false when content is truncated.

## COMM Style

The `render_diagram` tool description teaches agents when and how to call drawz automatically via MCP. No system prompt changes are required for basic usage.

To make agents **prefer diagrams over prose**, add these lines to your `AGENTS.md` or `CLAUDE.md`:

```
- Use render_diagram for all visual output — tables, trees, flows, sequences, state machines, DAGs, freeform, and mermaid. Never hand-draw them.
- Prefer diagrams over prose. If it can be a table or flow, render it.
- Keep prose to one-liners that annotate the rendered diagram.
- For complex graphs with many nodes/edges, prefer type: "mermaid" with a code string over type: "dag" with verbose node/edge arrays.
- Always display the rendered output in a code block — tool results may not be visible to the user automatically.
```

## Tech Stack

| Layer | Choice | Why |
|-------|--------|-----|
| Schema | serde + serde_json | JSON deserialization with tagged enums |
| Width | unicode-width 0.2 | CJK=2, combining=0, the alignment foundation |
| DAG Layout | ascii-dag | Sugiyama algorithm, zero deps, diamond/fan-out |
| CLI | clap 4 (derive) | Declarative argument parsing |
| MCP | rust-mcp-sdk 0.9 | Protocol handling, stdio transport |
| Rendering | Pure Rust | Predictable, fast compile, no runtime deps |

## Testing Convention

- **Unit tests:** inline `#[cfg(test)] mod tests` in source files
- **Integration tests:** `tests/` directory, one file per diagram type
- **Test names:** `should_<behavior>_when_<condition>`
- **196 tests total**, clippy clean with `-D warnings -W clippy::perf`

## Useful Commands

```sh
cargo test                # all 196 tests
cargo test --test mermaid # single test file
cargo clippy --all-targets -- -D warnings
cargo build --release
cargo install --path crates/drawz-cli
```
