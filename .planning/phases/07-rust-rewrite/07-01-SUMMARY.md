---
phase: 07-rust-rewrite
plan: 01
subsystem: cli
tags: [rust, clap, cargo]

# Dependency graph
requires:
  - phase: 06-rename-to-klotho
    provides: Final CLI interface and command structure
provides:
  - Rust project scaffold with Cargo.toml and all dependencies
  - Clap-based CLI structure matching bash implementation
  - Command routing infrastructure with stub implementations
affects: [07-02, 07-03, 07-04, 07-05, 07-06]

# Tech tracking
tech-stack:
  added: [clap, anyhow, indicatif, owo-colors, rust-embed, serde, toml, dialoguer]
  patterns: [clap derive API for CLI, anyhow for error handling, stub implementations with todo!()]

key-files:
  created: [Cargo.toml, src/cli.rs, src/main.rs, src/lib.rs, .gitignore]
  modified: []

key-decisions:
  - "Use clap derive API for ergonomic argument parsing matching bash version"
  - "Add global --runtime flag for Docker fallback (default: auto for auto-detection)"
  - "Include new build/rebuild commands with --all flag not in bash version"
  - "Stub implementations print args and use todo!() for later implementation"

patterns-established:
  - "Command dispatch in main.rs matches on Commands enum"
  - "CLI definitions in src/cli.rs use #[derive(Parser)] and #[derive(Subcommand)]"
  - "Global flags defined at Cli struct level"

# Metrics
duration: 1min
completed: 2026-01-27
---

# Phase 07 Plan 01: Initialize Rust CLI Foundation Summary

**Rust project with clap-based CLI structure, global --runtime flag, and 7 commands matching bash version (start, stop, restart, ls, rm, build, rebuild)**

## Performance

- **Duration:** 1 min
- **Started:** 2026-01-27T15:52:43Z
- **Completed:** 2026-01-27T15:54:40Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments
- Cargo project initialized with all required dependencies (clap, anyhow, indicatif, owo-colors, rust-embed, serde, toml, dialoguer)
- Complete CLI structure with 7 commands matching bash version interface
- Global --runtime flag for Docker fallback support
- All commands have proper help text and argument parsing

## Task Commits

Each task was committed atomically:

1. **Task 1: Initialize Cargo project with dependencies** - `810d48c` (chore)
2. **Task 2: Create CLI structure with clap derive** - `a0a9163` (feat)

## Files Created/Modified
- `Cargo.toml` - Project manifest with all dependencies (clap, anyhow, indicatif, owo-colors, rust-embed, serde, toml, dialoguer)
- `.gitignore` - Added Rust artifacts (/target/, Cargo.lock)
- `src/cli.rs` - Clap derive structs defining all commands and arguments
- `src/main.rs` - Entry point with command dispatch and stub implementations
- `src/lib.rs` - Module re-exports

## Decisions Made

**1. Use clap derive API for ergonomic argument parsing**
- Rationale: More concise than builder API, better compile-time validation, matches bash version's interface exactly

**2. Add global --runtime flag for Docker fallback (default: "auto")**
- Rationale: Plan specifies Docker fallback support; global flag ensures all commands can respect runtime override

**3. Include new build/rebuild commands with --all flag**
- Rationale: Rust version provides opportunity to add container build commands not in bash version

**4. Stub implementations print args and use todo!()**
- Rationale: Establishes command routing infrastructure while deferring implementation to later plans

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None - straightforward initialization and CLI structure definition.

## User Setup Required

None - no external service configuration required. This is foundational infrastructure only.

## Next Phase Readiness

**Ready for 07-02 (Runtime Detection):**
- CLI argument parsing complete
- Global --runtime flag available for runtime override
- Cargo project compiles without errors

**Ready for 07-03+ (Command Implementations):**
- Command routing infrastructure in place
- All command signatures defined with proper types
- Help text matches bash version

**No blockers identified.**

---
*Phase: 07-rust-rewrite*
*Completed: 2026-01-27*
