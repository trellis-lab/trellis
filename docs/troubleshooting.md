---
layout: default
title: Troubleshooting
nav_order: 8
---

# Troubleshooting

Common problems and how to fix them. If your issue isn't here, open an [issue](https://github.com/trellis-lab/trellis/issues).

---

## Installation

### `trellis: command not found`

The binary isn't on your `PATH`. Extract the release archive and either move `trellis` into a directory already on your `PATH` (e.g. `/usr/local/bin`) or add its location to `PATH`:

```bash
export PATH="$PATH:/path/to/trellis-dir"
```

### macOS: "cannot be opened because the developer cannot be verified"

The binary is unsigned. Allow it once:

```bash
xattr -d com.apple.quarantine /path/to/trellis
```

Then run it again.

---

## Rendering

### Output file is empty or the command exits non-zero

Run `trellis validate <input>` first to check for syntax errors. Validation reports the line and reason without rendering.

### ASCII output does nothing

ASCII (`-f ascii`) is in development and disabled by default in pre-built binaries. Use `svg`, `png`, `html`, or `drawio` instead.

### Docker: "permission denied" or empty output

Make sure you mount the working directory and reference files relative to `/data`:

```bash
docker run --rm -v "$(pwd):/data" ghcr.io/trellis-lab/trellis:latest input.mmd -o output.svg
```

Both input and output paths resolve inside `/data`.

---

## Licensing

### "license required" / `render-batch` refuses to run

`render-batch` and MCP `svg`/`html`/`drawio` output are commercial features. Activate a license on the device:

```bash
trellis license activate --license-key YOUR_KEY
trellis license status
```

See [Licensing](licensing) for the subscribe link and key-resolution order.

### "no seats available" when activating

Each activated device consumes one seat. If you're out of seats, free one by deactivating a device you no longer use:

```bash
# On the old device
trellis license deactivate
```

If you can't access the old device, deregister it on Lemon Squeezy (below).

---

## Deregistering a device on Lemon Squeezy

Trellis licenses are managed through **Lemon Squeezy**. Every activation creates a device *instance* tied to your license key. If a device is lost, wiped, or you simply want to free its seat remotely, remove that instance from the Lemon Squeezy customer portal.

1. Find your license email from Lemon Squeezy (subject line mentions your order / license key) and open the **"Manage your orders"** / customer portal link. You can also go to the portal directly:

   👉 [app.lemonsqueezy.com/my-orders](https://app.lemonsqueezy.com/my-orders)

2. Enter the email you used at checkout. Lemon Squeezy emails you a magic sign-in link — open it.

3. Open the Trellis order and locate your **license key**. Expand it to see the list of **activated instances** (one per device, shown by device name/identifier).

4. Click **Deactivate** (trash/remove) next to the device you want to deregister. That frees its seat immediately.

5. Activate the freed seat on the new device:

   ```bash
   trellis license activate --license-key YOUR_KEY
   ```

> Prefer `trellis license deactivate` on the device itself when it's still reachable — it's the cleanest way to free a seat. Use the Lemon Squeezy portal only when the device is gone.

For seat-count changes, upgrades, or billing, use the subscribe/manage link on the [Licensing](licensing) page.
