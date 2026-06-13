---
layout: default
title: MCP Server
nav_order: 5
---

# Trellis MCP Server

The Trellis MCP server exposes diagram rendering as a tool for Claude Desktop and other MCP-compatible clients. It wraps the `trellis` CLI binary and supports both stdio and HTTP transports.

Built with [FastMCP](https://github.com/jlowin/fastmcp).

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
| `TRELLIS_KEY` | — | License key. Required for `svg`, `html`, and `drawio` output |

Use an absolute path for `TRELLIS_BIN` in production:

```bash
export TRELLIS_BIN=/usr/local/bin/trellis
```

> A `TRELLIS_KEY` is required for SVG, HTML, and Draw.io output. See [Licensing](licensing) for the subscribe link and key setup.

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
        "TRELLIS_BIN": "/usr/local/bin/trellis",
        "TRELLIS_KEY": "YOUR_KEY"
      }
    }
  }
}
```

Replace `/path/to/trellis/mcp/server.py`, `/usr/local/bin/trellis`, and `YOUR_KEY` with the actual values on your system.

---

## Tools

### `render`

Renders a Mermaid diagram string and returns the result. PNG is returned as a base64-encoded image; all other formats are returned as text.

**Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `mermaid` | string | required | Mermaid diagram source |
| `format` | string | `png` | Output format: `svg`, `png`, `html`, `drawio` |
| `theme` | string | `default` | Theme name: `default`, `paper`, `blueprint`, `dark`, `midnight`, `forest` |

**Returns:** Image content for PNG; text content for SVG, HTML, and Draw.io.

> SVG, HTML, and Draw.io output require a `TRELLIS_KEY`. See [Licensing](licensing).

**Example — ask Claude:**

> "Render this as a Trellis diagram:
> ```
> flowchart LR
>   User --> API --> DB
> ```"

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

### `formats://list`

Returns a JSON array of available output formats with descriptions.

```json
[
  {"name": "png",    "description": "(Default) Raster image result"},
  {"name": "svg",    "description": "Vector image result"},
  {"name": "html",   "description": "Interactive HTML output"},
  {"name": "drawio", "description": "Drawio output format for further editing"}
]
```

Format purposes and details: [CLI Reference → Output formats](cli#output-formats).

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
