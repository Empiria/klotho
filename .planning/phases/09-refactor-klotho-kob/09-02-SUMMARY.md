---
phase: 09-refactor-klotho-kob
plan: 02
subsystem: cli
tags: [rust, environment-variables, mounts, symlinks, klotho]

# Dependency graph
requires:
  - phase: 09-refactor-klotho-kob
    plan: 01
    provides: --linked-dir CLI flag infrastructure
provides:
  - KLOTHO_LINKED_DIRS environment variable support
  - Correct mount path logic (host path → same container path)
  - Clean codebase without legacy environment variables
affects: [09-03]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Colon-separated environment variable parsing for directory lists"
    - "Same-path mounting for symlink resolution (canonical:canonical:Z)"

key-files:
  created: []
  modified:
    - src/commands/start.rs
    - src/main.rs
    - README.md

key-decisions:
  - "Mount linked directories at canonical host path for symlink resolution"
  - "Parse KLOTHO_LINKED_DIRS as colon-separated (Unix PATH convention)"
  - "Merge CLI flags with environment variable (CLI extends env var)"
  - "Remove all legacy environment variable support in one clean break"
  - "Warn and skip non-existent directories instead of failing"

patterns-established:
  - "Canonicalize paths before mounting to ensure consistent resolution"
  - "Deduplicate directory lists before mounting (sort + dedup)"
  - "Environment variables parsed once, CLI flags extend values"

# Metrics
duration: 1.4min
completed: 2026-01-27
---

# Phase 09 Plan 02: KLOTHO_LINKED_DIRS Implementation Summary

**Fixed linked directories mount bug and completed KLOTHO_KOB refactoring by implementing correct same-path mounting and removing all legacy environment variable support**

## Performance

- **Duration:** 1.4 min
- **Started:** 2026-01-27T17:12:32Z
- **Completed:** 2026-01-27T17:13:57Z
- **Tasks:** 2
- **Files modified:** 3 (2 Rust source files, 1 documentation file)

## Accomplishments

- Linked directories mounted at canonical host path (not `/home/agent/.klotho`)
- KLOTHO_LINKED_DIRS environment variable parsed as colon-separated paths
- CLI --linked-dir flags merged with environment variable values
- All legacy environment variable support removed (KLOTHO_KOB, AGENT_SESSION_KOB, AGENT_SESSION_EXTRA_MOUNTS)
- README documents KLOTHO_LINKED_DIRS with clear examples and use cases
- Symlinks in workspaces now resolve correctly inside containers

## Task Commits

Each task was committed atomically:

1. **Task 1: Refactor mount logic for linked directories** - `0bb3ccc` (feat)
2. **Task 2: Update README documentation** - `164136d` (docs)

## Files Created/Modified

- `src/commands/start.rs` - Implemented KLOTHO_LINKED_DIRS parsing and correct mount path logic, removed all legacy env var support
- `src/main.rs` - Updated start command invocation to pass linked_dirs parameter
- `README.md` - Documented KLOTHO_LINKED_DIRS environment variable and --linked-dir CLI flag with examples

## Technical Details

### Mount Path Fix

**Before (bug):** Directories were mounted to `/home/agent/.klotho` via KLOTHO_KOB
**After (correct):** Directories mounted at canonical host path for symlink resolution

Example:
```rust
// Host: /home/user/shared-tools (canonical path)
// Container: /home/user/shared-tools (same path)
// Workspace symlink: ln -s /home/user/shared-tools/tool ./tool
// Result: Symlink resolves correctly inside container
```

### Environment Variable Parsing

```bash
# Colon-separated like Unix PATH
export KLOTHO_LINKED_DIRS="/home/user/shared-tools:/home/user/team-configs"
```

Implementation:
- Split on `:` delimiter
- Trim whitespace
- Deduplicate after merging with CLI flags
- Canonicalize to resolve `.` and `..` and symlinks
- Skip non-existent directories with warning

### Legacy Variables Removed

| Variable | Replacement | Notes |
|----------|-------------|-------|
| KLOTHO_KOB | KLOTHO_LINKED_DIRS | Different mount strategy - same path vs /home/agent/.klotho |
| AGENT_SESSION_KOB | KLOTHO_LINKED_DIRS | Legacy name from pre-Klotho era |
| AGENT_SESSION_EXTRA_MOUNTS | KLOTHO_MOUNTS | Already migrated in phase 7, legacy fallback removed |

## Decisions Made

1. **Mount at canonical host path** - Required for symlink resolution; container must see same absolute paths as host
2. **Colon-separated format** - Follows Unix PATH convention, familiar to users
3. **Merge CLI flags with env var** - CLI flags extend environment variable values (not replace)
4. **Remove all legacy support** - Clean break; deprecated variables had warnings in phase 7
5. **Warn on non-existent directories** - Better UX than failing; typos in config don't break workflow

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None - straightforward refactoring with clear specifications from research phase.

## User Setup Required

None - feature is fully implemented and documented. Users who need symlink resolution can now:

1. Export `KLOTHO_LINKED_DIRS` environment variable, or
2. Use `--linked-dir` CLI flag (repeatable), or
3. Combine both approaches

## Next Phase Readiness

Phase 09 complete - all KLOTHO_KOB refactoring finished:
- Plan 09-01: Added CLI infrastructure
- Plan 09-02: Implemented feature and removed legacy code

Codebase is clean, feature is documented, and symlinks work correctly in containers.

---
*Phase: 09-refactor-klotho-kob*
*Completed: 2026-01-27*
