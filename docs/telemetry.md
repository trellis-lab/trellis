---
layout: default
title: Telemetry
nav_order: 9
---

# Telemetry

Trellis collects anonymous usage pings when you run licensed commands. No personal information, no file content, no machine identifiers — only the three fields described below.

---

## What is collected

Each ping contains exactly three fields:

| Field | Type | Example | Description |
|---|---|---|---|
| `key_fingerprint` | string (16 hex chars) | `"a1b2c3d4e5f60718"` | SHA-256 of your license key, first 8 bytes as lowercase hex. The raw key is never sent. |
| `command` | string | `"render-batch"` | The CLI command that was run. One of: `render`, `render-batch`, `validate`, `evaluate`, `evaluate-batch`, `preprocess`, `generate-review`, `help`, `license`. |
| `cli_version` | string | `"1.2.0"` | The version of the Trellis binary, compiled in at build time. |

**Nothing else is sent.** No file paths, no diagram content, no system info (OS, CPU, hostname), no IP-derived location, no timestamps beyond what the server records from the HTTP request itself.

### Why `key_fingerprint` and not a random ID

The fingerprint lets us count unique licensed users and detect anomalous usage patterns without ever storing your license key. The 16-character hex is a one-way truncated SHA-256 — it cannot be reversed to recover the original key.

---

## How it is transmitted

- **Method:** HTTP POST to `https://api.trellislab.net/v1/usage`
- **Format:** JSON body with the three fields above
- **Timing:** Fire-and-forget in a detached background thread; never blocks or delays your command
- **Timeout:** 2 seconds — if the request does not complete in time, it is silently dropped
- **Failure handling:** Any network error is silently ignored; telemetry failure never affects exit codes or output

---

## When it fires

Two conditions must both be true for a ping to be sent:

1. `telemetry = true` in your config (default)
2. Your license is valid (`trellis license status` shows active)

Unlicensed users never send telemetry.

---

## Opting out

Set `telemetry = false` in your `trellis.toml` config file:

```toml
# trellis.toml
telemetry = false
```

Pass it with `--config`:

```bash
trellis render input.mmd -o output.svg --config trellis.toml
```

Or keep the config file at a fixed path and always pass `--config`. There is no separate opt-out command.

---

## WASM builds

The telemetry module is excluded from WASM builds entirely — the VS Code extension live preview and browser editor never send usage pings.
