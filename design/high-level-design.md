# High-Level Design — drawz-core

## Module Layout

```
drawz-core/src/
├── lib.rs              Public API: render(Diagram, width) → RenderResult
├── schema.rs           Input types (DiagramInput, Diagram enum, per-type structs)
├── result.rs           RenderResult + RenderContext
├── render.rs           Dispatch: match Diagram → per-type renderer
├── measure.rs          Unicode display width, padding, truncation (uses unicode-width crate)
├── frame.rs            Box drawing (┌─┐│└─┘), title insertion
├── renderers/
│   ├── mod.rs
│   ├── freeform.rs     Wrap in frame, fix alignment
│   ├── table.rs        Column sizing, borders, truncation
│   ├── tree.rs         Indent parsing, tree connectors (├── └──)
│   ├── flow.rs         Linear (steps), nested, full (nodes+edges)
│   ├── state.rs        State boxes + labeled transitions
│   ├── sequence.rs     Actor lifelines + message arrows
│   └── dag.rs          Topological sort + vertical layout
└── mermaid/
    ├── mod.rs           Parse dispatch (detect graph/sequence/state)
    ├── flowchart.rs     Parse graph LR/TD → FlowDiagram
    ├── sequence.rs      Parse sequenceDiagram → SequenceDiagram
    └── state.rs         Parse stateDiagram-v2 → StateDiagram
```

## Key Types

```rust
// result.rs

/// The structured response from every render call.
pub struct RenderResult {
    pub output: Option<String>,
    pub fit: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// Mutable context threaded through rendering.
/// Lives in result.rs alongside RenderResult (same concern: render output/metadata).
pub struct RenderContext {
    /// Available width for content (excluding frame borders if framed).
    pub inner_width: usize,
    /// Total output width (what the user specified).
    pub total_width: u16,
    /// Accumulated warnings during rendering.
    pub warnings: Vec<String>,
}
```

## Width Ownership (critical for alignment)

```
┌──────────────────────────────────────────────────────────────────────────┐
│ Who computes what                                                        │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│ User specifies:  total_width (e.g., 80)                                  │
│                                                                          │
│ render.rs computes inner_width based on framing rules:                   │
│   Framed types (freeform, flow, state, sequence, dag):                   │
│     inner_width = total_width - 4    (│ + space on each side)            │
│                                                                          │
│   Unframed types (table, tree):                                          │
│     inner_width = total_width        (no border overhead)                │
│                                                                          │
│ render.rs passes RenderContext { inner_width, total_width, warnings }     │
│ to the renderer.                                                         │
│                                                                          │
│ Renderer produces lines where: display_width(line) == inner_width         │
│ Most renderers achieve this via pad_right() on the full line.             │
│ Table achieves this via per-cell padding then joining with │ separators.  │
│                                                                          │
│ render.rs then either:                                                   │
│   • frame_box(lines, title) — adds borders (framed types)                │
│   • joins lines directly    — no borders (table, tree)                   │
│                                                                          │
│ Result: every output line has display_width == total_width                │
│ (framed: inner + borders = total. unframed: inner = total)               │
│                                                                          │
└──────────────────────────────────────────────────────────────────────────┘
```

## Data Flow (including error paths)

```
DiagramInput (JSON)
    │
    ├─ width: u16
    └─ diagram: Diagram
         │
         ▼
    render::render(diagram, width) → RenderResult
         │
         ├─ Validate input (missing fields?) ──── ERROR → RenderResult { output: None, errors }
         │
         ├─ Diagram::Mermaid → mermaid::parse(code)
         │       │
         │       ├─ parse fails → RenderResult { output: None, errors }
         │       └─ parse OK → Diagram variant → re-dispatch below
         │
         ├─ Compute inner_width (framed vs unframed)
         │
         ├─ Call renderer:
         │   ├─ Diagram::Freeform → renderers::freeform::render(d, ctx)
         │   ├─ Diagram::Table    → renderers::table::render(d, ctx)
         │   ├─ Diagram::Tree     → renderers::tree::render(d, ctx)
         │   ├─ Diagram::Flow     → renderers::flow::render(d, ctx)
         │   ├─ Diagram::State    → renderers::state::render(d, ctx)
         │   ├─ Diagram::Sequence → renderers::sequence::render(d, ctx)
         │   └─ Diagram::Dag      → renderers::dag::render(d, ctx)
         │         │
         │         ├─ Cannot fit? → RenderResult { output: None, errors, warnings }
         │         └─ OK → Vec<String> (lines, each padded to inner_width)
         │
         ├─ Framed type? → frame::frame_box(lines, title, total_width)
         │  Unframed?    → use lines directly
         │
         └─ Assemble RenderResult { output: Some(joined), fit, errors: [], warnings }
```

## Design Decisions

### No trait for rendering

Each renderer is a standalone function: `fn render(diagram: &XxxDiagram, ctx: &mut RenderContext) -> Result<Vec<String>, String>`

Returns `Ok(lines)` on success, `Err(message)` when rendering is impossible. Dispatch is a match in `render.rs`. The compiler enforces exhaustiveness.

### RenderContext lives in `result.rs`

Alongside `RenderResult`. Both are rendering metadata — one is input context, the other is output. Renderers import from `result.rs`, not from `frame.rs`.

### `measure.rs` owns ALL width-related functions

