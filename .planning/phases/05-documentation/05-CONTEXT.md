# Phase 5: Documentation - Context

**Gathered:** 2026-01-27
**Status:** Ready for planning

<domain>
## Phase Boundary

Comprehensive documentation enabling a colleague to install and run their first agent session in under 5 minutes. Covers installation, usage, and troubleshooting in a single README.

</domain>

<decisions>
## Implementation Decisions

### Documentation structure
- Single README.md file (no separate docs folder)
- Section order: Overview first, then prerequisites, then quick start
- Command reference in collapsed `<details>` blocks to keep quick start scannable
- Troubleshooting in dedicated section at end (not collapsed)

### Quick start style
- Copy-paste command blocks (no inline comments cluttering commands)
- Quick start demonstrates agent selection (not just default Claude)
- Prerequisites list installation links to official docs

### Example depth
- Show expected output for key commands only (like `ls` showing sessions)
- Happy path scenarios only — troubleshooting section handles errors
- All subcommands documented with examples (start, stop, restart, ls, rm)
- Use generic placeholders in examples (`/path/to/project`, `my-session`)

### Tone and audience
- Mixed developer audience: some know containers/multiplexers, some don't
- Separate concepts section explaining key terms (podman vs docker, zellij vs tmux)
- Conversational tone — friendly, explains the "why" behind things
- Include value prop explaining the problem this tool solves

### Claude's Discretion
- How much context before each individual command (varies by command)
- Exact wording and flow within sections
- Which commands deserve output examples

</decisions>

<specifics>
## Specific Ideas

- Audience includes developers who may know docker but not podman, tmux but not zellij
- "5 minutes to first session" is the success metric — design for skimmability

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 05-documentation*
*Context gathered: 2026-01-27*
