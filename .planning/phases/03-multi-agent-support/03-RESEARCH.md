# Phase 3: Multi-Agent Support - Research

**Researched:** 2026-01-26
**Domain:** Interactive CLI menus, container management, multi-agent configuration
**Confidence:** HIGH

## Summary

This phase adds interactive agent selection and support for OpenCode alongside Claude. The standard approach for bash menu selection is the built-in `select` construct, paired with `podman image exists` for build status detection. OpenCode installation closely mirrors Claude's architecture—both use curl-based installers, require API key management, support MCP servers via config files, and run in containerized environments.

The research reveals three key architectural points:
1. **Menu implementation** - Bash `select` provides numbered lists and input validation natively
2. **Build detection** - `podman image exists` returns exit codes for scriptable checks
3. **OpenCode parity** - Installation, config, and MCP setup follow similar patterns to Claude

**Primary recommendation:** Use bash `select` for the menu (native, simple, fits context decisions), `podman image exists` for build detection, and install OpenCode via curl script with uvx-based MCP servers configured in `~/.config/opencode/opencode.json`.

## Standard Stack

The established tools for this domain:

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| bash select | builtin | Interactive menu generation | POSIX-adjacent, no dependencies, built-in validation |
| podman image exists | latest | Image presence detection | Official Podman command for scripting |
| OpenCode installer | latest | OpenCode installation | Official installation method via curl script |
| uvx | latest (via uv) | MCP server execution | Standard Python tool runner, same as Claude uses |

### Supporting
| Tool | Version | Purpose | When to Use |
|------|---------|---------|-------------|
| read -p with ${var:-default} | builtin | Default value handling | Confirmations and simple prompts |
| podman build --target | latest | Multi-stage builds | Building specific agent images |
| OpenCode /connect | builtin | Provider API key setup | Initial OpenCode configuration |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| bash select | dialog/whiptail TUI | Context requires "simple numbered prompt, not fancy TUI" |
| Image manifest check | podman images -q | `image exists` is more explicit and scriptable |
| npm install -g | curl install script | Curl script is official, matches Claude installation pattern |

**Installation:**
```bash
# OpenCode (in Containerfile)
curl -fsSL https://opencode.ai/install | bash

# uv (for uvx MCP servers, in Containerfile)
curl -LsSf https://astral.sh/uv/install.sh | sh
```

## Architecture Patterns

### Recommended Project Structure
```
agent-session                    # Main entry script
config/agents/
  ├── claude/config.conf         # Existing
  └── opencode/config.conf       # New
scripts/
  ├── agent-menu.sh              # Interactive menu logic
  └── build-agent.sh             # Build confirmation and execution
```

### Pattern 1: Interactive Agent Menu with Default
**What:** Show numbered agent list, auto-select first (alphabetically) on empty input
**When to use:** No --agent flag provided
**Example:**
```bash
# Source: Bash select documentation + context decisions
agents=("claude" "opencode")

# Skip menu if only one agent
if [ ${#agents[@]} -eq 1 ]; then
    selected="${agents[0]}"
else
    PS3="Select agent (default: ${agents[0]}): "
    select choice in "${agents[@]}"; do
        if [[ -z "$REPLY" ]]; then
            # Empty input = default (first in list)
            selected="${agents[0]}"
            break
        elif [[ -n "$choice" ]]; then
            selected="$choice"
            break
        else
            echo "Invalid selection. Try again."
        fi
    done
fi
```

### Pattern 2: Build Status Detection
**What:** Check if image exists, prompt to build if needed
**When to use:** After agent selection, before container launch
**Example:**
```bash
# Source: podman image exists documentation
IMAGE_NAME="agent-session-${AGENT_TYPE}:latest"

if ! podman image exists "$IMAGE_NAME"; then
    echo "${AGENT_TYPE^} (not built)"
    read -p "Image not built. Build now? [y/N] " answer
    answer=${answer:-N}
    if [[ "$answer" =~ ^[Yy]$ ]]; then
        podman build --target "$AGENT_TYPE" -t "$IMAGE_NAME" .
    else
        echo "Cannot start session without built image."
        exit 1
    fi
else
    echo "${AGENT_TYPE^} (ready)"
fi
```

### Pattern 3: OpenCode MCP Server Configuration
**What:** Define MCP servers in `~/.config/opencode/opencode.json`
**When to use:** Container entrypoint merges config like Claude's `.claude` merge
**Example:**
```json
{
  "mcp": {
    "context7": {
      "type": "local",
      "command": ["uvx", "@upstash/context7-mcp"],
      "enabled": true
    },
    "serena": {
      "type": "local",
      "command": ["uvx", "--from", "git+https://github.com/oraios/serena", "serena", "start-mcp-server", "--context", "ide-assistant", "--project", "/workspace"],
      "enabled": true
    }
  }
}
```

