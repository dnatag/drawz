# Product Requirements Document — drawz

**Status:** Draft v2  
**Target:** v0.1.0

---

## 1. Problem Statement

AI agents communicate complex architecture, data flows, and state machines via prose or manually-crafted ASCII art. This fails because:

- **Misalignment breaks comprehension.** When `│` doesn't land under `┬`, or `──►` shifts columns, the diagram becomes noise. Agents cannot reliably compute Unicode character widths across terminals and fonts.
- **Agents waste tokens describing structure** that a diagram shows instantly. A wall of text explaining "A connects to B which connects to C" is slower to parse than a visual.
- **No standard tool exists** for agents to produce terminal-ready visuals with guaranteed alignment.
- **Existing tools have learning curves.** Mermaid/D2 require DSL syntax — but agents already know Mermaid from training data. drawz accepts both structured JSON (zero learning) and raw Mermaid (zero migration).

## 2. Solution

drawz is the **rendering guarantee layer** between AI agents and terminal display. Agents describe *what* to show (content + relationships). drawz guarantees *how* it looks (layout + alignment + framing).

```
Agent's job:  describe WHAT to show (structured JSON)
drawz's job:  guarantee HOW it looks (perfect alignment, always)
```

Comparable to Mermaid for markdown — but for terminals, and with zero learning curve for agents (JSON, not DSL).

### Why Terminal, Not HTML

Agents need a native output medium for structured visuals. HTML requires a browser, a localhost server, and a context switch. Terminal text requires nothing — it's already where the developer is looking, already what the agent is producing, already what tool calls return.

```
Agent → JSON → drawz → Terminal → Human  (zero context switch)
Agent → HTML → Server → Browser → Human  (rich, but friction)
```

### Design Principle: AI-Native

The #1 priority is minimal integration effort for AI agents:

- **Zero learning curve** — structured JSON with named fields, no syntax to memorize
- **Rendering guarantee** — output is always perfectly aligned regardless of Unicode complexity
- **Progressive structure** — simple flows use `steps`, complex topology uses `nodes` + `edges`
- **Fault tolerance** — bad input gets a useful error message with a hint, not a crash

## 3. Integration Surfaces

```
┌──────────────────────────────────────────────────────────────────┐
│ Integration Layers                                               │
├──────────────────────────────────────────────────────────────────┤
│                                                                  │
│ PRIMARY:   MCP tool (single: render_diagram)                     │
│            Best correctness, structured response, no escaping.   │
│                                                                  │
│ SECONDARY: CLI + Skill                                           │
│            Skill teaches agent heredoc invocation pattern.        │
│            For environments with shell but no MCP.               │
│                                                                  │
│ TERTIARY:  AGENTS.md (awareness, not execution)                  │
│            Tells agents drawz exists and when to reach for it.   │
│                                                                  │
│ LIBRARY:   use drawz_core::render(...)                           │
│            For Rust programs embedding drawz directly.           │
│                                                                  │
└──────────────────────────────────────────────────────────────────┘
```

### Why single MCP tool, not one per type

Each registered MCP tool costs ~250 tokens of context per turn (name + description + schema). 9 tools = ~2250 tokens overhead every turn, even when the agent doesn't use drawz. A single `render_diagram` tool with a `type` discriminator costs ~400 tokens and provides the same functionality. The JSON schema already handles type dispatch via the `type` field.

### Why MCP over raw CLI

The #1 failure mode of CLI integration is **JSON escaping in shell strings**. Agents frequently produce:
```bash
echo "{"type":"flow"}" | drawz   # BROKEN — unescaped quotes
```

MCP eliminates this entirely — the agent passes structured parameters, no shell escaping needed. The structured response (`{output, fit, errors, warnings}`) also enables programmatic retry logic.

### CLI via Skill (secondary path)

For agents with shell access but no MCP, a skill teaches the heredoc pattern:
```bash
drawz <<'EOF'
{"type":"flow","steps":["Build","Test","Deploy"]}
EOF
```

Heredoc avoids all escaping issues. The skill also embeds the diagram type mapping table and "when to use drawz" decision logic.

## 4. Supported Diagram Types

