# ADR-003: Mermaid Parsing Scope

**Status:** Accepted  
**Date:** 2026-06-14

## Context

drawz accepts `{"type":"mermaid","code":"..."}` as input. Mermaid is a large DSL with many diagram types (flowchart, sequence, state, class, ER, Gantt, pie, quadrant, mindmap, timeline, etc.). We need to decide what subset to support and how to implement parsing.

## Options Considered

### A. Full Mermaid parser (build from scratch)

- Covers all Mermaid diagram types
- Massive implementation effort (Mermaid's parser is ~20k LOC of JS)
- Perpetual catch-up with Mermaid's evolving syntax

### B. Shell out to Mermaid CLI + parse SVG output

- Uses official parser (always correct)
- Requires Node.js runtime dependency
- Adds ~200ms latency per render
- SVG→ASCII conversion is lossy and complex

### C. Support a subset of Mermaid, parse ourselves

- Cover the types agents actually use: flowchart, sequence, state
- Small, focused parser (~2-3k LOC Rust)
- Unsupported Mermaid types return a clear error with suggestion

### D. Parse only the graph structure, not the layout directives

- Extract nodes + edges from Mermaid syntax
- Convert to internal types (Flow, Sequence, State)
- Ignore Mermaid-specific styling/layout hints
- Let drawz handle layout with its own engine

## Decision

**Option D** — Parse Mermaid graph structure, convert to internal types, render with drawz's engine.

### Supported Mermaid syntax (v0.1)

| Mermaid Type | Converts To | Example |
|-------------|-------------|---------|
| `graph LR/TD` | Flow | `graph LR; A-->B-->C` |
| `sequenceDiagram` | Sequence | `sequenceDiagram; A->>B: msg` |
| `stateDiagram-v2` | State | `stateDiagram-v2; [*]-->Idle` |

### Unsupported (v0.1)

- `classDiagram` — no equivalent internal type yet
- `erDiagram` — can map to DAG in future
- `gantt` — timeline type needed (v0.2)
- `pie`, `quadrant`, `mindmap` — low priority

### Error for unsupported types

```json
{
  "output": null,
  "fit": false,
  "errors": ["mermaid type 'gantt' is not supported in drawz v0.1"],
  "warnings": ["suggestion: use type 'table' with timeline data, or wait for v0.2 timeline support"]
}
```

## Consequences

- Mermaid support is immediately useful for the 3 most common diagram types agents produce
- No external runtime dependency (pure Rust parser)
- Agents that already write Mermaid can switch to drawz with zero code change
- Unsupported types get actionable errors, not silent failures
- The parser is small and maintainable (~focused subset, not full grammar)
- Future Mermaid types can be added incrementally as internal types are built
