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

FORMAT GUIDE (pick by goal):
- HTML: interactive review tool — exploring/navigating a diagram in a browser
- DRAWIO: Drawio XML — when the user wants to edit the diagram further
- SVG: scalable vector — quick high-quality preview or web embedding
- PNG: raster, fixed size — unlicensed users (no TRELLIS_KEY) or a plain image file
Note: SVG/HTML/DRAWIO require TRELLIS_KEY; PNG works without a license. If no key
is set, default to PNG.

C4 NODE TYPES:
Trellis supports standard Mermaid C4 plus Trellis-specific container shapes
(use in C4Container / C4Component diagrams). Prefer the specific shape when it
matches an element's role rather than a generic Container.
- ContainerFrontend — web UI / browser app (chrome bar)
- ContainerApp      — desktop/mobile app
- ContainerFolder   — file/config store (folder tab)
- ContainerGateway  — API gateway / router
- ContainerBucket   — object storage (S3-like)
Each also has a _Ext variant for external/third-party systems.
Standard: Person, System(Db|Queue), Container(Db|Queue), Component(Db|Queue) (+_Ext).
Boundaries: Enterprise_Boundary, System_Boundary, Container_Boundary, Deployment_Node.
Relations: Rel, BiRel, Rel_U/D/L/R, Rel_Back — Rel(from, to, label[, tech]).
Arg order: System/Person = (alias, label, description);
Container/Component = (alias, label, technology, description).

ENVIRONMENT:
- TRELLIS_BIN: Path to trellis binary (default: ./trellis)
- TRELLIS_MCP_TRANSPORT: http or stdio (default: stdio)
- TRELLIS_KEY: license key. Mandatory for SVG/HTML/DRAWIO format
""",
    icons=[Icon(src="https://trellislab.net/assets/trellis-logo-narrow.svg", mimeType="image/svg")],
    version="1.0.2",
)

# Resources
@mcp.resource("themes://list")
def list_themes() -> str:
    return json.dumps(THEMES)


@mcp.resource("formats://list")
def list_themes() -> str:
    return json.dumps(FORMATS)


## Prompts
@mcp.prompt(
    name="c4_cheatsheet",
    description="Reference for all Trellis C4 node types, including Trellis-specific shapes not in standard Mermaid.",
)
def c4_cheatsheet() -> str:
    return """You are helping author a Trellis C4 diagram. Trellis supports standard
Mermaid C4 plus these TRELLIS-SPECIFIC container shapes (container-level diagrams only,
i.e. C4Container / C4Component):

- ContainerFrontend(alias, label, tech, desc) — web UI / browser app (chrome bar)
- ContainerApp(alias, label, tech, desc)      — desktop/mobile app (play-button deco)
- ContainerFolder(alias, label, tech, desc)   — file/config store (folder tab)
- ContainerGateway(alias, label, tech, desc)  — API gateway / router
- ContainerBucket(alias, label, tech, desc)   — object storage (S3-like)
Each also has a _Ext variant for external/third-party systems.

Standard types: Person, Person_Ext, System, SystemDb, SystemQueue (+_Ext),
Container, ContainerDb, ContainerQueue, Component, ComponentDb, ComponentQueue.
Boundaries: Enterprise_Boundary, System_Boundary, Container_Boundary, Deployment_Node.
Relations: Rel, BiRel, Rel_U/D/L/R, Rel_Back — Rel(from, to, label[, tech]).

Arg order:
- System/Person      = (alias, label, description)
- Container/Component = (alias, label, technology, description)

Prefer the specific Trellis shape over a generic Container when it matches the
element's role (UI -> ContainerFrontend, storage bucket -> ContainerBucket, etc.).
Then call the `render` tool with format=svg."""


@mcp.prompt(
    name="c4_pick_shape",
    description="Map a plain-English element description to the right Trellis C4 node type.",
)
def c4_pick_shape(element: str) -> str:
    return f"""Pick the best Trellis C4 node type for: "{element}".

Mapping rules:
- web/browser UI, SPA, React/Vue frontend -> ContainerFrontend
- desktop or mobile app                   -> ContainerApp
- database                                -> ContainerDb / SystemDb
- message queue / topic / broker          -> ContainerQueue / SystemQueue
- object storage (S3, GCS, blob)          -> ContainerBucket
- API gateway / reverse proxy / router    -> ContainerGateway
- config / file store / folder            -> ContainerFolder
- generic service / API                   -> Container
- third-party/external                    -> add _Ext suffix

Output the single C4 line with placeholder args, then offer to render it."""


@mcp.prompt(
    name="output_format_guide",
    description="Pick the right Trellis render output format for the user's goal.",
)
def output_format_guide() -> str:
    return """Choose the `render` format that fits the user's intent:

- html   — interactive review tool; best for exploring/navigating a diagram in a browser
- drawio — Drawio XML; best when the user wants to edit the diagram further
- svg    — scalable vector; best for a quick, high-quality preview or web embedding
- png    — raster image; use for unlicensed users (no TRELLIS_KEY) or when a fixed-size
           image file is needed

Note: svg / html / drawio require a license key (TRELLIS_KEY). png works without a license.
If no key is set, default to png. Ask the user which they want when the goal is ambiguous,
then call the `render` tool with the chosen format."""


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
