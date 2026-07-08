---
name: drawz-shell
description: >
  Render diagrams via drawz CLI instead of MCP for instant display.
  Use when user asks for a diagram, visualization, table, tree, flow,
  sequence, state machine, or DAG in a terminal context where shell
  output is directly visible (not collapsed behind a UI toggle).
allowed-tools: Bash
---

# drawz-shell

Render diagrams instantly via the drawz CLI. Output appears in shell results
with zero LLM streaming delay — the diagram never passes through token generation.

## Why shell over MCP

MCP `render_diagram` returns the rendered text to the LLM, which must then stream it
token-by-token to the user. For a 40-line diagram, that's 2–3 seconds of pure passthrough.

Shell rendering bypasses this entirely: the diagram appears in the command output instantly.

**Use shell** (this skill): terminal contexts where shell output is visible inline.
**Use MCP**: GUI contexts where tool results render in a dedicated panel (Claude Desktop, etc.)
and shell output would be collapsed or hidden.

## Invocation Methods

### Method 1: Flags (preferred for simple diagrams)

No JSON quoting needed. Agent-friendly.

```sh
drawz render flow --steps 'Build,Test,Deploy' --direction LR
drawz render table --headers 'Name,Status' --row 'Alice,Active' --row 'Bob,Inactive'
drawz render tree --indent 'src\n  main.rs\n  lib.rs'
drawz render sequence --actors 'Client,Server' --msg 'Client:Server:GET /api' --msg 'Server:Client:200 OK'
drawz render state --edge 'Idle:Running:start' --edge 'Running:Done:finish'
drawz render dag --edge 'Parse:Lint' --edge 'Parse:Compile' --edge 'Lint:Link' --edge 'Compile:Link'
drawz render mermaid --code 'graph LR; A-->B-->C'
drawz render freeform --content 'line1\nline2\nline3'
```

### Method 2: JSON argument (complex diagrams)

Use when the diagram needs nested structures, node IDs, or fields not covered by flags.

```sh
drawz '{"type":"flow","steps":["A",{"label":"B","steps":["X","Y"]},"C"]}'
```

### Method 3: Heredoc (large/complex JSON)

For multi-line JSON that would be unwieldy as a single-line argument:

```sh
drawz <<'EOF'
{
  "type": "dag",
  "nodes": [{"id": "p", "label": "Parse"}, {"id": "l", "label": "Lint"}],
  "edges": [{"from": "p", "to": "l", "label": "AST"}]
}
EOF
```

### Method 4: Pipe

```sh
echo '{"type":"table","headers":["A","B"],"rows":[["1","2"]]}' | drawz
```

## Flag Reference

### Global

| Flag | Effect |
|------|--------|
| `-w <WIDTH>` | Constrain output width (default: terminal width or 120) |

### Per-type flags

| Subcommand | Flags |
|------------|-------|
| `flow` | `--steps` (comma-sep), `--direction` (LR/TD), `--title` |
| `table` | `--headers` (comma-sep), `--row` (comma-sep, repeatable), `--title` |
| `tree` | `--indent` (use `\n` for newlines), `--title` |
| `sequence` | `--actors` (comma-sep), `--msg` (from:to:label, repeatable), `--title` |
| `state` | `--edge` (from:to or from:to:label, repeatable), `--title` |
| `dag` | `--edge` (from:to or from:to:label, repeatable), `--title` |
| `mermaid` | `--code` (or stdin), `--title` |
| `freeform` | `--content` (use `\n` for newlines, or stdin), `--title` |

## Width Handling

drawz auto-detects terminal width. Override with `-w`:

```sh
drawz -w 60 render flow --steps 'A,B,C,D,E'
```

When width is not specified:
1. Uses terminal width if detectable
2. Falls back to 120 characters

## Error Handling

drawz exits with code 1 on errors and prints to stderr:

```sh
# Missing required args
drawz render table --headers 'A,B'
# → error: at least one --row is required

# Invalid JSON
drawz '{"type":"bogus"}'
# → error: invalid diagram JSON: ...
```

**Agent rule**: if drawz exits non-zero, report the error message to the user. Do not retry
with different input unless you can identify and fix the problem.

## Rules

1. **Never repeat the rendered output** — it's already visible in the shell result
2. **Never hand-draw diagrams** — always use drawz
3. **Prefer `render` subcommands** over raw JSON for simple cases (avoids quoting issues)
4. **Use heredoc** for JSON with nested objects, special characters, or >100 chars
5. **Add at most one sentence** describing what the diagram shows
6. **Use single quotes** around arguments to prevent shell expansion
7. **For complex graphs**, prefer `render mermaid --code '...'` over verbose edge lists
8. **Colon-separated edge format**: `from:to:label` — if a node name contains colons, use JSON method instead

## Decision Guide

```
Is the diagram simple (flat steps, rows, edges)?
  YES → drawz render <type> --flags
  NO → Does it need nested structures or node IDs?
    YES → drawz <<'EOF' ... EOF  (heredoc JSON)
    NO → Does agent already have Mermaid code?
      YES → drawz render mermaid --code '...'
      NO → drawz '<json>'  (single-line JSON arg)
```

## Examples

### Pipeline visualization
```sh
drawz render flow --steps 'Parse,Validate,Transform,Load' --direction LR --title 'ETL Pipeline'
```

### Comparison table
```sh
drawz render table --headers 'Approach,Latency,Complexity' \
  --row 'MCP,High (streaming),Low' \
  --row 'Shell,Zero,Medium' \
  --row 'Hybrid,Low,High'
```

### Architecture as Mermaid
```sh
drawz render mermaid --code 'graph LR; Client-->Gateway-->Service-->DB'
```

### Complex DAG with heredoc
```sh
drawz <<'EOF'
{
  "type": "dag",
  "edges": [
    {"from": "Parse", "to": "Lint"},
    {"from": "Parse", "to": "TypeCheck"},
    {"from": "Lint", "to": "Bundle"},
    {"from": "TypeCheck", "to": "Bundle"},
    {"from": "Bundle", "to": "Deploy"}
  ]
}
EOF
```
