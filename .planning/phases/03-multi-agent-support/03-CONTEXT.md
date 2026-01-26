# Phase 3: Multi-Agent Support - Context

**Gathered:** 2026-01-26
**Status:** Ready for planning

<domain>
## Phase Boundary

Users can select and run multiple agent types interactively or via flags. OpenCode agent runs alongside Claude in separate containers. Interactive menu appears when no --agent flag specified. Both agents install their dependencies correctly and launch without conflicts.

</domain>

<decisions>
## Implementation Decisions

### Interactive menu
- Numbered list format: `1. Claude  2. OpenCode`
- Name only, no descriptions in menu
- Enter without input selects default (first in list)
- Invalid input re-shows menu with hint, doesn't exit

### Default behavior
- Always show menu when no --agent flag specified
- --agent flag is the only way to bypass menu (no env var)
- Skip menu if only one agent configured (auto-select)
- Default agent is first in alphabetical order

### Agent presentation
- List agents alphabetically
- Show build status in parentheses: `Claude (ready)` or `OpenCode (not built)`
- Unbuilt agent selection prompts: "Image not built. Build now? [y/N]"

### OpenCode configuration
- Installation method: Claude's discretion (research best approach)
- Needs API keys and config directory mounted (similar to Claude)
- Same Zellij layout as Claude for consistent experience
- Must have serena, context7, and gsd MCP servers installed and enabled

### Claude's Discretion
- OpenCode installation method (go install vs binary download)
- Exact prompt text for build confirmation
- Menu styling details
- Error message wording

</decisions>

<specifics>
## Specific Ideas

- Menu should feel like a simple numbered prompt, not a fancy TUI
- OpenCode should have the same MCP tooling as Claude: serena, context7, gsd

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 03-multi-agent-support*
*Context gathered: 2026-01-26*
