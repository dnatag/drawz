# drawz 1.x Roadmap

| Priority | Item | Type | Version | Effort |
|----------|------|------|---------|--------|
| P0 | DAG fan-in/fan-out rendering | Bug fix | 1.0.2 | M |
| P1 | Mermaid subgraph → grouped boxes | Feature | 1.1.0 | L |
| P2 | State 2D layout (branch targets) | Enhancement | 1.2.0 | L |
| P3 | Component diagram type | Feature | 1.3.0 | XL |

## P0 — DAG fan-in/fan-out rendering (v1.0.2)

**Problem:** When 3+ nodes point to one node (fan-in) or one node fans out to 3+, the renderer stacks them vertically instead of placing them in a row with converging/diverging arrows.

**Desired:** Nodes at the same depth (same rank) should appear on one row with arrows converging down to their shared target.

**Approach:**
- The Sugiyama layering already groups nodes by rank — the issue is in `render_level` which renders parallel nodes on one row, but the layer assignment may be wrong when fan-in patterns exist
- Fix: ensure nodes with the same set of successors (or predecessors) get assigned to the same layer
- Draw converging arrow lines (╲ │ ╱) between fan-in layers

**Scope:** `crates/drawz-core/src/renderers/dag.rs` — layer assignment + arrow rendering between layers

## P1 — Mermaid subgraph → grouped boxes (v1.1.0)

**Problem:** Mermaid `subgraph` directives are currently skipped. Agents writing `flowchart` with subgraphs get the nodes but lose the grouping.

**Desired:** Subgraphs render as labeled framed boxes containing their internal nodes + edges. Inter-group edges drawn between frames.

**Approach:**
- Parse `subgraph <name>` / `end` blocks in `mermaid/flowchart.rs` (currently skipped)
- Group nodes by subgraph membership
- Render as framed boxes containing their internal nodes + edges
- Inter-group edges drawn between frames
- Falls back to flat rendering if layout can't fit

**Scope:** `mermaid/flowchart.rs` (parser), new `renderers/component.rs` or extend `dag.rs` with grouping

## P2 — State 2D layout (v1.2.0)

**Problem:** State renderer is strictly linear (top-to-bottom). States with multiple entry points (like "Failed") end up in the middle of a linear chain instead of off to the side.

**Desired:** Main path renders vertically, branch targets render horizontally to the side with horizontal arrows.

**Approach:**
- Detect the "main path" (longest chain from initial state)
- Branch targets (states reachable only as alternatives) render to the right
- Reuse DAG's Sugiyama layer assignment to position states in 2D
- Horizontal arrows for branches, vertical for main flow

**Scope:** `renderers/state.rs` — replace linear chain with 2D grid layout

## P3 — Component diagram type (v1.3.0)

**Problem:** No native type for architecture diagrams (boxes-within-boxes with labeled ports). This is the #1 case where agents hand-draw.

**Schema:**
```json
{
  "type": "component",
  "groups": [
    { "label": "Parent Process", "nodes": ["Scheduler", "DAG Engine"] },
    { "label": "Child Process", "nodes": ["Sandbox", "Runtime"] }
  ],
  "connections": [
    { "from": "Scheduler", "to": "Sandbox", "label": "spawn" },
    { "from": "DAG Engine", "to": "Runtime", "label": "pipe" }
  ]
}
```

**Approach:**
- Each group renders as a framed block containing its nodes horizontally
- Groups lay out top-to-bottom or left-to-right based on connections
- Inter-group connections render as labeled arrows between frames
- Builds on P1's grouped rendering infrastructure

**Scope:** New `schema::ComponentDiagram`, new `renderers/component.rs`, schema extension

## Dependencies

- P1's subgraph rendering infrastructure feeds into P3
- P0 and P2 are independent
