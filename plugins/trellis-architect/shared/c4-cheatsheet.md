# C4 Cheatsheet (Trellis shapes)

Mirrors the Trellis MCP `c4_cheatsheet` prompt. Use this instead of asking the MCP
server when it isn't configured, and to pick a shape without a round-trip when it is.

## Diagram levels

| Diagram | Directive | Used from phase |
|---|---|---|
| Context | `C4Context` | expectations |
| Container | `C4Container` | design-ideas onward |
| Component | `C4Component` | select-design, documentation (when a container needs internal detail) |

## Standard node types

All take `_Ext` variants for external / third-party systems (e.g. `System_Ext`,
`Container_Ext`).

| Type | Argument order | Use for |
|---|---|---|
| `Person` | `(alias, label, description)` | A human role |
| `System`, `SystemDb`, `SystemQueue` | `(alias, label, description)` | Context-level systems |
| `Container`, `ContainerDb`, `ContainerQueue` | `(alias, label, technology, description)` | Generic deployable units at container level |
| `Component`, `ComponentDb`, `ComponentQueue` | `(alias, label, technology, description)` | Internals of a container |

## Trellis-specific container shapes

Prefer these over generic `Container` whenever the element's role matches — they
render with role-specific iconography in SVG/PNG/HTML and role-aware shapes in
Draw.io export.

| Type | Use for |
|---|---|
| `ContainerFrontend` | Web UI / browser app (rendered with a browser-chrome bar) |
| `ContainerApp` | Desktop / mobile app |
| `ContainerFolder` | File / config store (rendered as a folder tab) |
| `ContainerGateway` | API gateway / router |
| `ContainerBucket` | Object storage (S3-like) |

Each has an `_Ext` variant (`ContainerFrontend_Ext`, …) for the same role when it
lives outside the system boundary.

## Boundaries

`Enterprise_Boundary`, `System_Boundary`, `Container_Boundary`, `Deployment_Node`
— wrap related elements; nest freely.

## Relations

`Rel(from, to, label[, technology])`, `BiRel`, directional variants `Rel_U` /
`Rel_D` / `Rel_L` / `Rel_R` (up/down/left/right layout hints), `Rel_Back`
(reverse arrowhead without swapping source/target).

## Picking a shape

Ask: what does this element *do*, not what it's called.

- Serves a browser UI → `ContainerFrontend`.
- Native/mobile client → `ContainerApp`.
- Routes/authenticates/rate-limits traffic before it reaches business logic →
  `ContainerGateway`.
- Holds files, configs, or blobs with a filesystem-like access pattern →
  `ContainerFolder`.
- Object storage accessed by key (S3-like) → `ContainerBucket`.
- Runs business logic, a database, or a queue that isn't one of the above →
  generic `Container` / `ContainerDb` / `ContainerQueue`.
- Not owned by this system → add `_Ext` to whichever of the above fits.

## Minimal container example

```mermaid
C4Container
  title Web Platform — Containers
  Person(user, "User", "A customer")
  ContainerFrontend(spa, "Web UI", "React", "Single-page app")
  ContainerGateway(gw, "API Gateway", "Kong", "Routes traffic")
  Container(api, "API", "Rust/Actix", "Business logic")
  ContainerDb(db, "Database", "PostgreSQL", "Stores data")
  ContainerBucket(assets, "Assets", "S3", "Static files")
  Rel(user, spa, "Uses")
  Rel(spa, gw, "Calls", "HTTPS")
  Rel(gw, api, "Routes to")
  Rel(api, db, "Reads/writes")
  Rel(api, assets, "Stores objects")
```