| Type | Input | Use Case | Complexity |
|------|-------|----------|------------|
| `freeform` | lines of text | Any — agent sends pre-structured text, gets framing | Trivial |
| `mermaid` | Mermaid DSL code | Any — agents already know Mermaid, drawz renders to terminal | Low (agent) / Medium (impl) |
| `table` | headers + rows | Comparisons, option matrices | Low |
| `tree` | recursive nodes | File structures, hierarchies | Low |
| `flow` | nodes + edges | Data pipelines, request flows | Medium |
| `state` | states + transitions | State machines, lifecycles | Medium |
| `sequence` | actors + messages | API interactions, protocols | Medium |
| `dag` | nodes + edges | Task dependencies, build graphs | High |

### When agents reach for drawz

```
┌──────────────────────────────────────────────────────────────────┐
│ Agent Decision: inline ASCII vs drawz                            │
│                                                                  │
│ Simple (3-4 items, linear)?  → agent writes inline, no tool call │
│ Complex (branching, 5+ nodes, alignment-sensitive)?  → use drawz │
│ Table with many columns?     → use drawz (alignment guarantee)   │
│ Deep tree / nested structure? → use drawz (indent math is hard)  │
│ Already have Mermaid?        → send to drawz, get terminal output│
└──────────────────────────────────────────────────────────────────┘
```

drawz earns its tool call when alignment matters and complexity makes hand-crafted ASCII unreliable.

## 5. Input Format

Every diagram type supports a **minimal form** (fewest tokens for the common case) and a **full form** (complete control for complex cases). The dual-form rule: the minimal form should be so obvious that an agent gets it right on the first call without reading docs. The full form adds explicit IDs, titles, and structural detail when needed.

### Common parameters (all types)

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `type` | string | (required) | Diagram type discriminator |
| `title` | string | none | Optional title rendered inside frame |
| `width` | integer | 80 | Maximum output width in characters |

Width can be set in the JSON input, as a CLI flag (`--width 120`), or as an MCP tool parameter. Precedence: CLI/MCP flag > JSON field > default (80).

### freeform

The agent already knows how to write ASCII diagrams — but it can't guarantee alignment. `freeform` takes agent-authored text and fixes Unicode character width issues, ensures consistent column alignment, and wraps it in a properly-sized frame. The agent provides the content; drawz guarantees it displays correctly.

**Minimal** — single string with newlines:

```json
{"type":"freeform","content":"A ──► B ──► C\n      │\n      ▼\n      D"}
```

**Full** — array of lines with title:

```json
{"type":"freeform","title":"My Diagram","lines":["A ──► B ──► C","      │","      ▼","      D"]}
```

Note: `content` (string) and `lines` (array) are interchangeable. drawz measures actual Unicode display widths (not byte lengths) to produce correct framing and alignment.

### mermaid

Agents already produce Mermaid fluently. drawz accepts raw Mermaid code and renders it as aligned terminal text — no browser or SVG needed.

```json
{"type":"mermaid","code":"graph LR\n  A[Request] --> B[Auth]\n  B --> C[Handler]\n  B --> D[Reject]"}
```

```json
{"type":"mermaid","code":"sequenceDiagram\n  Client->>Server: POST /login\n  Server->>DB: SELECT user\n  DB-->>Server: row\n  Server-->>Client: 200 + token"}
```

Note: Any valid Mermaid syntax works. drawz parses and re-renders as terminal-aligned text. This is the zero-learning-curve path — agents already know Mermaid from producing it in markdown.

### flow

**Minimal** — just labels in order, linear chain implied:

```json
{"type":"flow","steps":["Request","Auth","Handler","Response"]}
```

**Nested sub-flows** — child pipelines inline:

```json
{"type":"flow","steps":["Build","Test",{"label":"Deploy","steps":["Stage","Canary","Promote"]}]}
```

**Full** — explicit nodes and edges (for branching, cycles, any topology):

```json
{"type":"flow","nodes":[{"label":"Build"},{"label":"Test"},{"label":"Stage"},{"label":"Rollback"}],"edges":[{"from":"Build","to":"Test"},{"from":"Test","to":"Stage","label":"pass"},{"from":"Test","to":"Rollback","label":"fail"}]}
```

Notes: `steps` is for linear/nested flows. For branching or complex topology, use `nodes` + `edges` with labels as identifiers (no IDs needed).

**flow vs dag:** Both accept nodes + edges. The difference is rendering intent:
- `flow` implies a **directional pipeline** (left→right or top→down). Nodes are ordered along a main axis. Best for: request flows, CI/CD, data pipelines.
- `dag` implies a **dependency graph** with potentially wide fan-in/fan-out. Layout optimizes for crossing minimization. Best for: build systems, task scheduling, prerequisite chains.

