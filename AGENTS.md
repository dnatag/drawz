# AGENTS.md

Guidelines for AI agents working on this codebase.

## Project Overview

drawz is the **rendering guarantee layer** between AI agents and terminal display. Agents describe *what* to show (structured JSON or Mermaid). drawz guarantees *how* it looks (perfect alignment, always). Comparable to Mermaid for markdown, but for terminals.

## Crate Map

```
drawz-core/           Schema, rendering engine, alignment guarantee
├── src/lib.rs         Public API: render(Diagram, width) → RenderResult
├── src/schema.rs      DiagramInput, Diagram enum (8 types)
├── src/result.rs      RenderResult + RenderContext
├── src/render.rs      Dispatch: match Diagram → renderer
├── src/measure.rs     Unicode display width (THE alignment foundation)
├── src/frame.rs       Box drawing (┌─┐│└─┘)
├── src/renderers/     Per-type rendering modules
└── src/mermaid/       Mermaid parser (subset → internal types)

drawz-cli/            Binary: stdin JSON → stdout diagram
└── src/main.rs        Thin wrapper over drawz_core::render()
```

## Diagram Types

| Type | Minimal Input | Renders As |
|------|--------------|------------|
| `freeform` | `{"type":"freeform","content":"..."}` | Framed text block |
| `mermaid` | `{"type":"mermaid","code":"graph LR;A-->B"}` | Converts to internal type |
| `table` | `{"type":"table","headers":[...],"rows":[...]}` | Bordered table |
| `tree` | `{"type":"tree","indent":"src/\n  main.rs"}` | Tree with ├── └── |
| `flow` | `{"type":"flow","steps":["A","B","C"]}` | Framed pipeline |
| `state` | `{"type":"state","transitions":[{from,to,label}]}` | Framed state diagram |
| `sequence` | `{"type":"sequence","actors":[...],"messages":[...]}` | Framed sequence |
| `dag` | `{"type":"dag","edges":[{from,to}]}` | Framed dependency graph |

## Architecture Invariants

- **Alignment by construction.** All width decisions use `measure::display_width()`, never `.len()`. Padding via `measure::pad_right()` ensures every output line is exactly `total_width` chars wide.
- **No traits for rendering.** Match dispatch, one function per type. Compiler enforces exhaustiveness.
- **Renderers return `Result<Vec<String>, String>`.** Lines padded to `inner_width`. Caller applies frame.
- **No panics in library code.** Return errors in `RenderResult.errors`.
- **Deterministic.** Same input + same width = same output, always.

## COMM Style

When explaining architecture or changes:

- **Prefer ASCII diagrams over prose.** Use `boxes -d ansi` logic for framing.
- **Use tables for enumerations** (states, error variants, config options).
- **Keep prose to one-liners** that annotate the diagram.
- **Name things concretely** — actual struct/function names, not abstractions.

## Tech Stack

| Layer | Choice | Why |
|-------|--------|-----|
| Schema | serde + serde_json | JSON deserialization with tagged enums |
| Width | unicode-width | CJK=2, combining=0, the alignment foundation |
| Rendering | Pure Rust | Predictable, fast compile, no runtime deps |
| CLI | stdin/stdout + --width | Unix composable, pipe-friendly |
| MCP | (Phase 4) | Single `render_diagram` tool |

## Key Design Docs

| Doc | What it covers |
|-----|---------------|
| `design/prd.md` | Full requirements, input/output format, MCP design |
| `design/high-level-design.md` | Module layout, data flow, alignment guarantee |
| `design/adr/001-rendering-constraints.md` | Width rules, degradation strategies |
| `design/adr/002-response-contract.md` | `{output, fit, errors, warnings}` contract |
| `design/adr/003-mermaid-scope.md` | Mermaid subset parser decision |
