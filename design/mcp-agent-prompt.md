# drawz MCP — Agent Integration Prompt

Add this to your agent's system prompt or MCP tool instructions so it knows when and how to use drawz.

---

## Prompt

```
You have access to the `render_diagram` MCP tool (via the drawz server). Use it whenever you need to display structured visual information to the user — tables, trees, flows, sequences, state machines, DAGs, or any ASCII diagram.

### When to use render_diagram

- Comparing options or features → type: "table"
- Showing file/directory structures or hierarchies → type: "tree"
- Explaining pipelines, request flows, or steps → type: "flow"
- Describing API interactions or protocols → type: "sequence"
- Illustrating state machines or lifecycles → type: "state"
- Showing dependency graphs or build order → type: "dag"
- Displaying pre-formatted text with guaranteed alignment → type: "freeform"
- Agent already has Mermaid syntax → type: "mermaid"

### How to call render_diagram

Pass diagram fields directly as the tool arguments object. Do NOT wrap in a string or nest under an "input" key.

Examples:

Table:
{
  "type": "table",
  "headers": ["Feature", "Status"],
  "rows": [["Alignment", "✓"], ["Unicode", "✓"]]
}

Tree:
{
  "type": "tree",
  "indent": "src/\n  main.rs\n  lib.rs\n  utils/"
}

Flow (linear):
{
  "type": "flow",
  "steps": ["Parse", "Validate", "Execute", "Respond"]
}

Flow (nested sub-pipelines):
{
  "type": "flow",
  "steps": ["Init", {"label": "Build", "steps": ["Compile", "Link"]}, "Deploy"]
}

Sequence:
{
  "type": "sequence",
  "actors": ["Client", "Server", "DB"],
  "messages": [
    {"from": "Client", "to": "Server", "label": "GET /users"},
    {"from": "Server", "to": "DB", "label": "SELECT *"},
    {"from": "DB", "to": "Server", "label": "rows"},
    {"from": "Server", "to": "Client", "label": "200 OK"}
  ]
}

State:
{
  "type": "state",
  "transitions": [
    {"from": "[*]", "to": "Idle"},
    {"from": "Idle", "to": "Running", "label": "start"},
    {"from": "Running", "to": "Done", "label": "finish"}
  ]
}

DAG:
{
  "type": "dag",
  "edges": [
    {"from": "Parse", "to": "Typecheck"},
    {"from": "Parse", "to": "Lint"},
    {"from": "Typecheck", "to": "Codegen"},
    {"from": "Lint", "to": "Codegen"}
  ]
}

Mermaid:
{
  "type": "mermaid",
  "code": "graph LR; A-->B-->C"
}

Freeform:
{
  "type": "freeform",
  "content": "Header line\n  indented detail\n  another line"
}

### Optional fields (all types)

- "width": integer (default 80) — max output width in display columns
- "title": string — shown in the frame header if the diagram type is framed

### Reading the response

The tool returns JSON with:
- output: the rendered diagram string (newline-separated lines, each padded to width)
- fit: true if content fits without truncation
- errors: array of error messages (output will be null if non-empty)
- warnings: array of warnings (e.g., content was truncated to fit)

Display the "output" field directly in a code block. If fit is false, check warnings for context.

### Rules

1. NEVER hand-draw ASCII tables, trees, or diagrams. Always use render_diagram.
2. Choose the most specific type. A comparison → table. A file listing → tree. Don't use freeform as a fallback when a structured type fits.
3. Pass data as structured fields, not pre-formatted strings. Let drawz handle alignment.
4. If you need to show the user a quick visual, call render_diagram. It's fast.
```
