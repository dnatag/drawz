# ADR-001: Terminal Rendering Constraints

**Status:** Accepted  
**Date:** 2026-06-14

## Context

drawz renders structured diagrams as ASCII/Unicode text for terminal display. Terminals impose hard constraints that don't exist in SVG/browser rendering. We need explicit rules for how drawz operates within these limits.

## Constraints

| Constraint | Terminal | SVG/Browser |
|------------|----------|-------------|
| Canvas | Fixed-width character grid | Infinite 2D |
| Width | ~80-200 columns | Unlimited |
| Height | Vertical scroll only | Fit/zoom |
| Positioning | Integer character cells | Sub-pixel |
| Connectors | `─│┌┐└┘├┤┬┴┼▼►◄▲` | Bezier curves |
| Line crossings | Cannot overlay two chars in one cell | Auto-routed |
| Node shapes | Boxes (`┌─┐│└─┘`) | Circles, diamonds, etc. |

## Decisions

### 1. Horizontal overflow is NEVER acceptable

Vertical scroll is fine — terminals handle it naturally. Horizontal overflow destroys alignment and readability. drawz MUST fit all output within the configured width.

**Width parameter:** Default 80. Agent or caller may specify a different width.

### 2. drawz controls layout, not the agent

The agent describes *what* to show. drawz decides *how* to lay it out based on available width. If 5 nodes fit horizontally, render horizontally. If they don't, switch to vertical.

The agent does not specify layout direction. drawz chooses automatically based on content and available width.

### 3. Labels truncate with visible `…`

Long labels are truncated to fit available space. Truncation is always visible via `…`, never silent.

### 4. Node reordering to minimize crossings

For DAGs and flows, drawz may reorder nodes to reduce line crossings. The agent specifies relationships, not positions. Output node order may differ from input order — topology is preserved.

### 5. Vertical layout is the default for complex graphs

When a graph has crossing edges that can't be eliminated by reordering, drawz prefers vertical layout (nodes stacked top-to-bottom, edges run downward). This eliminates most crossing problems.

### 6. Degradation strategies by diagram type

**Table:**
1. Auto-size columns to content
2. If too wide: shrink widest columns proportionally
3. If still too wide: truncate cells with `…` (minimum 6 chars per column)
4. If still too wide: cannot render

**Flow / DAG:**
1. Try horizontal layout
2. If too wide: switch to vertical layout
3. If single node label exceeds width: truncate with `…`
4. Nested sub-flows collapse to a single node if space is tight

**Sequence:**
1. Compact actor names to fit width
2. If actors + arrows don't fit: truncate actor names
3. If actors cannot fit even truncated (each actor needs ~12 chars minimum: name + padding + arrow): cannot render. Max actors ≈ width / 12

**Tree:**
1. Render as indented text (naturally vertical)
2. If label + indent depth exceeds width: truncate labels
3. Trees rarely fail — depth grows vertically

**Mermaid:**
- Parse → convert to internal type → apply that type's strategy

**Freeform:**
- Wrap lines at word boundary if they exceed width

## Consequences

- Output always fits within configured width — safe to pipe, embed, display anywhere
- Agents never think about terminal geometry — they describe structure, drawz handles layout
- Some diagrams may render with truncated labels or simplified layout
- Very complex diagrams may be impossible to render at narrow widths
