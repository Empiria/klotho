---
phase: 02-agent-abstraction
plan: 03
subsystem: infra
tags: [config, orchestration, shell, podman, docker, session-management]

# Dependency graph
requires:
  - phase: 02-01
    provides: "Shell-sourceable config format for agent definitions"
  - phase: 02-02
    provides: "Multi-stage Containerfile and config-driven build script"
provides:
  - "Config-driven agent-session orchestration script"
  - "Agent-agnostic entrypoint with conditional Claude logic"
  - "End-to-end verified working Claude sessions with config architecture"
affects: [agent-additions, multi-agent-support, session-management]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Config-driven session orchestration with --agent flag"
    - "Agent-agnostic container initialization with conditional agent-specific behavior"
    - "XDG-style config loading in runtime scripts"

key-files:
  created: []
  modified:
    - agent-session
    - entrypoint.sh

key-decisions:
  - "Default --agent to 'claude' for backward compatibility"
  - "Use agent-session-<agent>:latest image naming pattern"
  - "Merge mounted .claude config rather than replace when container dir exists"
  - "Make entrypoint agent-agnostic with conditional Claude behavior based on binary presence"

patterns-established:
  - "Runtime orchestration pattern: Load config early, use config values for image/shell/commands"
  - "Container initialization pattern: Generic setup + conditional agent-specific setup based on detection"
  - "Config merge pattern: Process mounted configs always, skip items that exist (prefer container versions)"

# Metrics
duration: 1min
completed: 2026-01-26
---

# Phase 2 Plan 3: Config-Driven Orchestration Summary

**Config-driven session orchestration with agent-agnostic entrypoint enabling multi-agent support through config files alone**

## Performance

- **Duration:** 1 min
- **Started:** 2026-01-26T16:27:48Z
- **Completed:** 2026-01-26T16:28:42Z
- **Tasks:** 3 (2 tasks + 1 checkpoint)
- **Files modified:** 2

## Accomplishments
- Updated agent-session to load agent config and use config values for image name, shell, and launch command
- Made entrypoint.sh agent-agnostic with conditional Claude-specific behavior
- Fixed config mounting bug discovered during end-to-end verification
- Verified complete Claude session lifecycle works with new architecture

## Task Commits

Each task was committed atomically:

1. **Task 1: Update agent-session to load agent config** - `bbea2d7` (feat)
2. **Task 2: Make entrypoint.sh agent-agnostic** - `66f8c1f` (refactor)
3. **Task 3: End-to-end verification checkpoint** - (user verified, bug fix committed separately)

**Bug fix discovered during verification:** `68ee89c` (fix)
**Plan metadata:** (pending - this commit)

## Files Created/Modified
- `agent-session` - Config-driven orchestration with --agent flag, XDG config loading, dynamic image name from config
- `entrypoint.sh` - Agent-agnostic initialization with conditional Claude logic (GSD install, version output)

## Decisions Made

**1. Backward compatibility via default agent**
- **Decision:** Default --agent to "claude" when not specified
- **Rationale:** Existing usage `./agent-session -n foo` should continue working without requiring --agent flag
- **Impact:** Users don't need to change existing workflows

**2. Image naming pattern**
- **Decision:** Use `agent-session-<agent>:latest` pattern instead of `claude-agent`
- **Rationale:** Consistent naming across all agents, makes agent type clear from image name
- **Impact:** Requires one-time image rebuild with build.sh

**3. Config merge strategy for .claude directory**
- **Decision:** Always process mounted config, but skip items that already exist in container
- **Rationale:** Claude Code creates ~/.claude during image build, so "only if not exists" check fails. Merging allows both container and mounted configs to coexist.
- **Impact:** Container runtime directories take precedence, user configs add to them

**4. Agent detection for conditional behavior**
- **Decision:** Use `command -v claude` to detect Claude, fall back to generic messages
- **Rationale:** Entrypoint can be agent-agnostic while still providing nice Claude experience
- **Impact:** Future agents get generic startup, Claude gets version output and GSD installation

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed .claude config mounting when directory exists**
- **Found during:** Task 3 (end-to-end verification checkpoint)
- **Issue:** entrypoint.sh had condition `if [[ -d /config/.claude && ! -d ~/.claude ]]` which failed because Claude Code creates ~/.claude during image build. Symlinks were never created, so mounted credentials didn't work.
- **Fix:** Changed condition to `if [[ -d /config/.claude ]]` (always process) and added skip logic `[[ -e "$target" || -L "$target" ]] && continue` to avoid overwriting container files. This merges mounted config with existing container config rather than replacing it.
- **Files modified:** entrypoint.sh
- **Verification:** End-to-end test passed - Claude Code started with proper authentication, credentials successfully mounted
- **Committed in:** 68ee89c (separate fix commit after verification)

---

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** Bug fix was critical for functionality - mounted credentials wouldn't work without it. Discovered during planned verification checkpoint, fixed immediately.

## Issues Encountered

None - tasks executed smoothly, bug was discovered during verification phase as expected.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

**Phase 02 Complete:** Agent abstraction architecture fully implemented.

**What's ready:**
- Config format defined and documented (AGENTS.md)
- Multi-stage Containerfile with base + agent stages
- Config-driven build script with validation
- Config-driven runtime orchestration
- Agent-agnostic container initialization
- End-to-end verified working with Claude

**Adding new agents now requires:**
1. Create `config/agents/<name>/config.conf`
2. Add `FROM base AS <name>` stage to Containerfile
3. Run `./scripts/build.sh <name>`
4. Use `./agent-session --agent <name> -n session-name`

**No changes needed to:**
- build.sh (reads any config)
- agent-session (reads any config)
- entrypoint.sh (works with any agent)

**Next phase opportunities:**
- Test with second agent (aider, continue, etc.)
- Add agent discovery (list available agents)
- Add config validation CLI command

---
*Phase: 02-agent-abstraction*
*Completed: 2026-01-26*
