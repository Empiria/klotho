# Agent Session

## What This Is

A containerized environment for running AI coding agents with full permissions safely sandboxed. Users mount their config directories so agents behave identically to local, while Zellij sessions allow disconnecting and reconnecting from any machine. The real value: any team member can pick up where any agent left off because memory lives in committed tool artifacts (serena, gsd), not sessions.

## Core Value

Consistent, reproducible agent environments that enable seamless handoff between people and agents through committed artifacts.

## Requirements

### Validated

- ✓ Container-based sandboxed execution — agents run with full permissions safely — existing
- ✓ Zellij sessions persist across disconnects — reconnect from anywhere — existing
- ✓ Claude Code integration — installed and configured in container — existing
- ✓ Config directory mounting — user's `.claude/` injected via bind mounts — existing
- ✓ Session creation with named containers — `agent-session -n NAME /path` — existing
- ✓ Session reattachment — reconnect to running or stopped containers — existing
- ✓ GSD plugin installation — automatic at container startup — existing
- ✓ Non-root container execution — runs as `agent` user with UID mapping — existing

### Active

- [ ] Multi-agent support — select between Claude, opencode, and future agents
- [ ] Interactive agent selection — menu-based choice when starting session
- [ ] Usage documentation — setup and daily usage guide for colleagues
- [ ] Release audit — verify no hardcoded paths, secrets, or personal config

### Out of Scope

- Community contributions — open source for distribution convenience, not collaboration
- Centralized/shared infrastructure — each user runs their own container locally
- Agent-specific features — this is the environment, not the agents themselves

## Context

**Current state:** Working for Owen, Claude-only, shell script orchestration.

**Team tooling already agreed:**
- Serena (memory/context tool)
- Context7 (documentation lookup)
- GSD (planning/execution workflow)

These are pre-installed/configured because they're team standards. Agent choice is personal preference.

**How handoff works:** Agents create files (serena memories, gsd planning docs) that get committed. Any agent reading those files can continue the work — the memory is in the repo, not the session.

**Tech stack:**
- Podman containers (Debian bookworm-slim base)
- Zellij terminal multiplexer
- Bash orchestration script
- Fish shell + Starship prompt in container

## Constraints

- **Distribution**: Must be simple to install — colleagues should be able to get started quickly
- **Maintenance**: Keep it simple — shell script is fine if it works, only add complexity if needed
- **Portability**: No hardcoded paths or personal config — must work on any Linux machine with Podman
- **User autonomy**: Each user brings their own configs and agent preferences

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Podman over Docker | Rootless containers, no daemon, better security model | — Pending |
| Zellij over tmux | Better default experience, session management built-in | — Pending |
| Mount configs vs copy | Users get their own preferences, changes persist | — Pending |
| Tool artifacts for memory | Enables handoff between agents/users via committed files | — Pending |

---
*Last updated: 2026-01-26 after initialization*
