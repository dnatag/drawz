# ADR-002: Response Contract and Graceful Error Handling

**Status:** Accepted  
**Date:** 2026-06-14

## Context

When a diagram cannot render perfectly (content too wide, too many nodes, impossible layout), drawz needs a clear protocol for communicating this to the calling agent. The agent must know: did it work? If not, what went wrong? How to fix it?

## Decision

### Structured response, always

drawz returns a structured response for every render call:

```json
{
  "output": "<rendered diagram or null>",
  "fit": true | false,
  "errors": [],
  "warnings": []
}
```

`errors` — problems that prevented rendering or indicate invalid input. If `errors` is non-empty, `output` is null.

`warnings` — rendering compromises (truncation, layout changes). Output was produced but is degraded.

### Four outcomes

**1. Perfect fit** — no errors, no warnings:

```json
{
  "output": "┌───────────────────┐\n│ A ──► B ──► C     │\n└───────────────────┘",
  "fit": true,
  "errors": [],
  "warnings": []
}
```

**2. Degraded fit** — no errors, has warnings:

```json
{
  "output": "┌──────────────────────────────────┐\n│ Configur… │ Descript… │ Default… │\n...",
  "fit": false,
  "errors": [],
  "warnings": [
    "2 column headers truncated to 8 chars",
    "suggestion: reduce to 4 columns or set width to 120"
  ]
}
```

**3. Cannot render** — rendering error (valid input, but impossible at this width):

```json
{
  "output": null,
  "fit": false,
  "errors": ["cannot render 20-column table at width 80 (minimum width needed: 160)"],
  "warnings": ["suggestion: split into multiple tables or increase width"]
}
```

**4. Invalid input** — schema/validation error:

```json
{
  "output": null,
  "fit": false,
  "errors": ["missing required field 'edges' for dag diagram"],
  "warnings": ["hint: {\"type\": \"dag\", \"edges\": [{\"from\": \"A\", \"to\": \"B\"}]}"]
}
```

### Warning and error format

Each entry is a plain string. Suggestions are prefixed with `suggestion:` and hints with `hint:` for programmatic parsing.

**Error examples:**
- `"missing required field 'edges' for dag diagram"`
- `"cannot render 20-column table at width 80 (minimum width needed: 160)"`
- `"mermaid type 'gantt' is not supported in drawz v0.1"`

**Warning examples:**
- `"label 'AuthenticationMiddleware' truncated to 'Authentic…'"`
- `"switched from horizontal to vertical layout (nodes exceed width)"`
- `"5 edge crossings could not be resolved; simplified routing"`
- `"suggestion: break into 2 subgraphs or reduce to 8 nodes"`
- `"hint: {\"type\": \"dag\", \"edges\": [{\"from\": \"A\", \"to\": \"B\"}]}"`

### Design rationale

1. **Always render when possible.** The human should see *something* immediately — even if degraded. Agents can iterate, but humans shouldn't wait for agent retries.

2. **Never silently degrade.** Every compromise is reported in `warnings`. The agent can decide: show as-is, or adjust and retry.

3. **Separate errors from warnings.** Agents check `errors.len() > 0` to know if something failed. They check `warnings` to know if output quality could be improved. No string-prefix parsing needed for severity.

4. **Suggestions teach agents.** By including actionable suggestions, drawz helps agents learn what works.

5. **`fit: false` is not always an error.** Degraded output with warnings may still be perfectly usable.

## Consequences

- Agents check `errors` array for failures — no string parsing for severity
- Agents check `warnings` for quality feedback — actionable but non-blocking
- Humans always see best-effort output without waiting for retries
- No silent degradation — every compromise is visible
- Suggestions create a feedback loop that improves agent behavior
- Single response shape regardless of outcome — simple to parse
- CLI mode prints `output` to stdout, `errors` + `warnings` to stderr
