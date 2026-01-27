---
phase: 07-rust-rewrite
plan: 02
subsystem: infra
tags: [rust, config, podman, docker, container-runtime]

# Dependency graph
requires:
  - phase: 07-01
    provides: Rust project structure with clap-based CLI
provides:
  - Agent config loading with XDG layering and security validation
  - Container runtime abstraction supporting podman and docker
  - Runtime auto-detection with --runtime flag override
affects: [07-03, 07-04, 07-05, 07-06, 07-07, 07-08]

# Tech tracking
tech-stack:
  added: [anyhow (error handling)]
  patterns: [KEY=value config parsing, XDG config layering, runtime abstraction]

key-files:
  created:
    - src/agent.rs
    - src/config.rs
    - src/container.rs
    - tests/config_test.rs
    - tests/container_test.rs
  modified:
    - src/lib.rs

key-decisions:
  - "Security validation rejects $() and backticks but allows $VAR expansion"
  - "Config layering: repo defaults + optional XDG user overrides"
  - "Runtime detection prioritizes podman, falls back to docker with warning"
  - "Support both new (klotho-*) and legacy (agent-session-*) image naming"

patterns-established:
  - "AgentConfig struct holds parsed config with validation"
  - "Runtime enum abstracts podman/docker commands"
  - "XDG_CONFIG_HOME/klotho preferred over legacy ~/.config/agent-session"

# Metrics
duration: 4min
completed: 2026-01-27
---

# Phase 07 Plan 02: Config Loading and Runtime Summary

**XDG-compliant config loading with command substitution protection and runtime abstraction supporting podman/docker with auto-detection**

## Performance

- **Duration:** 4 min
- **Started:** 2026-01-27T15:57:10Z
- **Completed:** 2026-01-27T16:01:28Z
- **Tasks:** 2
- **Files modified:** 8

## Accomplishments
- Agent config loading parses KEY=value format with security validation
- XDG config layering merges repo defaults with user overrides
- Container runtime abstraction handles podman and docker
- Runtime auto-detection with --runtime flag override support
- Both new (klotho-*) and legacy (agent-session-*) image naming supported

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement agent config loading** - `364169f` (feat)
2. **Task 2: Implement container runtime abstraction** - `0563780` (feat)

## Files Created/Modified
- `src/agent.rs` - AgentConfig struct with KEY=value parsing and security validation
- `src/config.rs` - XDG config home resolution and config layering functions
- `src/container.rs` - Runtime enum and container operations (status, start, stop, remove, list)
- `tests/config_test.rs` - Integration tests for config loading
- `tests/container_test.rs` - Integration tests for runtime detection
- `src/lib.rs` - Module exports updated

## Decisions Made

**Config security validation (Task 1)**
- Rejects command substitution ($() and backticks) to prevent code injection
- Allows variable expansion ($VAR) as safe for shell-sourceable configs
- Rationale: Command substitution executes during sourcing; variable expansion is passive

**XDG config layering (Task 1)**
- Priority: XDG_CONFIG_HOME/klotho > ~/.config/agent-session (legacy)
- Repo config required, user config optional
- Rationale: Follows Linux conventions, enables user customization without repo modification

**Runtime detection priority (Task 2)**
- Auto-detect tries podman first, then docker
- Shows warning when using docker ("podman is recommended for better rootless support")
- --runtime flag overrides auto-detection
- Rationale: Podman provides better rootless container support and is preferred

**Image naming support (Task 2)**
- Check new naming (klotho-agent:latest) first
- Fall back to legacy naming (agent-session-agent:latest) with notice
- Rationale: Smooth migration path from bash version to Rust version

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Ready for 07-03 (Session Management). Infrastructure complete:
- Config loading works for all agents
- Runtime detection handles both podman and docker
- Container abstraction provides operations needed by commands
- All tests pass

No blockers or concerns.

---
*Phase: 07-rust-rewrite*
*Completed: 2026-01-27*
