# Phase 4: Session Management - Context

**Gathered:** 2026-01-27
**Status:** Ready for planning

<domain>
## Phase Boundary

Container lifecycle commands for agent sessions — stop, restart, list, remove — without requiring raw podman commands. Users manage sessions through the agent-session CLI.

</domain>

<decisions>
## Implementation Decisions

### Command structure
- Subcommand style: `agent-session start`, `agent-session stop`, `agent-session ls`, `agent-session rm`
- Existing functionality (current script behavior) becomes the `start` subcommand
- Docker-style verbs: start, stop, rm, ls (familiar to container users)
- Running `agent-session` without subcommand shows help (not default to start)
- Session name is optional positional arg; defaults to current directory's session name

### Output format
- Colors for status: green=running, red=stopped
- Action feedback: echo what happened ("Stopped: myproject")
- No --json flag — human output only

### Confirmation behavior
- `rm` prompts for confirmation by default; `--force`/`-f` skips prompt
- `stop` does not prompt (stopping is reversible via restart)
- Cannot rm a running container — error: "Cannot remove running session. Stop it first."
- No bulk remove command — drop to podman for that use case

### Edge case handling
- Stopping already-stopped session: silent success (idempotent)
- Session not found: error with hint "Session 'foo' not found. Use 'agent-session ls' to see sessions."
- Empty session list: "No sessions found" message

### Claude's Discretion
- Exact ls output format (table vs plain lines)
- Zellij session loss handling on restart
- Exit codes for various error conditions

</decisions>

<specifics>
## Specific Ideas

- Pattern after docker/podman CLI conventions — familiar to container users
- Keep commands simple and focused; advanced operations can use podman directly

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 04-session-management*
*Context gathered: 2026-01-27*
