# Phase 2: Agent Abstraction - Context

**Gathered:** 2026-01-26
**Status:** Ready for planning

<domain>
## Phase Boundary

Extract agent definitions into config-driven architecture. Claude agent becomes defined via config file with installation, paths, and commands. Adding a new agent requires only a config file and Containerfile stage, not orchestration logic changes.

</domain>

<decisions>
## Implementation Decisions

### Config file format
- Shell-sourceable format (KEY=value), can be sourced directly in bash scripts
- Variable expansion only ($VAR and ${VAR}), no command substitution
- Multi-line values use space-separated strings parsed with read or IFS
- Minimal metadata: AGENT_NAME and AGENT_DESCRIPTION, no versioning

### Directory structure
- Repo defaults in config/agents/, user can override in ~/.config/agent-session/agents/
- Subdirectory per agent: agents/claude/config.conf, agents/opencode/config.conf
- Only config.conf recognized for now — keep it simple, expand later if needed

### Containerfile integration
- Build-time ARG injection: pass config values as --build-arg, Containerfile uses ARG directives
- Single Containerfile with multi-stage builds: base stage shared, agent stages branch, build with --target=<agent>
- Stage names match config directory name: agents/claude/ → stage named 'claude'
- Build script validates that a Containerfile stage exists for each agent config before running podman build

### Required config fields
- Comprehensive set required: name, description, install command, launch command, required mounts, env vars
- No capability flags yet — keep simple, add when Phase 3 needs them
- Validation happens during build only, no standalone validator command

### Documentation
- AGENTS.md reference doc listing all fields, their purpose, and examples
- No template config file — reference doc is the source of truth

### Claude's Discretion
- Exact ARG names and mapping conventions
- Error message formatting for validation failures
- Order of fields in config files

</decisions>

<specifics>
## Specific Ideas

- XDG-style layering: repo provides defaults, user overrides in ~/.config/agent-session/agents/
- Build script should fail early with clear error if stage missing, before podman runs

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 02-agent-abstraction*
*Context gathered: 2026-01-26*