If unsure, use `flow` for linear-ish things, `dag` for diamond/converging patterns.

### table

**Minimal** — headers and rows:

```json
{"type":"table","headers":["A","B"],"rows":[["1","2"]]}
```

**Full** — same with title:

```json
{"type":"table","title":"Comparison","headers":["Feature","drawz","Mermaid"],"rows":[["AI-native","Yes","No"]]}
```

Note: Table is already minimal. Title is the only optional addition.

### tree

**Minimal** — indentation encodes hierarchy (2-space indent per level):

```json
{"type":"tree","indent":"src/\n  main.rs\n  lib.rs\n    schema.rs"}
```

**Full** — explicit recursive structure:

```json
{"type":"tree","title":"Project","root":{"label":"src/","children":[{"label":"main.rs"},{"label":"lib.rs","children":[{"label":"schema.rs"}]}]}}
```

Note: `indent` parses whitespace to infer parent/child. `root` gives explicit structure.

### sequence

**Minimal** — actors and messages:

```json
{"type":"sequence","actors":["Client","Server"],"messages":[{"from":"Client","to":"Server","label":"GET /"}]}
```

**Full** — same + optional title:

```json
{"type":"sequence","title":"Auth Flow","actors":["Client","Server"],"messages":[{"from":"Client","to":"Server","label":"GET /"}]}
```

Note: Sequence is already fairly minimal. No shorthand needed.

### state

**Minimal** — just transitions (states inferred):

```json
{"type":"state","transitions":[{"from":"Idle","to":"Connecting","label":"connect()"},{"from":"Connecting","to":"Connected","label":"ack"}]}
```

**Full** — explicit states + transitions:

```json
{"type":"state","title":"Lifecycle","states":[{"label":"Idle"},{"label":"Connecting"},{"label":"Connected"}],"transitions":[{"from":"Idle","to":"Connecting","label":"connect()"},{"from":"Connecting","to":"Connected","label":"ack"}]}
```

Note: States are inferred from transitions if not declared. Declare states explicitly only when you need isolated states (no transitions) or custom IDs.

### dag

**Minimal** — just edges (nodes inferred):

```json
{"type":"dag","edges":[{"from":"Compile","to":"Test"},{"from":"Test","to":"Deploy"}]}
```

**Full** — explicit nodes + edges:

```json
{"type":"dag","title":"Build Graph","nodes":[{"label":"Compile"},{"label":"Test"},{"label":"Deploy"}],"edges":[{"from":"Compile","to":"Test"},{"from":"Test","to":"Deploy"}]}
```

Note: Nodes are inferred from edges if not declared. Declare nodes explicitly only for isolated nodes (no edges).

## 6. Output Requirements

- Correct column alignment regardless of Unicode character widths
- Pure text output — works in any terminal, no image protocol needed
- Deterministic: same input + same width = same output, always
- Width-bounded: output never exceeds the configured width (default: 80)

### Framing rules per diagram type

| Type | Framing | Why |
|------|---------|-----|
| `freeform` | Outer box (`┌─┐│└─┘`) with optional title | Visually separates diagram from surrounding text |
| `table` | Cell borders ARE the frame (no outer box needed) | Table grid is self-contained |
| `tree` | No frame — indented text is self-evident | Adding a box around `tree` output adds noise |
| `flow` | Outer box with title | Groups the flow as a visual unit |
| `state` | Outer box with title | Groups the state machine as a unit |
| `sequence` | Outer box with title | Bounds the interaction diagram |
| `dag` | Outer box with title | Groups the dependency graph |
| `mermaid` | Inherits from converted type | Mermaid→Flow gets flow framing, etc. |

"Framed" means the output is a **self-contained visual unit** that a human can distinguish from surrounding text. For most types that's a Unicode box border. For table and tree, the structure itself provides visual boundaries.

### Expected output examples

**freeform** — input: `{"type":"freeform","title":"Flow","content":"A ──► B ──► C"}`
```
┌─────────────────┐
│ Flow            │
│                 │
│ A ──► B ──► C   │
└─────────────────┘
```

**table** — input: `{"type":"table","headers":["Feature","drawz","Mermaid"],"rows":[["AI-native","Yes","No"],["Terminal","Yes","No"]]}`
```
Feature   │ drawz │ Mermaid
──────────┼───────┼────────
AI-native │ Yes   │ No
Terminal  │ Yes   │ No
```

