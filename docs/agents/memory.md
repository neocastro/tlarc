# Temporal Memory (minigraf)

The `temporal-reasoning` skill provides bi-temporal graph memory across
sessions. The graph file is `memory.graph` at the repo root (gitignored —
it is session state, not source).

## Starting the MCP server

The server is registered in `.mcp.json` (also gitignored) and must be
launched with an `mcp<2` pin:

```
uvx --with "mcp<2" temporal-reasoning[git-ingestion]
```

Without the pin, uv resolves `mcp 2.x` and the server crashes at startup
with `AttributeError: 'Server' object has no attribute 'list_tools'` — the
server code uses the 1.x `@server.list_tools()` decorator API.

Run from the repo root so `memory.graph` resolves to the right file (or set
`MINIGRAF_GRAPH_PATH=/home/neo/dev/algebraik/tlarc/memory.graph`).

## Direct fallback (no MCP server)

The `minigraf` Python binding can read/write the same graph without the
server, useful when the server is unavailable:

```bash
uvx --from minigraf python -c "
from minigraf import MiniGrafDb
db = MiniGrafDb.open('memory.graph')
print(db.execute('(query [:find ?a ?v :where [ENT ?a ?v]])'))
"
```

Query datalog is wrapped in `(query ...)`; writes in
`(transact {:valid-from "<ISO-8601-UTC-Z>"} [...])`.
