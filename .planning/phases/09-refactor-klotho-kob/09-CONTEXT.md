# Phase 9: Refactor KLOTHO_KOB - Context

**Gathered:** 2026-01-27
**Status:** Ready for planning

<domain>
## Phase Boundary

Rename KLOTHO_KOB to a self-explanatory name, fix the Rust implementation bug (wrong mount path), remove all legacy environment variable support, and document the feature properly. Also remove the deprecated bash script.

</domain>

<decisions>
## Implementation Decisions

### Naming
- Rename `KLOTHO_KOB` to `KLOTHO_LINKED_DIRS`
- Support multiple directories (colon-separated), matching KLOTHO_MOUNTS pattern
- Add CLI flag `--linked-dir` (repeatable) for per-command override
- Keep `KLOTHO_MOUNTS` name unchanged

### Bug fix
- Rust currently mounts at `/home/agent/.klotho` — this breaks symlinks
- Must mount at same path as host (e.g., `/home/user/external:/home/user/external:Z`)
- This matches the original bash behavior that made symlinks work

### Legacy removal
- Remove all `KLOTHO_KOB` support — clean break, no fallback
- Remove all `AGENT_SESSION_*` variable support (KOB, MOUNTS, EXTRA_MOUNTS)
- Delete the bash `klotho` script entirely — Rust version is canonical

### Claude's Discretion
- Documentation structure and wording for the README
- Error messages when linked directories don't exist
- Whether to validate symlinks actually point to linked dirs

</decisions>

<specifics>
## Specific Ideas

The feature's purpose: mount external directories so that symlinks in the workspace resolve correctly inside the container.

Use cases (generic, not team-specific):
- Shared tooling/configs across multiple projects
- Files that shouldn't be committed to the current repo but need to be accessible
- Team-shared resources linked into individual workspaces
- Planning directories that live outside client codebases

The symlinks themselves can be excluded from git via `.git/info/exclude`.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 09-refactor-klotho-kob*
*Context gathered: 2026-01-27*
