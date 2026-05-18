import base64
import json
import os
import subprocess

from fastmcp import Context, FastMCP
from fastmcp.dependencies import CurrentContext
from mcp.types import Icon, ImageContent, TextContent, ToolAnnotations, ToolResultContent
from prefab_ui.components import Embed

TRANSPORT_HTTP = "http"
TRANSPORT_STDIO = "stdio"

DEFAULT_HTTP_HOST = "127.0.0.1"
DEFAULT_HTTP_PORT = 9000

ENV_TRELLIS_BIN = "TRELLIS_BIN"

OUTPUT_PNG: str = "png"
OUTPUT_SVG: str = "svg"
OUTPUT_HTML: str = "html"
OUTPUT_ASCII: str = "ascii"

VALID_FORMATS = [OUTPUT_SVG, OUTPUT_PNG, OUTPUT_HTML, OUTPUT_ASCII]

THEMES = [
    {"name": "default", "description": "Default Trellis theme"},
    {"name": "paper", "description": "Paper theme"},
    {"name": "blueprint", "description": "Blueprint theme"},
    {"name": "dark", "description": "Dark theme"},
    {"name": "midnight", "description": "Midnight theme"},
    {"name": "forest", "description": "Forest theme"},
]

VALID_THEMES = [t["name"] for t in THEMES]


# Initialize
mcp = FastMCP(
    "Trellis",
    instructions="",
    icons=[Icon(src="https://trellislab.net/assets/trellis-logo-narrow.svg", mimeType="image/svg")],
    version="0.10.0",
)


# Utilities
def _get_return_content(output: str, output_type: str) -> ImageContent | TextContent:
    match output_type:
        case "png":
            return ImageContent(
                type="image", data=base64.b64encode(output).decode("utf-8"), mimeType="image/png"
            )

        case "svg":
            return ImageContent(type="image", data=output, mimeType="image/svg+xml")

        case _:
            return TextContent(
                type="text",
                text=output,
            )


# Resources
@mcp.resource("themes://list")
def list_themes() -> str:
    return json.dumps(THEMES)


## Tools
@mcp.tool(
    "render",
    description="Render a Mermaid diagram using Trellis. Returns SVG/HTML/ASCII as string, or base64-encoded PNG.",
    tags=["mermaid", "render"],
    annotations=ToolAnnotations(
        title="", readOnlyHint=True, destructiveHint=False, idempotentHint=True, openWorldHint=False
    ),
)
def render_mermaid(
    mermaid: str,
    format: str = OUTPUT_SVG,
    theme: str = "default",
    ctx: Context | None = None,
) -> ToolResultContent:
    """
    Render a Mermaid diagram via the Trellis binary (TRELLIS_BIN env var).
    Reads diagram from stdin, returns rendered output from stdout.
    PNG output is base64-encoded; all other formats are plain strings.
    """

    if ctx is None:
        ctx = CurrentContext()

    try:
        if format not in VALID_FORMATS:
            ctx.error(f"Invalid format '{format}'. Must be one of: {', '.join(VALID_FORMATS)}")
            return
        if theme not in VALID_THEMES:
            ctx.error(f"Invalid theme '{theme}'. Must be one of: {', '.join(VALID_THEMES)}")
            return

        binary = os.environ.get(ENV_TRELLIS_BIN, "./trellis")
        cmd = [binary, "render", "-", "-f", format]

        ctx.debug(cmd)

        is_binary = format == OUTPUT_PNG
        result = subprocess.run(
            cmd,
            input=mermaid,
            capture_output=True,
            text=not is_binary,
        )

        if result.returncode != 0:
            stderr = result.stderr if isinstance(result.stderr, str) else result.stderr.decode()
            ctx.error(f"Trellis render failed (exit {result.returncode}): {stderr.strip()}")

        return ToolResultContent(content=[_get_return_content(result.stdout, format)])
    except Exception as e:
        ctx.error(f"Render error: {e}")


# Apps
@mcp.tool(
    "run-interactive",
    app=True,
    annotations=ToolAnnotations(
        title="", readOnlyHint=True, destructiveHint=False, idempotentHint=True, openWorldHint=False
    ),
)
def run_interactive(
    mermaid: str,
    ctx: Context | None = None,
) -> Embed:
    if ctx is None:
        ctx = CurrentContext()

    try:
        binary = os.environ.get(ENV_TRELLIS_BIN, "./trellis")
        cmd = [binary, "render", "-", "-f", OUTPUT_HTML]

        result = subprocess.run(
            cmd,
            input=mermaid,
            capture_output=True,
            text=True,
        )

        if result.returncode != 0:
            stderr = result.stderr if isinstance(result.stderr, str) else result.stderr.decode()
            ctx.error(f"Trellis render failed (exit {result.returncode}): {stderr.strip()}")

        return Embed(html=result.stdout, width="1200px", height="800px")
    except Exception as e:
        ctx.error(f"Render error: {e}")


# Run server
if __name__ == "__main__":
    transport = os.environ.get("TRELLIS_MCP_TRANSPORT", TRANSPORT_STDIO)

    if transport == TRANSPORT_HTTP:
        mcp.run(transport=TRANSPORT_HTTP, host=DEFAULT_HTTP_HOST, port=DEFAULT_HTTP_PORT)
    else:  # stdio
        mcp.run()
