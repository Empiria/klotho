# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-01-26)

**Core value:** Consistent, reproducible agent environments that enable seamless handoff between people and agents through committed artifacts.
**Current focus:** Phase 2 - Agent Abstraction

## Current Position

Phase: 2 of 5 (Agent Abstraction)
Plan: 3 of 3 in current phase
Status: Phase complete
Last activity: 2026-01-26 - Completed 02-03-PLAN.md (Config-Driven Orchestration)

Progress: [██████████] 100% (Phase 2 complete)

## Performance Metrics

**Velocity:**
- Total plans completed: 6
- Average duration: 3.3 min
- Total execution time: 0.40 hours

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 01-release-audit | 3 | 18min | 6min |
| 02-agent-abstraction | 3 | 6min | 2min |

**Recent Trend:**
- Last 5 plans: 02-01 (2min), 02-02 (3min), 02-03 (1min)
- Trend: Infrastructure tasks are fast due to cached builds and focused scope

*Updated after each plan completion*

## Accumulated Context

### Decisions

Decisions are logged in PROJECT.md Key Decisions table.
Recent decisions affecting current work:

| Phase | Decision | Rationale |
|-------|----------|-----------|
| 01-01 | Use $HOME variable expansion in examples | Portability across systems; variable expansion is idiomatic |
| 01-02 | Support macOS via Podman Desktop and podman machine | User requirement for macOS compatibility |
| 01-02 | Document Homebrew as macOS package manager | Standard package manager for macOS developer tools |
| 01-02 | Include platform-specific troubleshooting sections | Different failure modes on Linux vs macOS require separate guidance |
| 01-03 | Use conditional checks for optional directories | Preserves "optional" semantics, better UX than requiring mkdir |
| 01-03 | Follow EXTRA_MOUNTS pattern for OPTIONAL_MOUNTS | Maintains codebase consistency with existing mount handling |
| 02-01 | Shell-sourceable KEY=value format for agent configs | Simple, portable, easy to parse and validate, idiomatic for shell scripts |
| 02-01 | No command substitution allowed in configs | Security requirement to prevent code injection via config files |
| 02-01 | XDG override path at ~/.config/agent-session/agents/ | Standard Linux convention, allows user customization without modifying repository |
| 02-02 | Reject only command substitution in config validation | Pipes/semicolons in quoted strings are safe during sourcing; only $() and backticks execute |
| 02-02 | Install uv in claude stage, not base | Agent-specific tooling (Python MCP servers) doesn't belong in shared base stage |
| 02-02 | Declare ARGs only in agent stages | Keep base stage parameter-free for better caching and cleaner separation |
| 02-03 | Default --agent to "claude" for backward compatibility | Existing usage patterns should continue working without requiring new flags |
| 02-03 | Use agent-session-<agent>:latest image naming pattern | Consistent naming across all agents, makes agent type clear from image name |
| 02-03 | Merge mounted .claude config rather than replace | Claude Code creates ~/.claude during build; merging allows both container and user configs to coexist |
| 02-03 | Agent detection for conditional behavior in entrypoint | Enables agent-agnostic base with nice agent-specific experiences via detection |

### Pending Todos

[From .planning/todos/pending/ - ideas captured during sessions]

None yet.

### Blockers/Concerns

[Issues that affect future work]

None yet.

## Session Continuity

Last session: 2026-01-26 16:29:42 UTC
Stopped at: Completed 02-03-PLAN.md (Config-Driven Orchestration) - Phase 2 complete
Resume file: None
