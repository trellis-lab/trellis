import json
import os
import subprocess

from fastmcp import Context, FastMCP
from fastmcp.dependencies import CurrentContext
from fastmcp.exceptions import ToolError
from fastmcp.utilities.types import Image
from mcp.types import Icon, ToolAnnotations


TRANSPORT_HTTP = "http"
TRANSPORT_STDIO = "stdio"

DEFAULT_HTTP_HOST = "127.0.0.1"
DEFAULT_HTTP_PORT = 9000

ENV_TRELLIS_BIN = "TRELLIS_BIN"

OUTPUT_PNG: str = "png"

FORMATS = [
    {"name": "png", "description": "(Default) Raster image result"},
    {"name": "svg", "description": "Vector image result"},
    {"name": "html", "description": "Interactive HTML output"},
    {"name": "drawio", "description": "Drawio output format for further editing"},
]

VALID_FORMATS = [t["name"] for t in FORMATS]

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
    mask_error_details=True,
    instructions="""Trellis renders Mermaid diagrams with advanced styling and export options.

TOOLS:
- render: Convert Mermaid syntax to SVG/PNG/HTML/ASCII. Returns formatted output.
  Parameters: mermaid (diagram source), format (svg|png|html|ascii, default: svg), theme (default|paper|blueprint|dark|midnight|forest)
  Use for: Static exports, documentation, high-quality images

RESOURCE:
- themes://list: List available themes with descriptions
- formats://list: List of available output formats

BEST PRACTICES:
1. Use render with format=svg for web, format=png for images
2. Use run-interactive for exploration and live editing
3. Select theme based on context: default (neutral), dark/midnight (dark backgrounds), blueprint (technical), forest (organic)
4. Validate Mermaid syntax before rendering complex diagrams
5. For PNG output, ensure TRELLIS_BIN env var points to trellis binary

FORMAT GUIDE:
- SVG: Scalable, embeddable, good for web
- PNG: Raster, fixed size, good for images/docs
- HTML: Interactive in browser, includes styling
- DRAWIO: DRAWIO XML output enables further adjustments

ENVIRONMENT:
- TRELLIS_BIN: Path to trellis binary (default: ./trellis)
- TRELLIS_MCP_TRANSPORT: http or stdio (default: stdio)
- TRELLIS_KEY: license key. Mandatory for SVG/HTML/DRAWIO format
""",
    icons=[Icon(src="https://trellislab.net/assets/trellis-logo-narrow.svg", mimeType="image/svg")],
    version="0.10.0",
)

# Resources
@mcp.resource("themes://list")
def list_themes() -> str:
    return json.dumps(THEMES)


@mcp.resource("formats://list")
def list_themes() -> str:
    return json.dumps(FORMATS)


## Tools
@mcp.tool(
    "render",
    description="Render a Mermaid diagram to SVG, PNG, HTML, or Drawio using Trellis. Accepts Mermaid syntax, applies visual theme, returns formatted output.",
    tags=["mermaid", "render", "diagram", "visualization"],
    annotations=ToolAnnotations(
        title="Render Mermaid Diagram", readOnlyHint=True, destructiveHint=False, idempotentHint=True, openWorldHint=False
    ),
)
def render_mermaid(
    mermaid: str,
    format: str = OUTPUT_PNG,
    theme: str = "default",
    ctx: Context | None = None,
) -> Image | str:
    """
    Render a Mermaid diagram via the Trellis binary (TRELLIS_BIN env var).
    Reads diagram from stdin, returns rendered output from stdout.
    PNG output is base64-encoded; all other formats are plain strings.
    """ 

    if ctx is None:
        ctx = CurrentContext()

    try:
        if format not in VALID_FORMATS:
            err = f"Invalid format '{format}'. Must be one of: {', '.join(VALID_FORMATS)}"

            ctx.error(err)
            raise ToolError(err)
        
        if theme not in VALID_THEMES:
            err = f"Invalid theme '{theme}'. Must be one of: {', '.join(VALID_THEMES)}"

            ctx.error(err)
            raise ToolError(err)

        binary = os.environ.get(ENV_TRELLIS_BIN, "./trellis")
        cmd = [binary, "render", "-", "-f", format]

        ctx.debug(cmd)

        is_binary = format == OUTPUT_PNG
        result = subprocess.run(
            cmd,
            input=mermaid.encode() if is_binary else mermaid,
            capture_output=True,
            text=not is_binary,
        )

        if result.returncode == 0:
          if format == OUTPUT_PNG:
              return Image(data=result.stdout, format="png")
          else:
              return result.stdout
        else:
          stderr = result.stderr if isinstance(result.stderr, str) else result.stderr.decode()
          ctx.error(f"Trellis render failed (exit {result.returncode}): {stderr.strip()}")
          raise ToolError(stderr)

    except Exception as e:
        ctx.error(f"Render error: {e}")
        raise ToolError(str(e))

# Run server
if __name__ == "__main__":
    transport = os.environ.get("TRELLIS_MCP_TRANSPORT", TRANSPORT_STDIO)

    if transport == TRANSPORT_HTTP:
        mcp.run(transport=TRANSPORT_HTTP, host=DEFAULT_HTTP_HOST, port=DEFAULT_HTTP_PORT)
    else:  # stdio
        mcp.run()