**tree** — input: `{"type":"tree","indent":"src/\n  main.rs\n  lib.rs\n    schema.rs"}`
```
src/
├── main.rs
├── lib.rs
│   └── schema.rs
```

**flow (linear)** — input: `{"type":"flow","steps":["Build","Test","Deploy"]}`
```
┌─────────────────────────────────┐
│ Build ──► Test ──► Deploy       │
└─────────────────────────────────┘
```

**flow (branching)** — input with nodes+edges:
```
┌──────────────────────────────────┐
│ Build ──► Test ─┬──► Stage       │
│                 └──► Rollback    │
└──────────────────────────────────┘
```

**sequence** — input: `{"type":"sequence","actors":["Client","Server","DB"],...}`
```
┌──────────────────────────────────────────────┐
│ Client         Server         DB             │
│ │               │              │             │
│ │──POST /login─►│              │             │
│ │               │──SELECT ────►│             │
│ │               │◄── row ──────│             │
│ │◄── 200 ───────│              │             │
└──────────────────────────────────────────────┘
```

**state** — input with transitions:
```
┌────────────────────────────────────────────┐
│ [Idle] ──connect()──► [Connecting]         │
│                          │                 │
│                         ack                │
│                          ▼                 │
│                       [Connected]          │
└────────────────────────────────────────────┘
```

**dag** — input with edges:
```
┌──────────────────────────┐
│ Compile                  │
│    │                     │
│    ▼                     │
│  Test                    │
│    │                     │
│    ▼                     │
│ Deploy                   │
└──────────────────────────┘
```

These are target renderings. Exact box widths and connector styles may vary during implementation, but alignment and framing rules must hold.

## 7. MCP Design

### Transport

stdio (standard for local MCP servers). Agent spawns `drawz mcp` and communicates over stdin/stdout using JSON-RPC.

### Single Tool: render_diagram

One tool with the `type` field as discriminator. Keeps context overhead minimal (~400 tokens).

```json
{
  "name": "render_diagram",
  "description": "Render a structured diagram as perfectly-aligned ASCII/Unicode art for terminal display. Accepts JSON with a 'type' field: freeform, mermaid, flow, table, tree, sequence, state, or dag. Returns rendered output with fit/error metadata. Example: {\"type\":\"flow\",\"steps\":[\"Build\",\"Test\",\"Deploy\"]}",
  "inputSchema": {
    "type": "object",
    "properties": {
      "type": {
        "type": "string",
        "enum": ["freeform", "mermaid", "flow", "table", "tree", "sequence", "state", "dag"]
      },
      "width": { "type": "integer", "default": 80 },
      "title": { "type": "string" }
    },
    "required": ["type"],
    "additionalProperties": true
  }
}
```

Note: `additionalProperties: true` allows per-type fields (steps, nodes, edges, headers, rows, code, etc.) to pass through. The full per-type schema is available via the `introspect_drawz` tool.

### Introspect Tool

A second tool for discoverability:

```json
{
  "name": "introspect_drawz",
  "description": "List supported diagram types, show examples, and return the diagram type mapping table."
}
```

Total MCP context cost: 2 tools, ~500 tokens. Acceptable for a utility that delivers visual communication.

### Tool Descriptions Include Examples

The `render_diagram` description contains a working minimal example. Agents learn from examples faster than from schemas.

### Response Contract

All calls return the same structured response (see ADR-002):

```json
{
  "output": "<rendered diagram or null>",
  "fit": true,
  "errors": [],
  "warnings": []
}
```

On invalid input:

```json
{
  "output": null,
  "fit": false,
  "errors": ["missing required field 'edges' for dag diagram"],
  "warnings": ["hint: {\"type\": \"dag\", \"edges\": [{\"from\": \"A\", \"to\": \"B\"}]}"]
}
```

`errors` = something failed. `warnings` = output is degraded or includes suggestions.

## 8. Discoverability

### CLI

- `drawz --schema` — dumps JSON schema for all diagram types
- `drawz --example flow` — prints an example input/output pair for a given type
- `drawz --types` — lists supported diagram types with one-line descriptions

### MCP

- `introspect` tool — returns capabilities, supported types, and examples
- Tool descriptions embed examples directly

### Diagram Type Mapping

