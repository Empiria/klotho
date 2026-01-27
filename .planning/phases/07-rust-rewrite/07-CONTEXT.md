# Phase 7: Rust Rewrite - Context

**Gathered:** 2026-01-27
**Status:** Ready for planning

<domain>
## Phase Boundary

Rewrite the klotho CLI from bash (~855 lines) to Rust for maintainability and single-binary distribution. **Expanded scope:** Also add build and rebuild commands that don't exist in the current bash version.

</domain>

<decisions>
## Implementation Decisions

### Distribution & installation
- Support both GitHub releases and cargo install equally
- Pre-built binaries for all major platforms: Linux (x86_64, aarch64), macOS (x86_64, aarch64), Windows
- Fully static binaries using musl on Linux — no glibc dependencies
- Provide curl | sh installer script that detects platform and downloads correct binary

### Build command design
- Two separate commands: `klotho build` and `klotho rebuild` (rebuild = build --no-cache)
- Interactive agent selection when no argument provided (consistent with start command)
- Support building multiple agents: `--all` flag and multi-select in interactive mode
- Build progress: spinner with stage name, full podman output only on error

### Error & output UX
- Colored output with auto-detection (colors when terminal supports, plain when piped or NO_COLOR set)
- Simple exit codes: 0 for success, 1 for error
- No verbosity flags — single output level, keep it simple

### Podman interaction
- Shell out to podman CLI (not Rust bindings) — simpler, works with any podman version
- Support Docker as fallback via `--runtime` flag, podman is default
- Detection cascade:
  - Podman found → use it
  - No podman but docker found → warning, use docker
  - Neither found → error with platform-specific install instructions
- Embedded resources (Containerfile, default configs) compiled into binary via rust-embed
- User can override via ~/.config/klotho/ paths

### Claude's Discretion
- Error message formatting and suggestions
- Specific Rust crates for CLI parsing, colors, spinners
- Internal code architecture and module structure
- Config file parsing implementation

</decisions>

<specifics>
## Specific Ideas

- Single binary that "just works" — download and run, no dependencies
- Spinner with stage name during builds feels more polished than raw podman output
- Docker fallback with warning is pragmatic for users who haven't switched to podman

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 07-rust-rewrite*
*Context gathered: 2026-01-27*
