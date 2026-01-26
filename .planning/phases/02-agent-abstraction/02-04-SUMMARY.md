---
phase: 02-agent-abstraction
plan: 04
subsystem: infra
tags: [containerfile, agent-config, wrapper-scripts, config-driven]

# Dependency graph
requires:
  - phase: 02-01
    provides: Agent config format and validation
  - phase: 02-02
    provides: Agent-specific Containerfile stages
  - phase: 02-03
    provides: Config-driven orchestration
provides:
  - Dynamic wrapper script naming using AGENT_NAME
  - Clean config format with only used fields
  - Fully config-driven agent addition (no orchestration changes needed)
affects: [03-advanced-features, future-agent-definitions]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Wrapper script name derives from AGENT_NAME ARG in Containerfile"
    - "Config fields only include what's actually consumed"

key-files:
  created: []
  modified:
    - Containerfile
    - agent-session
    - scripts/build.sh
    - config/agents/claude/config.conf
    - docs/AGENTS.md

key-decisions:
  - "Dynamic wrapper script naming via AGENT_NAME ARG enables config-only agent addition"
  - "Remove unused AGENT_REQUIRED_MOUNTS to eliminate technical debt"

patterns-established:
  - "Wrapper script path: /home/agent/.local/bin/${AGENT_NAME}-session"
  - "Build ARG passing: --build-arg AGENT_NAME from build.sh"
  - "Config consumption: Only define fields that orchestration actually uses"

# Metrics
duration: 2min
completed: 2026-01-26
---

# Phase 2 Plan 4: Gap Closure Summary

**Dynamic wrapper script naming and dead config cleanup enable true config-only agent addition**

## Performance

- **Duration:** 2 min
- **Started:** 2026-01-26T16:47:47Z
- **Completed:** 2026-01-26T16:49:51Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments

- Eliminated hardcoded "claude-session" references blocking multi-agent support
- Wrapper script name now derived from AGENT_NAME at build time
- Removed unused AGENT_REQUIRED_MOUNTS field eliminating false documentation
- Adding new agents now requires only config file + Containerfile stage changes

## Task Commits

Each task was committed atomically:

1. **Task 1: Make wrapper script name dynamic** - `2a48150` (refactor)
   - Added AGENT_NAME ARG to Containerfile claude stage
   - Changed wrapper creation to use ${AGENT_NAME}-session pattern
   - Updated agent-session to construct wrapper path from AGENT_NAME config
   - Modified build.sh to pass AGENT_NAME to container build

2. **Task 2: Remove unused AGENT_REQUIRED_MOUNTS field** - `d1bd842` (refactor)
   - Removed from config/agents/claude/config.conf
   - Removed from docs/AGENTS.md Required Fields table
   - Removed from example config in documentation

## Files Created/Modified

- `Containerfile` - Added AGENT_NAME ARG, use ${AGENT_NAME}-session for wrapper script name
- `agent-session` - Use ${AGENT_NAME}-session for wrapper path (line 141)
- `scripts/build.sh` - Pass --build-arg AGENT_NAME to podman build
- `config/agents/claude/config.conf` - Removed AGENT_REQUIRED_MOUNTS (unused field)
- `docs/AGENTS.md` - Removed AGENT_REQUIRED_MOUNTS from Required Fields and example

## Decisions Made

**Dynamic wrapper naming via AGENT_NAME ARG**
- Rationale: When adding a new agent (e.g., "aider"), its wrapper becomes `aider-session` automatically. Without this, agent-session would try to execute hardcoded `claude-session`, breaking config-only goal.
- Implementation: ARG passed from build.sh → Containerfile, used in both wrapper creation and runtime path construction.

**Remove AGENT_REQUIRED_MOUNTS**
- Rationale: Field was documented as required but never consumed by any script. Creates false expectations and technical debt. If mount validation needed in future, add with actual implementation.
- Impact: Config format now contains only fields that orchestration actually uses.

## Deviations from Plan

None - gap closure plan executed exactly as written.

## Issues Encountered

None - straightforward refactoring with verification confirming both gaps closed.

## User Setup Required

None - internal infrastructure changes only.

## Next Phase Readiness

**Phase 2 (Agent Abstraction) complete:**
- ✅ Agent configs define installation, shell, launch commands
- ✅ Config validation prevents command injection
- ✅ agent-session uses --agent flag with config-driven orchestration
- ✅ Wrapper scripts dynamically named from AGENT_NAME
- ✅ Config format clean (only used fields)

**Phase 2 verification gaps closed:**
- Gap 1 (BLOCKER): No hardcoded claude-session in executable code
- Gap 2 (WARNING): No unused config fields

**Ready for Phase 3:** Advanced features (multi-repo support, better CLI UX)

---
*Phase: 02-agent-abstraction*
*Completed: 2026-01-26*
