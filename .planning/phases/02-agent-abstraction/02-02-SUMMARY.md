---
phase: 02-agent-abstraction
plan: 02
subsystem: infra
tags: [podman, docker, containerfile, multi-stage-build, bash, config-validation, xdg]

# Dependency graph
requires:
  - phase: 02-01
    provides: "Shell-sourceable config format for agent definitions"
provides:
  - "Multi-stage Containerfile with base and claude stages"
  - "Config-driven build script with security validation"
  - "Working agent-session-claude:latest image"
affects: [02-03, runtime-implementation, agent-additions]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Multi-stage Docker builds with shared base stage"
    - "Build-time ARG injection from config files"
    - "XDG-style config layering (repo default + user override)"
    - "Shell config validation before sourcing"

key-files:
  created:
    - scripts/build.sh
  modified:
    - Containerfile

key-decisions:
  - "Reject only command substitution ($() and backticks) in validation, allow pipes in quoted strings"
  - "Use eval for AGENT_INSTALL_CMD in Containerfile to handle complex commands with pipes"
  - "Place uv installation in claude stage (agent-specific for MCP servers)"
  - "Keep base stage parameter-free, declare ARGs only in agent stages"

patterns-established:
  - "validate_config() checks for command substitution before sourcing config files"
  - "load_agent_config() implements XDG precedence: repo default first, then user override"
  - "validate_required_fields() ensures all critical config values are present"
  - "validate_stage_exists() fails early with helpful error listing available stages"

# Metrics
duration: 3min
completed: 2026-01-26
---

# Phase 02 Plan 02: Multi-Stage Container Build Summary

**Multi-stage Containerfile with shared base layer and config-driven build script enabling agent-specific image builds without modifying build logic**

## Performance

- **Duration:** 3 min
- **Started:** 2026-01-26T16:00:51Z
- **Completed:** 2026-01-26T16:03:48Z
- **Tasks:** 3
- **Files modified:** 2 (1 created, 1 modified)

## Accomplishments
- Refactored monolithic Containerfile into multi-stage build with base and claude stages
- Created build.sh script with config validation, XDG layering, and stage verification
- Successfully built agent-session-claude:latest image from config

## Task Commits

Each task was committed atomically:

1. **Task 1: Refactor Containerfile to multi-stage build** - `5a036eb` (refactor)
2. **Task 2: Create build script with validation** - `ca03fde` (feat)
3. **Task 3: Build and test Claude image** - (no commit - verification only)

## Files Created/Modified
- `Containerfile` - Refactored to multi-stage build with base stage (common tools) and claude stage (agent-specific via ARG injection)
- `scripts/build.sh` - Config-driven build script with validation, XDG layering, required field checks, and stage existence verification

## Decisions Made

**1. Config validation approach**
- **Decision:** Reject only command substitution ($() and backticks), allow pipes/semicolons in quoted strings
- **Rationale:** Research validation was overly conservative. Pipes inside quotes don't execute during sourcing - they're just string content. Command substitution DOES execute during sourcing even in quotes, so that's the real security risk.
- **Impact:** Allows legitimate install commands like `curl ... | bash` in config values

**2. uv placement**
- **Decision:** Install uv in claude stage, not base stage
- **Rationale:** uv is specifically for Python MCP servers, which is Claude-specific functionality. Future agents may not need it. Keep base minimal.
- **Impact:** Base stage remains generic, agent stages add their specific tooling

**3. ARG scope**
- **Decision:** Declare ARGs only in agent-specific stages, keep base stage parameter-free
- **Rationale:** Base stage is shared and shouldn't have agent-specific parameters. ARG values are only needed in agent stages where they're used.
- **Impact:** Cleaner separation, base can be built/cached without knowing about agents

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed config validation regex**
- **Found during:** Task 2 (initial build script testing)
- **Issue:** Research validation pattern rejected pipes in quoted strings, but research example config contains pipe in AGENT_INSTALL_CMD. Validation was contradictory and too strict.
- **Fix:** Changed validation from rejecting `'|;|&|$()` to rejecting only `'$()` (command substitution). Pipes/semicolons inside double-quoted strings are safe during sourcing - they're just string content.
- **Files modified:** scripts/build.sh
- **Verification:** Build script now accepts config with `curl ... | bash` in quoted string, correctly rejects `$(whoami)` test
- **Committed in:** ca03fde (part of Task 2 commit)

---

**Total deviations:** 1 auto-fixed (1 bug fix)
**Impact on plan:** Bug fix was necessary to match research examples and correct security model. No scope creep.

## Issues Encountered
None - all tasks completed as planned after fixing validation bug.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Multi-stage Containerfile complete with base and claude stages
- Build script validates configs and builds agent images
- agent-session-claude:latest image successfully built
- Ready for Plan 03: Run script implementation
- Future agent additions can add new stages without modifying build logic

---
*Phase: 02-agent-abstraction*
*Completed: 2026-01-26*