### Anti-Patterns to Avoid
- **Fancy TUI libraries** - Context explicitly states "simple numbered prompt, not fancy TUI"
- **Build-time agent detection** - Agents configs must exist before build, not discovered dynamically
- **Hardcoded agent list** - Always scan `config/agents/` directory for agent configs
- **env var bypass of menu** - Context states "--agent flag is the only way to bypass menu"

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Numbered menu with validation | Loop with case statements | bash `select` construct | Built-in, handles numbering, re-prompts on invalid input |
| Image existence check | Parse `podman images` output | `podman image exists` | Returns clean exit codes (0=exists, 1=not), scriptable |
| Default input values | Custom input parsing | `read` with `${var:-default}` | Standard bash parameter expansion |
| MCP server management | Custom server launcher | OpenCode's mcp config | OpenCode handles discovery, lifecycle, stdio/HTTP transports |

**Key insight:** Bash provides `select` specifically for menus—don't reimplement numbering and validation. Podman provides `image exists` for scripts—don't parse human-readable output.

## Common Pitfalls

### Pitfall 1: select Loop Without Empty Input Handling
**What goes wrong:** User hits Enter expecting default, but select re-prompts forever
**Why it happens:** `select` sets `$choice` to empty on invalid input (including empty), doesn't auto-default
**How to avoid:** Check if `$REPLY` is empty inside select block, break with default value
**Warning signs:** Menu tests show "won't accept empty input" or "can't select default"

### Pitfall 2: Config/Auth File Locations Differ Per Agent
**What goes wrong:** Assuming all agents use `~/.claude.json` pattern, mounting wrong paths
**Why it happens:** OpenCode uses `~/.config/opencode/` and `~/.local/share/opencode/auth.json`, different from Claude
**How to avoid:** Agent configs should specify mount requirements; OpenCode needs both config and data directories
**Warning signs:** "API keys not found" in OpenCode despite configuration

### Pitfall 3: Building All Agents When Only One Needed
**What goes wrong:** `podman build` without `--target` rebuilds all stages, wastes time
**Why it happens:** Multi-stage Containerfiles build all stages by default
**How to avoid:** Always use `podman build --target <agent-name>` for agent-specific builds
**Warning signs:** Build logs show "Building stage: base", "Building stage: claude", "Building stage: opencode" when only one needed

### Pitfall 4: MCP Servers Not Available in Container
**What goes wrong:** MCP tools don't appear in OpenCode despite config
**Why it happens:** `uvx` not installed in image, or MCP config not mounted
**How to avoid:** Install `uv` in agent stage (provides `uvx`), mount `~/.config/opencode/` as config directory
**Warning signs:** OpenCode launches but `/mcp list` shows no servers

### Pitfall 5: Alphabetical Sort Depends on Locale
**What goes wrong:** Agent order changes based on system locale (affects default selection)
**Why it happens:** Shell sort behavior varies with `LC_COLLATE`
**How to avoid:** Use `LC_COLLATE=C sort` for consistent ASCII ordering
**Warning signs:** Different default agent on different systems

## Code Examples

Verified patterns from official sources and best practices:

### Alphabetical Agent Discovery
```bash
# Source: Standard Unix file listing
# Context: Default agent is first alphabetically

mapfile -t agents < <(
    find config/agents -mindepth 1 -maxdepth 1 -type d -printf '%f\n' \
    | LC_COLLATE=C sort
)

# First agent becomes default
DEFAULT_AGENT="${agents[0]}"
```

### Podman Build with Target and Build Args
```bash
# Source: Podman build documentation + Phase 2 decisions
# Note: Build args come from sourced agent config

# Source agent config to get build args
source "config/agents/${AGENT_TYPE}/config.conf"

podman build \
    --target "$AGENT_TYPE" \
    --build-arg AGENT_NAME="$AGENT_NAME" \
    --build-arg AGENT_INSTALL_CMD="$AGENT_INSTALL_CMD" \
    --build-arg AGENT_SHELL="$AGENT_SHELL" \
    --build-arg AGENT_LAUNCH_CMD="$AGENT_LAUNCH_CMD" \
    -t "agent-session-${AGENT_TYPE}:latest" \
    .
```

### OpenCode Config Mounting (Parallel to Claude)
```bash
# Source: OpenCode documentation + Phase 2 merge pattern
# Pattern: Mount separate config directory, entrypoint merges

OPTIONAL_MOUNTS=""

# Claude mounts (existing)
[[ -d "$HOME/.claude" ]] && \
    OPTIONAL_MOUNTS="$OPTIONAL_MOUNTS -v $HOME/.claude:/config/.claude:Z"

# OpenCode mounts (new - needs both config and auth)
[[ -d "$HOME/.config/opencode" ]] && \
    OPTIONAL_MOUNTS="$OPTIONAL_MOUNTS -v $HOME/.config/opencode:/config/opencode:Z"
[[ -d "$HOME/.local/share/opencode" ]] && \
    OPTIONAL_MOUNTS="$OPTIONAL_MOUNTS -v $HOME/.local/share/opencode:/home/agent/.local/share/opencode:Z"
```

