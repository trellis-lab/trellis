---
layout: page
title: MCP Server
---

# Trellis MCP Server

The Trellis MCP server exposes diagram rendering as tools for Claude Desktop and other MCP-compatible clients. It wraps the `trellis` CLI binary and supports both stdio and HTTP transports.

---

## Prerequisites

- Python 3.10+
- `trellis` binary installed and accessible
- `pip install fastmcp prefab-ui`

---

## Installation

```bash
# Clone or navigate to the mcp/ directory
cd trellis/mcp/

# Install dependencies
pip install -r requirements.txt
```

`requirements.txt` contents:

```
fastmcp
prefab-ui
```

---

## Environment variables

| Variable | Default | Description |
|----------|---------|-------------|
| `TRELLIS_BIN` | `./trellis` | Path to the `trellis` binary |
| `TRELLIS_MCP_TRANSPORT` | `stdio` | Transport mode: `stdio` or `http` |

Use an absolute path for `TRELLIS_BIN` in production:

```bash
export TRELLIS_BIN=/usr/local/bin/trellis
```

---

## Running the server

### stdio (for MCP clients)

```bash
python server.py
```

The server communicates over stdin/stdout. MCP clients connect via the Claude Desktop config (see below).

### HTTP (for testing and debugging)

```bash
TRELLIS_MCP_TRANSPORT=http python server.py
# Listens on http://127.0.0.1:9000
```

---

## Claude Desktop configuration

Add to `~/Library/Application Support/Claude/claude_desktop_config.json` (macOS) or the equivalent path on your platform:

```json
{
  "mcpServers": {
    "trellis": {
      "command": "python",
      "args": ["/path/to/trellis/mcp/server.py"],
      "env": {
        "TRELLIS_BIN": "/usr/local/bin/trellis"
      }
    }
  }
}
```

Replace `/path/to/trellis/mcp/server.py` and `/usr/local/bin/trellis` with the actual paths on your system.

---

## Tools

### `render`

Renders a Mermaid diagram string. Returns the result as SVG or HTML text, or base64-encoded PNG, or ASCII text.

**Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `mermaid` | string | required | Mermaid diagram source |
| `format` | string | `svg` | Output format: `svg`, `png`, `html`, `ascii` |
| `theme` | string | `default` | Theme name |

**Returns:** Image content for SVG/PNG, or text content for HTML/ASCII.

**Example — ask Claude:**

> "Render this as a Trellis diagram in SVG:
> ```
> flowchart LR
>   User --> API --> DB
> ```"

---

### `run-interactive`

Renders a Mermaid diagram as an interactive embedded HTML viewer inside Claude Desktop. Click nodes and edges to explore the diagram.

**Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `mermaid` | string | required | Mermaid diagram source |

**Returns:** Embedded HTML panel (1200×800px) rendered inside Claude Desktop.

---

## Resources

### `themes://list`

Returns a JSON array of available themes with names and descriptions.

```json
[
  {"name": "default",   "description": "Default Trellis theme"},
  {"name": "paper",     "description": "Paper theme"},
  {"name": "blueprint", "description": "Blueprint theme"},
  {"name": "dark",      "description": "Dark theme"},
  {"name": "midnight",  "description": "Midnight theme"},
  {"name": "forest",    "description": "Forest theme"}
]
```

---

## Usage examples

### Flowchart

```
flowchart LR
  User -->|HTTP| API
  API -->|SQL| DB
  API -->|Cache| Redis
```

### Class diagram

```
classDiagram
  Animal <|-- Dog
  Animal <|-- Cat
  Animal : +String name
  Animal : +speak()
  Dog : +fetch()
  Cat : +purr()
```

### ER diagram

```
erDiagram
  CUSTOMER ||--o{ ORDER : places
  ORDER ||--|{ LINE-ITEM : contains
  PRODUCT ||--o{ LINE-ITEM : "included in"
```

### C4 architecture diagram

```
C4Context
  title System Overview
  Person(user, "User", "A customer")
  System(web, "Web App", "Frontend")
  System_Ext(auth, "Auth Service", "OAuth provider")
  Rel(user, web, "Uses")
  Rel(web, auth, "Authenticates via")
```
