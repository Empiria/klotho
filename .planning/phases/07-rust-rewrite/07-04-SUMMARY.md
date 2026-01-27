---
phase: 07-rust-rewrite
plan: 04
subsystem: commands
tags: [rust, start-command, zellij, mounts, interactive-selection]

# Dependency graph
requires:
  - phase: 07-02
    provides: Config loading and container runtime abstraction
  - phase: 07-03
    provides: Session management with embedded resources
  - phase: 07-06
    provides: Build commands with auto-build capability
provides:
  - Start command creating sessions with proper mounts
  - Interactive agent selection when -a flag omitted
  - Auto-build prompt for missing images
  - Zellij session attachment with TTY
affects: [07-08]

# Tech tracking
tech-stack:
  added: []
  patterns: [interactive agent selection, auto-build on missing image, zellij attachment]

key-files:
  created:
    - src/commands/start.rs
  modified:
    - src/commands/mod.rs
    - src/cli.rs
    - src/main.rs

key-decisions:
  - "Agent parameter optional in CLI to distinguish explicit vs default selection"
  - "Interactive Select menu when no -a flag provided"
  - "Auto-build prompt (default No) when image missing"
  - "Support KLOTHO_* env vars with AGENT_SESSION_* legacy fallback"
  - "Optional mounts only added if directories exist"

patterns-established:
  - "Start command handles three states: running container (attach), stopped container (start then attach), no container (create then attach)"
  - "Zellij session detection via list-sessions with ANSI stripping"
  - "Environment variables SHELL and AGENT_LAUNCH_CMD passed to exec for agent initialization"

# Metrics
duration: 3min
completed: 2026-01-27
---

# Phase 07 Plan 04: Start Command Summary

**Complete start command with interactive selection, mounts, auto-build, and Zellij attachment**

## Performance

- **Duration:** 3 min
- **Started:** 2026-01-27T16:11:31Z
- **Completed:** 2026-01-27T16:14:22Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments
- Start command creates containers with project mounts and proper SELinux labels
- Interactive agent selection when -a flag not provided (uses dialoguer Select)
- Auto-build prompt when image missing (calls build::run_build)
- Zellij session attachment works for both create and attach scenarios
- Support for KLOTHO_KOB and KLOTHO_MOUNTS env vars with legacy fallbacks
- Optional directory mounts (~/.claude, ~/.config/opencode, ~/.config/zellij)
- Both new (klotho-session-*) and legacy container naming supported

## Task Commits

Each task was committed atomically:

1. **Task 1: Create commands module structure** - `53b73c7` (chore)
2. **Task 2: Implement start command** - `2f5ed3c` (feat)

## Files Created/Modified
- `src/commands/start.rs` - Complete start command implementation with interactive selection, mount handling, auto-build, and Zellij attachment
- `src/commands/mod.rs` - Added start module export
- `src/cli.rs` - Made agent parameter Option<String> for interactive selection
- `src/main.rs` - Wired up start command routing

## Decisions Made

**Interactive agent selection (Task 2)**
- Made CLI agent parameter optional (Option<String> instead of String with default)
- When None, show dialoguer Select menu with available agents
- Single agent skips menu, multiple agents show selection
- Rationale: Better UX than requiring -a flag, matches bash version behavior

**Auto-build on missing image (Task 2)**
- Call build::run_build when image_exists returns false
- Prompt user with dialoguer::Confirm (default: No)
- Error with helpful message if user declines
- Rationale: Prevents cryptic container runtime errors, guides users to build first

**Environment variable legacy support (Task 2)**
- Support KLOTHO_KOB with AGENT_SESSION_KOB fallback
- Support KLOTHO_MOUNTS with AGENT_SESSION_EXTRA_MOUNTS fallback
- Show deprecation notice when legacy vars used
- Rationale: Smooth migration path for existing users

**Optional mounts (Task 2)**
- Check PathBuf::exists() before adding mount for ~/.claude, ~/.config/opencode, ~/.config/zellij
- Always mount ~/.claude.json if exists
- Rationale: Avoids mount errors for directories that don't exist, preserves "optional" semantics

**Zellij session handling (Task 2)**
- Check if session exists via zellij list-sessions
- Strip ANSI codes for reliable parsing
- Create new session with -s flag, attach with attach command
- Fallback to exec shell if session dies
- Rationale: Robust session management that handles both new and existing sessions

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Ready for 07-08 (Integration Testing). Start command complete:
- Creates containers with all required mounts
- Interactive agent selection works
- Auto-build prompt guides users
- Zellij attachment functional
- Legacy naming and env vars supported

No blockers or concerns.

---
*Phase: 07-rust-rewrite*
*Completed: 2026-01-27*
