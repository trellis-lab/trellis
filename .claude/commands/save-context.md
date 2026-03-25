---
description: Save current session knowledge to memory MCP before ending or compacting. Preserves decisions, progress, and learnings.
model: haiku
allowed-tools: mcp__memory__create_entities, mcp__memory__create_relations, mcp__memory__add_observations
---

You are a context saver. Extract the most important knowledge from this session and persist it to the memory MCP.

## What to save

Review the conversation and identify:

1. **Decisions made** — architectural choices, library selections, trade-offs discussed
2. **Work completed** — features implemented, bugs fixed, refactors done
3. **Work in progress** — what's partially done, next steps, blockers
4. **New patterns** — code conventions discovered, debugging insights, gotchas found
5. **Key files touched** — which files were modified and why

## How to save

1. Create or update entities using `create_entities` with types:
   - `decision` for architectural/design choices
   - `task` for completed or in-progress work
   - `pattern` for conventions and recurring solutions
   - `session` for a session summary entity named with today's date

2. Add detailed observations to each entity via `add_observations`

3. Create relations between entities via `create_relations`:
   - "implements" (task → decision)
   - "discovered_in" (pattern → task)
   - "blocks" / "blocked_by" (task → task)
   - "continues" (today's session → previous session)

## Format

Use concise, factual language. Each observation should be one clear sentence.
Prefer specific file paths and function names over vague descriptions.
Tag session entities as "session-YYYY-MM-DD" for easy retrieval.
```

**The two commands form a loop:**
```
┌─ Session start ──────────────────────────┐
│  /project:recall          (Haiku reads)  │
│  ... do your work on Opus/Sonnet ...     │
│  /project:save-context    (Haiku writes) │
└──────────────────────────────────────────┘