### OpenCode Agent Config
```bash
# config/agents/opencode/config.conf
AGENT_NAME="opencode"
AGENT_DESCRIPTION="OpenCode AI coding agent"
AGENT_INSTALL_CMD="curl -fsSL https://opencode.ai/install | bash"
AGENT_LAUNCH_CMD="opencode"
AGENT_SHELL="/usr/bin/fish"
AGENT_ENV_VARS="PATH=/home/agent/.local/bin:\$PATH SHELL=/usr/bin/fish"
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Hardcoded agent selection | Config-driven agents | Phase 2 (2026-01) | New agents via config, not code |
| Docker-only | Podman-native commands | Phase 1 (2026-01) | `podman image exists` instead of parsing `docker images` |
| Single agent (Claude) | Multi-agent with menu | Phase 3 (2026-01) | Interactive selection, scripting via --agent |
| NPM MCP servers | uvx-based MCP servers | 2025-2026 | No npm in container, Python-based tooling |

**Deprecated/outdated:**
- GSD as "MCP server" - GSD is a CLI tool with slash commands, not an MCP server (installs via npx)
- `docker` commands in scripts - Project uses `podman` throughout

## Open Questions

Things that couldn't be fully resolved:

1. **GSD Installation for OpenCode**
   - What we know: GSD installs via `npx get-shit-done-cc --opencode`, adds slash commands to OpenCode
   - What's unclear: Whether OpenCode container needs Node.js for GSD, or if GSD is OpenCode-compatible at all
   - Recommendation: Test GSD installation in OpenCode container during implementation; may need Node.js added to base stage or OpenCode stage

2. **OpenCode API Key Setup Without Interactive TUI**
   - What we know: OpenCode uses `/connect` command for interactive API key setup, stores in `~/.local/share/opencode/auth.json`
   - What's unclear: How to pre-configure API keys non-interactively (for container rebuild without interaction)
   - Recommendation: Mount existing `auth.json` if present, document that users must run `/connect` in first session

3. **MCP Server Config Merge Strategy**
   - What we know: OpenCode loads config from `~/.config/opencode/opencode.json` (global) and `./opencode.json` (project)
   - What's unclear: Whether mounting `/config/opencode` and having entrypoint merge to `~/.config/opencode` follows same pattern as Claude
   - Recommendation: Test merge behavior during implementation; may need entrypoint logic to merge mounted config

## Sources

### Primary (HIGH confidence)
- [Podman image exists documentation](https://docs.podman.io/en/latest/markdown/podman-image-exists.1.html) - Exit code behavior
- [Bash select documentation](https://www.baeldung.com/linux/shell-script-simple-select-menu) - Menu patterns and validation
- [OpenCode official docs](https://opencode.ai/docs/) - Installation and configuration
- [OpenCode MCP servers documentation](https://opencode.ai/docs/mcp-servers/) - MCP server configuration format
- [OpenCode config documentation](https://opencode.ai/docs/config/) - Config file structure and locations
- [GitHub: OpenCode repository](https://github.com/anomalyco/opencode) - Official installation methods
- [GitHub: Context7 MCP server](https://github.com/upstash/context7) - Context7 installation
- [GitHub: Serena MCP server](https://github.com/oraios/serena) - Serena uvx installation
- [Serena documentation - Connecting Clients](https://oraios.github.io/serena/02-usage/030_clients.html) - uvx installation for Claude Code
- Phase 2 planning docs - Agent config architecture decisions

### Secondary (MEDIUM confidence)
- [OpenCode container setup blog posts](https://piotrnowicki.com/posts/2026-01-11/keeping-ai-agents-like-opencode-as-separate-environment-in-docker/) (January 2026) - Docker environment variables
- [OpenCode MCP with context optimization](https://composio.dev/blog/mcp-with-opencode) - MCP usage patterns
- [Bash read with default values](https://ricard.dev/how-to-have-default-values-with-read-in-bash/) - Input default patterns

### Tertiary (LOW confidence)
- [GSD announcement blog post](https://medium.com/@joe.njenga/i-tested-gsd-claude-code-meta-prompting-that-ships-faster-no-agile-bs-ca62aff18c04) - GSD as slash command tool (not MCP)
- Community discussions on OpenCode authentication - Security concerns about plaintext auth.json

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - Official Podman/Bash docs, OpenCode official installation
- Architecture: HIGH - Bash select is well-documented, OpenCode config structure is official
- Pitfalls: HIGH - Based on documented behavior and prior phase implementation patterns
- OpenCode MCP setup: MEDIUM - Extrapolated from Claude pattern, needs testing
- GSD installation: LOW - Unclear if GSD works with OpenCode in containers

**Research date:** 2026-01-26
**Valid until:** 2026-02-26 (30 days - relatively stable bash/podman APIs, OpenCode is fast-moving but installation stable)