```rust
// measure.rs — sole owner of width calculation and padding.
// Depends on `unicode-width` crate externally.

/// Display width of a string. Skips ANSI escapes.
/// CJK = 2, normal = 1, combining = 0.
pub fn display_width(s: &str) -> usize

/// Pad string with spaces on the right to reach exact target display width.
/// If string already exceeds target, truncates instead.
pub fn pad_right(s: &str, target_width: usize) -> String

/// Truncate string to fit max display width, append "…" (width 1).
pub fn truncate(s: &str, max_width: usize) -> String
```

No padding functions exist anywhere else. `frame.rs` calls `measure::pad_right`. Renderers call `measure::pad_right`. One source of truth.

### `frame.rs` only does box-drawing

```rust
// frame.rs — draws the ┌─┐│└─┘ box around pre-padded lines.

/// Wrap pre-padded lines in a Unicode box with optional title.
/// Assumes all lines already have display_width == inner_width.
/// Output lines will have display_width == inner_width + 4.
pub fn frame_box(lines: &[String], title: Option<&str>, total_width: u16) -> Vec<String>
```

`frame_box` doesn't pad content — that's the renderer's job. It only adds `│ ` prefix and ` │` suffix, plus top/bottom borders.

### Normalization before rendering

Types with "minimal forms" normalize to a canonical internal representation before layout:

```rust
// Normalized forms (private to each renderer module):

// flow.rs
struct NormalizedFlow {
    nodes: Vec<(String, String)>,  // (id, label) — id defaults to label
    edges: Vec<(String, String, Option<String>)>,  // (from_id, to_id, label)
    sub_flows: Vec<(String, NormalizedFlow)>,  // (parent_label, sub)
}

// tree.rs
// indent string "src/\n  main.rs" → TreeNode { label, children }
// Both indent and root normalize to the same TreeNode structure.

// state.rs
// transitions alone → infer states from from/to fields
// Normalized: states Vec + transitions Vec (always both present)
```

### Framed vs Unframed dispatch

```rust
// In render.rs
let framed = matches!(diagram, Diagram::Freeform(_) | Diagram::Flow(_) |
    Diagram::State(_) | Diagram::Sequence(_) | Diagram::Dag(_));

let title = extract_title(&diagram);  // each variant has Option<String> title field
let inner_width = if framed { total_width as usize - 4 } else { total_width as usize };
let ctx = RenderContext { inner_width, total_width, warnings: vec![] };

let lines = call_renderer(diagram, &mut ctx)?;

let output_lines = if framed {
    frame::frame_box(&lines, title.as_deref(), total_width)
} else {
    lines
};
```

### `fit` flag determination

```rust
let fit = ctx.warnings.is_empty();
// fit = true  → perfect render, no compromises
// fit = false → warnings present (truncation, layout change) OR errors present
```

## Alignment Guarantee (learned from `boxes`)

The `boxes` tool guarantees alignment through a "rich string" type (`bxstr_t`) that pre-computes display width for every string using `uc_width()` per character. Padding is always computed from display width, never byte length.

drawz adopts the same principle: **guarantee alignment by construction, not validation.**

### The invariant

Every output line has: `display_width(line) == total_width`

This holds because:
1. Renderers pad all content to `inner_width` via `measure::pad_right()`
2. `frame_box` adds exactly 4 chars of border (`│ ` + ` │`) for framed types
3. Unframed types: `inner_width == total_width`, so renderer output IS the final output

### Why no validator trait is needed

If every renderer uses `pad_right(content, inner_width)`, misalignment can't happen — the padding function guarantees it. A `debug_assert!` in tests verifies this as a safety net:

```rust
#[cfg(debug_assertions)]
fn assert_alignment(lines: &[String], expected_width: usize) {
    for (i, line) in lines.iter().enumerate() {
        assert_eq!(measure::display_width(line), expected_width,
            "alignment broken on line {i}: {line:?}");
    }
}
```

### ANSI escape handling

`display_width` skips `ESC[...m` sequences (invisible). This allows future ANSI color support without breaking alignment.

## Dependencies

```toml
# drawz-core/Cargo.toml
[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
unicode-width = "0.2"  # Foundation of the alignment guarantee
```

## Internal Dependency Graph

```
measure.rs          (external dep: unicode-width crate)
    │
    ▼
frame.rs            (uses measure::pad_right)
    │
    ▼
renderers/*         (use measure::pad_right, measure::truncate, measure::display_width)
    │
    ▼
mermaid/*           (uses schema types — produces Diagram variants)
    │
    ▼
render.rs           (dispatches to renderers + mermaid, applies frame)
    │
    ▼
lib.rs              (public API: re-exports render + types)
```

## What drawz-cli does

Thin wrapper:

```rust
fn main() {
    let input: DiagramInput = read_stdin_json();
    let width = cli_width.unwrap_or(input.width);
    let result = drawz_core::render(&input.diagram, width);

    if let Some(output) = &result.output {
        println!("{output}");
    }
    for err in &result.errors {
        eprintln!("error: {err}");
    }
    for warn in &result.warnings {
        eprintln!("warning: {warn}");
    }

    std::process::exit(if result.errors.is_empty() { 0 } else { 1 });
}
```

## What drawz-mcp will do (Phase 4)

Thin wrapper around the same `drawz_core::render()` — serializes `RenderResult` as JSON-RPC response. No new logic, just transport.
