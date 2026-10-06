---
layout: default
title: Prompting Your AI Agent
nav_order: 6
---

# Prompting Your AI Agent

AI agents already write valid Mermaid. What they don't do on their own is use **Trellis commands** — the `%% trellis <command> <argument>` comments Trellis reads on top of standard Mermaid (see [Mermaid Language Support](mermaid-support)). This page is a reference for those commands and how to get an agent to actually use them, whether it talks to Trellis through the [MCP server](mcp) or just writes a `.mmd` file for you to render yourself.

---

## Trellis commands

A Trellis command is a comment placed directly above the statement it applies to:

```
%% trellis <command> <argument>
```

Every other Mermaid renderer sees a plain comment and ignores it — a diagram using Trellis commands still renders correctly (minus the extra behavior) anywhere else. This section grows as new commands ship; today there's one.

### `tag`

```
%% trellis tag frontend, public
Web[Web App]
```

Assigns a comma-separated, trimmed, deduplicated tag list to the node below. Stacking multiple `tag` lines (or re-tagging an already-tagged node) merges the lists:

```
%% trellis tag backend, api
%% trellis tag security
Auth[Auth Service]
```

`Auth` ends up tagged `backend, api, security`.

Tags surface in the **interactive HTML output** (`-f html`): a legend lists every tag with a count, and clicking one highlights matching nodes plus their edges and dims the rest. SVG, PNG, and Draw.io output are unaffected. An unknown command name or malformed argument is silently ignored — same Mermaid-compatible behavior as unsupported syntax elsewhere.

---

## Teaching your agent to use them

An agent has no reason to reach for `tag` unless you tell it to — it's not standard Mermaid. Two ways to make that instruction stick:

**One-off, in a chat prompt:**

> "Design a C4 container diagram for an order-processing system. Tag each
> container with its owning team (`payments`, `fulfillment`, `platform`) and
> mark anything handling PII with a `pii` tag, using Trellis's
> `%% trellis tag <a>, <b>` comment syntax."

**Standing instruction, for every diagram going forward** — add this to whatever your agent reads as project-level rules (`CLAUDE.md`, `AGENTS.md`, `.cursorrules`, a Copilot instructions file, or a custom system prompt):

```markdown
When writing Mermaid diagrams for Trellis, tag nodes with
`%% trellis tag <a>, <b>` comments placed directly above the node.
Use tags to mark domain/team ownership, data sensitivity (e.g. `pii`),
or anything else worth filtering by later. Multiple `tag` lines on the
same node merge. This is a plain Mermaid comment — never invent other
`%% trellis ...` syntax beyond what's documented.
```

If your agent talks to Trellis through the [MCP server](mcp), it already gets equivalent guidance automatically via the server's connection instructions — no extra prompting needed.

---

## Example: tagging a C4 diagram

```
C4Container
  title Order Processing — Container Diagram

  %% trellis tag platform
  Person(customer, "Customer", "Places and tracks orders")

  %% trellis tag payments, pii
  Container(billing, "Billing Service", "Rust", "Charges cards, stores billing address")

  %% trellis tag fulfillment
  Container(shipping, "Shipping Service", "Go", "Creates shipments, tracks delivery")

  %% trellis tag platform
  ContainerDb(db, "Order DB", "PostgreSQL", "Orders, line items, status")

  Rel(customer, billing, "Pays via")
  Rel(billing, db, "Reads/writes")
  Rel(shipping, db, "Reads/writes")
```

Render it as `html` and click the `pii` tag — every container that touches sensitive data lights up, along with its edges, while the rest of the diagram dims. Useful for onboarding, security review, or just checking the agent tagged what you asked it to.