Agents think in terms of what they want to *communicate*, not drawz type names. The `introspect` tool and `--types` output include this mapping so agents pick the right type without guessing:

| What you want to show | Use type | Why |
|----------------------|----------|-----|
| Linear pipeline, request flow | `flow` (steps) | Ordered sequence of stages |
| Branching pipeline, if/else paths | `flow` (nodes+edges) | Non-linear topology |
| Nested pipeline, sub-processes | `flow` (nested steps) | Hierarchy within a flow |
| File/directory structure | `tree` | Natural hierarchy |
| Class hierarchy, org chart | `tree` | Parent-child relationships |
| State machine, lifecycle | `state` | States + transitions |
| API interaction, protocol | `sequence` | Ordered messages between actors |
| Task dependencies, build graph | `dag` | Fan-in/fan-out, parallel paths |
| ER diagram, entity relationships | `dag` | Labeled connections between entities |
| Network topology | `dag` | Arbitrary connections |
| Comparison, pros/cons | `table` | Structured columns |
| Config options, feature matrix | `table` | Key-value or multi-column |
| Timeline, project phases | `table` or `freeform` | Rows as time periods |
| Decision tree | `tree` or `flow` | Tree if binary, flow if merging |
| Mind map, concept cluster | `tree` | Radiating from center = root + children |
| Any pre-formatted ASCII art | `freeform` | Already structured, just fix alignment |
| Any Mermaid diagram | `mermaid` | Agent already has Mermaid code |

This mapping is included in the `introspect` tool response and in each tool's description.

## 9. Architecture

```
drawz/
├── Cargo.toml                  # Workspace root
└── crates/
    ├── drawz-core/             # Schema types, layout engine, renderer
    │   ├── src/schema.rs       # Input types (Diagram enum, Node, Edge, etc.)
    │   ├── src/render.rs       # Rendering logic per diagram type
    │   ├── src/result.rs       # RenderResult response type
    │   └── src/layout.rs       # (future) Graph layout algorithms
    ├── drawz-cli/              # Binary: stdin JSON → stdout diagram
    │   └── src/main.rs
    └── drawz-mcp/              # (future) MCP server binary
        └── src/main.rs
```

### Public API

```rust
/// The structured response from every render call.
pub struct RenderResult {
    pub output: Option<String>,
    pub fit: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// Main entry point.
pub fn render(diagram: &Diagram, width: u16) -> RenderResult;
```

CLI prints `output` to stdout, `errors` + `warnings` to stderr. MCP returns the full struct as JSON.

## 10. Non-Goals (v0.1)

- No interactive editing
- No SVG/image output
- No color/ANSI styling (future consideration)
- No graph layout optimization (simple top-down/left-right for v0.1)
- No natural-language-to-diagram (future: `description` field parsed by LLM layer)

### Versioning & Stability

- The response contract (`{output, fit, errors, warnings}`) is stable from v0.1 — new fields may be added but existing fields won't change shape.
- New diagram types may be added in minor versions (e.g., `timeline` in v0.2). Existing types won't have breaking schema changes.
- MCP tool names are permanent once registered. New tools are additive.
- The CLI supports `drawz --version` for agents to check compatibility.

## 11. Success Criteria

- An agent can call `render_diagram` with any type and get a correctly laid-out diagram
- Output aligns perfectly in any monospace terminal
- MCP server exposes `render_diagram` + `introspect_drawz` with example-bearing descriptions
- Structured response includes fit/errors/warnings for agent self-correction
- CLI supports `drawz --schema`, `drawz --example <type>`, and `drawz --types`
- Heredoc CLI pattern works without escaping issues
- Same JSON format works via MCP, CLI, and Rust library

## 12. Implementation Priority

```
┌────────────────────────────────────────────────────────────────┐
│ Build Order                                                    │
│                                                                │
│ Phase 1: freeform + table rendering (prove the engine works)   │
│ Phase 2: tree + flow + state                                   │
│ Phase 3: sequence + dag + mermaid parsing                      │
│ Phase 4: MCP server + introspect (expose via standard protocol)│
└────────────────────────────────────────────────────────────────┘
```

**Why MCP is built last but called "primary":** MCP is the primary *integration surface* — it's how agents will discover and call drawz in production. But it's a thin transport layer over the rendering engine. Building rendering first means MCP ships with working tools, not stubs. During Phases 1-3, the CLI serves as the test harness. Phase 4 wraps the proven engine in MCP with minimal new code.
