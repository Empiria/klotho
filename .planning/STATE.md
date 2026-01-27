# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-01-26)

**Core value:** Consistent, reproducible agent environments that enable seamless handoff between people and agents through committed artifacts.
**Current focus:** Phase 7 - Rust Rewrite (Complete)

## Current Position

Phase: 9 of 9 (Refactor KLOTHO_KOB)
Plan: 1 of 2 in current phase
Status: In progress
Last activity: 2026-01-27 - Completed 09-01-PLAN.md

Progress: [████████████████████░] 74% (20/27 milestone plans)

## Performance Metrics

**Velocity:**
- Total plans completed: 20
- Average duration: 2.7 min
- Total execution time: 0.97 hours

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 01-release-audit | 3 | 18min | 6min |
| 02-agent-abstraction | 4 | 8min | 2min |
| 03-multi-agent-support | 3 | 4min | 1.3min |
| 04-session-management | 2 | 3min | 1.5min |
| 05-documentation | 2 | 4.4min | 2.2min |
| 06-rename-to-klotho | 3 | 3min | 1min |
| 07-rust-rewrite | 8 | 26min | 3.25min |
| 09-refactor-klotho-kob | 1 | 1min | 1min |

**Recent Trend:**
- Phase 9 in progress: 1/2 plans complete. Added CLI flag infrastructure for KLOTHO_LINKED_DIRS feature.

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
| 02-04 | Dynamic wrapper script naming via AGENT_NAME ARG | Enables config-only agent addition; new agent wrapper is automatically named correctly |
| 02-04 | Remove unused AGENT_REQUIRED_MOUNTS field | Config format contains only fields actually consumed; eliminates technical debt |
| 03-02 | Use bash select menu for interactive agent selection | Standard bash pattern for numbered menus with default on empty input |
| 03-02 | Display build status inline with agent name | Clear visual feedback: "Agent (ready)" vs "Agent (not built)" |
| 03-02 | Skip menu if only one agent configured | Streamlined UX when no choice is needed |
| 03-02 | Prompt to build before starting session | Better UX than cryptic podman error when image missing |
| 03-02 | Default No for build prompt | Safe default requiring explicit opt-in to trigger build |
| 03-01 | OpenCode MCP config excludes GSD | Uncertain GSD compatibility with OpenCode per research; can add later if needed |
| 03-01 | Follow exact Claude pattern for OpenCode stage | Consistency over optimization for second agent proving abstraction |
| 03-03 | Mount OpenCode config/auth following Claude pattern | Consistency across agents makes system predictable and maintainable |
| 03-03 | Bundle MCP config as fallback via /tmp mount | First-time users need working MCP servers without manual config |
| 05-01 | Concepts section placed after quick start | Avoid cognitive overload before first success; users anxious to see tool work first |
| 05-01 | Quick start demonstrates agent selection | Show both -a flag and interactive menu, not just default Claude |
| 05-02 | Use collapsible <details> sections for command reference | Progressive disclosure keeps quick start visible without scrolling through walls of text |
| 05-02 | Keep troubleshooting NOT collapsed | Users in error states need immediate access to solutions without extra clicks |
| 05-02 | Link to PREREQUISITES.md from README | Detailed installation instructions available via reference while keeping README concise |
| 07-01 | Use clap derive API for ergonomic argument parsing | More concise than builder API, better compile-time validation, matches bash version's interface exactly |
| 07-01 | Add global --runtime flag for Docker fallback | Plan specifies Docker fallback support; global flag ensures all commands can respect runtime override |
| 07-01 | Include new build/rebuild commands with --all flag | Rust version provides opportunity to add container build commands not in bash version |
| 07-01 | Stub implementations print args and use todo!() | Establishes command routing infrastructure while deferring implementation to later plans |
| 07-02 | Security validation rejects $() and backticks but allows $VAR expansion | Command substitution executes during sourcing; variable expansion is passive and safe |
| 07-02 | Config layering: repo defaults + optional XDG user overrides | Follows Linux conventions, enables user customization without repo modification |
| 07-02 | Runtime detection prioritizes podman, falls back to docker with warning | Podman provides better rootless container support and is preferred |
| 07-02 | Support both new (klotho-*) and legacy (agent-session-*) image naming | Smooth migration path from bash version to Rust version |
| 07-03 | Use rust-embed derive macro for compile-time resource embedding | Compiles resources directly into binary, no runtime IO for resource access |
| 07-03 | Extract to /tmp/klotho-build for builds | Container builds need actual files on disk, temp directory provides clean isolated context |
| 07-03 | Auto-detect development vs production mode | Check for local config/ directory to decide whether to use embedded resources |
| 07-03 | Force-add opencode.json despite gitignore | File needs to be embedded in binary even though it's gitignored in config/ |
| 07-05 | Stop command is idempotent | Stopping already-stopped container should succeed silently; container module handles this |
| 07-05 | Restart extracts agent type from container name | Agent type needed for config loading; parsing container name provides it without user input |
| 07-05 | Ls parses both new and legacy naming patterns | During migration both patterns exist; rfind splits at last hyphen for correct parsing |
| 07-05 | Rm prevents removal of running containers | Safety measure with helpful error message showing "klotho stop <name>" command |
| 07-06 | Use indicatif spinner with build step extraction for progress feedback | Long-running builds need feedback; spinner with step names provides clear progress without overwhelming output |
| 07-06 | Support both embedded and local build contexts | Development mode (local files) vs production mode (embedded resources) need different resource access patterns |
| 07-06 | Interactive multi-select when no agents specified | Better UX than error message; allows easy batch building of multiple agents |
| 07-06 | Validate Containerfile contains target stage before building | Fail fast with clear error message instead of cryptic container build failure |
| 07-04 | Agent parameter optional in CLI for interactive selection | Distinguishes explicit -a flag from default, enables interactive menu when not specified |
| 07-04 | Auto-build prompt when image missing | Prevents cryptic errors, guides users to build with dialoguer::Confirm (default No) |
| 07-04 | Support KLOTHO_* env vars with legacy fallback | KLOTHO_KOB and KLOTHO_MOUNTS preferred over AGENT_SESSION_* with deprecation notices |
| 07-04 | Optional mounts only added if directories exist | Checks PathBuf::exists() before mounting ~/.claude, ~/.config/opencode, ~/.config/zellij |
| 07-07 | Use cross for Linux musl builds | Static musl binaries provide better portability across Linux distributions |
| 07-07 | Generate SHA256 checksums for all binaries | Security best practice to verify download integrity |
| 07-07 | Wrap installer in main() function | Safety measure prevents partial execution if curl pipe interrupted |
| 07-07 | Support both curl and wget in installer | Maximizes compatibility across different systems |
| 07-07 | Warn when install directory not in PATH | Better UX - users know immediately if they need to update shell profile |
| 09-01 | Use Vec<String> for repeatable --linked-dir flag | clap automatically handles collection of repeated flags |
| 09-01 | Use .. pattern in match to ignore linked_dirs field | Flag added to CLI interface but not yet consumed by start command until plan 09-02 |

### Pending Todos

[From .planning/todos/pending/ - ideas captured during sessions]

None yet.

### Blockers/Concerns

[Issues that affect future work]

None yet.

## Roadmap Evolution

- Phase 6 added: Rename to Klotho (rebrand project and CLI from agent-session to klotho)
- Phase 7 added: Rust Rewrite (migrate bash script to Rust for maintainability and single-binary distribution)
- Phase 8 added: Docs Cleanup
- Phase 9 added: Refactor KLOTHO_KOB

## Session Continuity

Last session: 2026-01-27
Stopped at: Completed 09-01-PLAN.md
Resume file: None

**Phase 9 In Progress:** Refactoring KLOTHO_KOB environment variable. Plan 09-01 complete: Added --linked-dir CLI flag infrastructure and removed deprecated bash script. Ready for plan 09-02 to wire flag to start command.
