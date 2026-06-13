---
layout: default
title: Licensing
nav_order: 7
---

# Licensing

Trellis is free to use for everyday rendering. Some advanced capabilities are commercial features that require a license key.

---

## What's free

- Rendering single diagrams with `trellis render` (PNG)
- Syntax validation with `trellis validate`
- The VS Code extension live preview and PNG creation
- The browser-based [live editor](https://trellislab.net/editor.html)
- MCP server to render PNG images out of mermaid definition

---

## What needs a license

| Feature | Where |
|---|---|
| `render-batch` — render a whole directory at once | CLI |
| SVG / HTML / Draw.io output | --license-key `KEY` command line argument or `TRELLIS_KEY` env variable required |
| SVG / HTML / Draw.io output via the MCP server | MCP server (`TRELLIS_KEY` required) |

Each activated device consumes one seat from your license.

---

## Subscribe

👉 **[Subscribe to Trellis](https://trellislab.net/#pricing)**

After subscribing you'll receive a license key. Activate it on each device you use.

---

## Activating a license (CLI)

```bash
# Activate this device (consumes one seat)
trellis license activate --license-key YOUR_KEY

# Show current status and device instance
trellis license status

# Deactivate this device (frees the seat)
trellis license deactivate
```

### Key resolution order

When a command needs a license, the key is resolved in this priority order:

1. `--license-key KEY` passed on the command line
2. `TRELLIS_KEY=...` in a `.env` file in the working directory
3. `TRELLIS_KEY` environment variable

```bash
# .env file approach
echo "TRELLIS_KEY=YOUR_KEY" > .env
trellis license activate
```

---

## License key for the MCP server

The MCP server requires `TRELLIS_KEY` to be set for SVG, HTML, and Draw.io output:

```bash
export TRELLIS_KEY=YOUR_KEY
```

Or set it in the MCP client configuration `env` block — see the [MCP guide](mcp).

---

## Questions

For licensing questions, seat counts, or team plans, contact us via the [issues page](https://github.com/trellis-lab/trellis/issues) or the subscribe link above.
