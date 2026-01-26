# Architecture Patterns for Multi-Agent CLI

**Domain:** Container-based multi-agent session orchestrator
**Researched:** 2026-01-26
**Confidence:** HIGH

## Executive Summary

The agent-session tool follows a **container-based session orchestrator** pattern where a host-side CLI manages containerized agent runtimes. To add multi-agent support, introduce an **agent abstraction layer** that separates agent-specific concerns (installation, commands, config) from the orchestration logic. The recommended approach uses **agent definition files** (simple config format) plus **containerfile build arguments** to create agent-specific container images.

This pattern aligns with 2026 industry trends: OpenCode's agent switching model, Google ADK's YAML-based agent configs, and asdf's extensible command structure via directories. The key insight: **don't make containers multi-agent, make orchestration agent-aware**.

## Current Architecture (As-Built)

### Three-Layer Model

**Host Layer** (`agent-session` script):
- Container lifecycle management via Podman
- Session naming and reattachment logic
- Volume mount construction
- Single-agent assumption (hardcoded Claude Code)

**Container Layer** (`Containerfile`):
- Debian bookworm-slim base
- Hardcoded Claude Code installation via curl | bash
- Fish shell + Zellij + Starship
- Single entrypoint path

**Initialization Layer** (`entrypoint.sh`):
- Config directory symlinking
- GSD plugin installation (Claude-specific)
- Zellij config setup

### Current Limitations

1. **Agent hardcoding**: Claude Code installation baked into Containerfile
2. **No abstraction**: Agent-specific logic scattered across files
3. **Single config path**: Mounts `~/.claude` only
4. **Fixed wrapper**: `claude-session` script hardcoded

## Recommended Architecture: Agent Abstraction Pattern

### Core Principle

**Separate concerns:**
- **Orchestration** (host script): Container lifecycle, session management, path mounting
- **Agent definition** (config files): Installation commands, config paths, invocation
- **Container build** (multi-stage): Agent-specific image variants

### Component Structure

```
agent-session/                  # Project root
├── agent-session               # Host orchestration script (modified)
├── agents/                     # NEW: Agent definitions directory
│   ├── claude.conf             # Claude Code definition
│   ├── opencode.conf           # OpenCode definition
│   └── [future].conf           # Extensible pattern
├── Containerfile               # Multi-stage with ARG-based branching
├── entrypoint.sh               # Agent-agnostic initialization
└── .planning/
```

### Agent Definition Format

Use **simple Bash-sourceable config files** (not YAML/JSON) for maximum compatibility with existing Bash orchestration.

**Structure** (`agents/claude.conf`):
```bash
# Agent metadata
AGENT_NAME="claude"
AGENT_DISPLAY_NAME="Claude Code"
AGENT_DESCRIPTION="Anthropic's official Claude coding agent"

# Container build
AGENT_IMAGE_TAG="claude-agent"
AGENT_INSTALL_COMMANDS='
    curl -fsSL https://claude.ai/install.sh | bash
'

# Runtime configuration
AGENT_CONFIG_HOST_PATH="$HOME/.claude"
AGENT_CONFIG_CONTAINER_PATH="/config/.claude"
AGENT_DATA_HOST_PATH="$HOME/.local/share/claude"
AGENT_DATA_CONTAINER_PATH="/home/agent/.local/share/claude"

# Invocation
AGENT_COMMAND="claude --dangerously-skip-permissions"
AGENT_SHELL_WRAPPER="claude-session"

# Initialization hooks
AGENT_INIT_COMMANDS='
    # Install GSD plugin
    if [[ ! -f ~/.claude/get-shit-done/VERSION ]]; then
        npx -y get-shit-done-cc@latest --claude --global
    fi
'
```

**Structure** (`agents/opencode.conf`):
```bash
AGENT_NAME="opencode"
AGENT_DISPLAY_NAME="OpenCode"
AGENT_DESCRIPTION="Open-source multi-model coding agent (SST)"

AGENT_IMAGE_TAG="opencode-agent"
AGENT_INSTALL_COMMANDS='
    # OpenCode uses Go binary
    curl -fsSL https://opencode.sh/install.sh | bash
'

AGENT_CONFIG_HOST_PATH="$HOME/.config/opencode"
AGENT_CONFIG_CONTAINER_PATH="/config/opencode"
AGENT_DATA_HOST_PATH="$HOME/.local/share/opencode"
AGENT_DATA_CONTAINER_PATH="/home/agent/.local/share/opencode"

AGENT_COMMAND="opencode"
AGENT_SHELL_WRAPPER="opencode-session"

AGENT_INIT_COMMANDS='
    # OpenCode auto-configures on first run
    true
'
```

### Benefits of This Format

1. **Bash-native**: No external parsers (yq, jq) required
2. **Simple sourcing**: `source agents/$AGENT_NAME.conf` loads all variables
3. **Extensible**: Add new fields without breaking existing configs
4. **Multi-line support**: Heredocs work naturally in Bash strings
5. **Comments**: Standard Bash comments for documentation

## Integration Points

### 1. Host Script Modifications (`agent-session`)

**Add agent selection:**
```bash
# NEW: Agent selection
AGENT_NAME="claude"  # Default

parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            -a|--agent)
                AGENT_NAME="$2"
                shift 2
                ;;
            --list-agents)
                list_agents
                exit 0
                ;;
            # ... existing args ...
        esac
    done
}

# NEW: Load agent definition
AGENT_DEF="$(dirname "$0")/agents/${AGENT_NAME}.conf"
if [[ ! -f "$AGENT_DEF" ]]; then
    echo "error: unknown agent: $AGENT_NAME" >&2
    echo "Available agents:" >&2
    ls "$(dirname "$0")/agents" | sed 's/\.conf$//'
    exit 1
fi
source "$AGENT_DEF"

# Container name now includes agent
CONTAINER_NAME="agent-${AGENT_NAME}-${SESSION_NAME}"
```

**Use dynamic mounts:**
```bash
# Build volume mounts from agent definition
AGENT_MOUNTS="
    -v ${AGENT_CONFIG_HOST_PATH}:${AGENT_CONFIG_CONTAINER_PATH}:Z
    -v ${AGENT_DATA_HOST_PATH}:${AGENT_DATA_CONTAINER_PATH}:Z
"

# Use agent-specific image
IMAGE_NAME="$AGENT_IMAGE_TAG"
```

**Interactive selection (optional):**
```bash
# If fzf available and no agent specified, offer menu
if command -v fzf >/dev/null 2>&1 && [[ -z "$AGENT_NAME" ]]; then
    AGENT_NAME=$(ls agents/*.conf | sed 's|.*/||; s|\.conf$||' | \
        fzf --prompt="Select agent: " --height=10)
fi
```

### 2. Containerfile Multi-Stage Pattern

Use **build arguments** to conditionally install agents:

```dockerfile
FROM debian:bookworm-slim AS base

# Common dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    curl git ca-certificates fish nodejs npm \
    && rm -rf /var/lib/apt/lists/*

# Install common tools
RUN curl -sSL https://github.com/zellij-org/zellij/releases/latest/download/zellij-x86_64-unknown-linux-musl.tar.gz | tar xz -C /usr/local/bin
RUN curl -sSL https://starship.rs/install.sh | sh -s -- -y
RUN useradd -m -s /usr/bin/fish -u 1000 agent

USER agent
RUN mkdir -p ~/.local/bin

# Configure fish
RUN mkdir -p ~/.config/fish && printf '%s\n' \
    'set -g fish_greeting' \
    'starship init fish | source' \
    > ~/.config/fish/config.fish

# Install uv (common for MCP servers)
RUN curl -LsSf https://astral.sh/uv/install.sh | sh

# --- Agent-specific stages ---

FROM base AS agent-claude
RUN curl -fsSL https://claude.ai/install.sh | bash
RUN printf '%s\n' \
    '#!/usr/bin/env fish' \
    'claude --dangerously-skip-permissions' \
    'exec fish' \
    > ~/.local/bin/agent-session && chmod +x ~/.local/bin/agent-session

FROM base AS agent-opencode
RUN curl -fsSL https://opencode.sh/install.sh | bash
RUN printf '%s\n' \
    '#!/usr/bin/env fish' \
    'opencode' \
    'exec fish' \
    > ~/.local/bin/agent-session && chmod +x ~/.local/bin/agent-session

# --- Final stage selection ---

ARG AGENT=claude
FROM agent-${AGENT} AS final

ENV PATH="/home/agent/.local/bin:$PATH"
ENV SHELL="/usr/bin/fish"
WORKDIR /workspace
ENTRYPOINT ["/entrypoint.sh"]
CMD ["zellij"]
```

**Build usage:**
```bash
# Build Claude image
podman build --build-arg AGENT=claude -t claude-agent .

# Build OpenCode image
podman build --build-arg AGENT=opencode -t opencode-agent .
```

### 3. Entrypoint Generalization

Make `entrypoint.sh` agent-agnostic by accepting environment variables:

```bash
#!/bin/bash
set -e

WORKDIR="$(pwd)"

# Generic config setup from ENV vars
if [[ -n "$AGENT_CONFIG_MOUNT" && -d "$AGENT_CONFIG_MOUNT" ]]; then
    CONFIG_DIR="${AGENT_CONFIG_HOME:-$HOME/.config/agent}"
    mkdir -p "$CONFIG_DIR"
    for item in "$AGENT_CONFIG_MOUNT"/* "$AGENT_CONFIG_MOUNT"/.*; do
        [[ "$(basename "$item")" == "." || "$(basename "$item")" == ".." ]] && continue
        [[ ! -e "$item" && ! -L "$item" ]] && continue

        name=$(basename "$item")
        if [[ -L "$item" && ! -e "$item" ]]; then
            mkdir -p "$CONFIG_DIR/$name"
        else
            ln -s "$AGENT_CONFIG_MOUNT/$name" "$CONFIG_DIR/$name"
        fi
    done
fi

# Run agent-specific init if provided
if [[ -n "$AGENT_INIT_SCRIPT" ]]; then
    eval "$AGENT_INIT_SCRIPT"
fi

# Detect agent type (for backward compatibility)
if command -v claude &>/dev/null; then
    echo "Claude Code $(claude --version) ready"
elif command -v opencode &>/dev/null; then
    echo "OpenCode ready"
fi

cd "$WORKDIR"
exec "$@"
```

**Pass init commands via environment:**
```bash
podman run \
    -e AGENT_CONFIG_MOUNT="/config/.agent" \
    -e AGENT_CONFIG_HOME="$HOME/.agent" \
    -e AGENT_INIT_SCRIPT="npx -y get-shit-done..." \
    ...
```

## Build Order: Migration Path

### Phase 1: Extract Agent Definition (No Breaking Changes)

**Changes:**
1. Create `agents/` directory
2. Create `agents/claude.conf` with current values
3. Modify `agent-session` to source `agents/claude.conf`
4. Test: Existing behavior unchanged

**Validation:**
- `agent-session` works identically
- No new flags or behavior yet

### Phase 2: Add Multi-Stage Containerfile

**Changes:**
1. Refactor Containerfile into base + agent-claude stages
2. Add ARG AGENT selection
3. Update build script to pass `--build-arg AGENT=claude`
4. Test: Same image as before, different structure

**Validation:**
- Built image is functionally identical
- `podman build --build-arg AGENT=claude` succeeds

### Phase 3: Add Second Agent (OpenCode)

**Changes:**
1. Create `agents/opencode.conf`
2. Add `FROM base AS agent-opencode` stage
3. Add `-a/--agent` flag to `agent-session`
4. Update container naming to include agent: `agent-${AGENT}-${SESSION}`
5. Test: Can create OpenCode session

**Validation:**
- `agent-session -a claude` works (existing)
- `agent-session -a opencode` works (new)
- Sessions don't conflict (different containers)

### Phase 4: Interactive Selection

**Changes:**
1. Add fzf-based agent selection if no `-a` flag
2. Add `--list-agents` flag
3. Add agent description display
4. Update help text

**Validation:**
- Interactive menu appears when agent not specified
- `--list-agents` shows available agents
- Selection passes through to container creation

### Phase 5: Generalize Entrypoint (Optional)

**Changes:**
1. Refactor `entrypoint.sh` to use ENV vars
2. Pass agent config via `-e` flags in `agent-session`
3. Test: Both agents work with generic entrypoint

**Validation:**
- Claude and OpenCode both initialize correctly
- Config symlinking works for both agents

## Alternative Patterns Considered

### Pattern: Single Mega-Container

**Approach:** Install all agents in one image, select at runtime.

**Rejected because:**
- Image bloat: 500MB+ for agents user never uses
- Dependency conflicts: Different Node versions, conflicting binaries
- Slower builds: Every agent change rebuilds everything
- Unclear ownership: Who maintains this image?

**When to reconsider:** If agents become plugins (npm packages) with identical runtimes.

### Pattern: Agent-Specific Scripts

**Approach:** Separate scripts like `claude-session`, `opencode-session`.

**Rejected because:**
- Code duplication: 90% of orchestration is identical
- User confusion: Which script do I use?
- Maintenance burden: Fix bug in 5 places
- Doesn't solve container problem

**When to reconsider:** If agents require fundamentally different orchestration (e.g., one is remote-only).

### Pattern: YAML/JSON Config

**Approach:** Define agents in structured data files, parse with yq/jq.

**Rejected because:**
- External dependency: Requires yq/jq on host
- Complexity: Bash manipulation of YAML is awkward
- Overkill: Agents have ~10 fields, not 100
- Harder to debug: Parsing failures obscure

**When to reconsider:** If config becomes complex (nested structures, conditionals, 20+ fields).

### Pattern: Plugin Directory with Executables

**Approach:** `agents/claude/install.sh`, `agents/claude/run.sh`, etc.

**Partially adopted:**
- We use directory structure (`agents/`)
- But configs not executables (sourced .conf files)

**Why hybrid:**
- Executables: More flexible but harder to compose
- Configs: Less flexible but easy to read/validate
- Best of both: Bash-sourceable configs with executable hooks if needed

## Architectural Anti-Patterns to Avoid

### Anti-Pattern 1: Config-Driven Container Selection

**What:** Store agent choice in `~/.agent-session.conf`, read on every invocation.

**Why bad:**
- Hidden state: User forgets config, confused by behavior
- Session conflicts: Different sessions need different agents
- Breaks explicitness: `agent-session -n work` doesn't show which agent

**Instead:** Make agent choice explicit per session (`-a` flag or interactive select).

### Anti-Pattern 2: Runtime Agent Switching

**What:** Build container with all agents, switch with `switch-agent` command inside.

**Why bad:**
- Image bloat (see rejected patterns)
- Complex state management: Which agent is "active"?
- Zellij session confusion: History mixed between agents
- Unclear memory model: Which agent "remembers" what?

**Instead:** One session = one agent. To switch, start new session.

### Anti-Pattern 3: Agent Version Pinning

**What:** Hardcode versions in agent.conf (`AGENT_VERSION="2.1.3"`).

**Why bad:**
- Stale quickly: Claude updates weekly
- User confusion: "Why is my local newer than container?"
- Maintenance burden: Update configs constantly
- Breaks auto-updates: Agents manage their own versions

**Instead:** Install latest, let agent's native update mechanism handle versions.

### Anti-Pattern 4: Shared Config Directories

**What:** Mount `~/.config/agents/` and symlink to agent-specific paths.

**Why bad:**
- Permission conflicts: UID mapping issues with nested paths
- Breaks agent expectations: Tools look for `~/.claude`, not `~/.config/agents/claude`
- Debugging nightmare: "Why isn't my config loading?"
- No benefit: Bind mounts cost nothing

**Instead:** Mount each agent's native config location directly.

## Scalability Considerations

### Adding New Agents (Future)

**Effort to add agent #3, #4, #5:**

1. **Create config file** (~5 minutes):
   - Copy `agents/template.conf`
   - Fill in installation command, paths, wrapper command
   - Test source parsing: `source agents/newagent.conf && echo $AGENT_NAME`

2. **Add Containerfile stage** (~10 minutes):
   - Copy existing `FROM base AS agent-X` block
   - Update installation command from config
   - Test build: `podman build --build-arg AGENT=newagent -t newagent-agent .`

3. **Test session creation** (~5 minutes):
   - `agent-session -a newagent -n test ~/projects/demo`
   - Verify agent launches, config mounted, commands work

**Total: ~20 minutes per agent** (after first two are working).

### At 5 Agents

| Concern | Impact | Mitigation |
|---------|--------|------------|
| Containerfile size | ~50 lines per agent = 250 lines total | Acceptable; multi-stage keeps stages isolated |
| Build time | N/A - only selected agent builds | BuildKit skips unused branches |
| Maintenance | Each agent has 1 config + 1 stage | Standardized format makes bulk updates easy |
| Testing | Test each agent independently | Automated: `for agent in agents/*.conf; do test_agent $agent; done` |

### At 20+ Agents

**Refactor trigger:** When agent-specific logic becomes complex (conditionals, dependencies).

**Migration path:**
1. Move to `agents/AGENT_NAME/` directories
2. Split config (`agent.conf`) from hooks (`install.sh`, `init.sh`)
3. Add agent discovery via manifest (`agents/registry.json`)
4. Consider dynamic Containerfile generation

## Key Abstractions

### Agent Definition

**Purpose:** Encapsulate all agent-specific configuration in one place.

**Boundary:** Everything that differs between Claude, OpenCode, Future Agent X.

**Examples:**
- Installation command
- Config directory paths
- Invocation command
- Initialization hooks

**Pattern:** Bash-sourceable config files in `agents/` directory.

### Session Identity

**Purpose:** Uniquely identify a running agent session.

**Boundary:** Combination of agent type + session name.

**Examples:**
- `agent-claude-frontend` (container name)
- `agent-opencode-backend` (container name)
- `default` (Zellij session name, unchanged)

**Pattern:** `agent-${AGENT_NAME}-${SESSION_NAME}` for containers, `${SESSION_NAME}` for Zellij (within container).

### Build Stage

**Purpose:** Isolate agent installation into container image layer.

**Boundary:** FROM base to final agent-ready image.

**Examples:**
- `agent-claude` stage: base + Claude installation
- `agent-opencode` stage: base + OpenCode installation

**Pattern:** Multi-stage Dockerfile with ARG-based final stage selection.

## Error Handling Considerations

### Agent Not Found

**Scenario:** User specifies `-a nonexistent`.

**Strategy:**
```bash
if [[ ! -f "agents/${AGENT_NAME}.conf" ]]; then
    echo "error: unknown agent: $AGENT_NAME" >&2
    echo "Available agents:" >&2
    ls agents/*.conf | sed 's|.*/||; s|\.conf$||' | sed 's/^/  - /'
    exit 1
fi
```

### Image Not Built

**Scenario:** Agent config exists but image not built.

**Strategy:**
```bash
if ! podman image exists "$AGENT_IMAGE_TAG"; then
    echo "error: image $AGENT_IMAGE_TAG not found" >&2
    echo "Build it with: podman build --build-arg AGENT=$AGENT_NAME -t $AGENT_IMAGE_TAG ." >&2
    exit 1
fi
```

### Config Path Missing

**Scenario:** Agent expects `~/.claude` but user doesn't have it.

**Strategy:**
```bash
if [[ ! -e "$AGENT_CONFIG_HOST_PATH" ]]; then
    echo "warning: $AGENT_CONFIG_HOST_PATH does not exist" >&2
    echo "Creating it now (agent will initialize defaults)..." >&2
    mkdir -p "$AGENT_CONFIG_HOST_PATH"
fi
```

### Invalid Agent Config

**Scenario:** Required field missing from agent.conf.

**Strategy:**
```bash
validate_agent_config() {
    local required=(AGENT_NAME AGENT_IMAGE_TAG AGENT_COMMAND)
    for var in "${required[@]}"; do
        if [[ -z "${!var}" ]]; then
            echo "error: $AGENT_DEF missing required field: $var" >&2
            exit 1
        fi
    done
}
```

## Cross-Cutting Concerns

### Agent Discovery

**Mechanism:** Filesystem scanning of `agents/*.conf` files.

**Command:** `agent-session --list-agents` shows all available.

**Interactive:** fzf menu built from discovered agents.

### Backward Compatibility

**Constraint:** Existing sessions must continue working after migration.

**Strategy:**
- Default to `claude` if no `-a` specified
- Maintain `agent-${SESSION_NAME}` container naming for default agent
- Environment variable override: `AGENT_SESSION_DEFAULT_AGENT=opencode`

### Documentation Generation

**Opportunity:** Generate help text from agent configs.

**Implementation:**
```bash
show_agents() {
    echo "Available agents:"
    for conf in agents/*.conf; do
        source "$conf"
        printf "  %-12s %s\n" "$AGENT_NAME" "$AGENT_DESCRIPTION"
    done
}
```

### Testing Strategy

**Unit tests:** Validate agent config parsing.

**Integration tests:** Build each agent image, create session, verify agent runs.

**Regression tests:** Existing Claude-only workflow continues working.

## Sources

### HIGH Confidence (Official Documentation)

- [Docker Multi-Stage Builds](https://docs.docker.com/build/building/multi-stage/) - Containerfile conditional patterns
- [Advanced Multi-Stage Build Patterns](https://medium.com/@tonistiigi/advanced-multi-stage-build-patterns-6f741b852fae) - BuildKit branching with ARG
- [asdf Plugin Creation Guide](https://asdf-vm.com/plugins/create.html) - Command directory patterns
- [Google Agent Development Kit: Agent Config](https://google.github.io/adk-docs/agents/config/) - YAML-based agent definition structure
- [fzf Interactive Menus](https://ctrl-c.club/~lettuce/fzf-tui.html) - Building selection interfaces in Bash

### MEDIUM Confidence (Verified Best Practices)

- [AWS CLI Agent Orchestrator](https://aws.amazon.com/blogs/opensource/introducing-cli-agent-orchestrator-transforming-developer-cli-tools-into-a-multi-agent-powerhouse/) - Multi-agent orchestration patterns
- [OpenCode vs Claude Code Architecture](https://www.builder.io/blog/opencode-vs-claude-code) - Client/server vs CLI architecture comparison
- [Agent Design Patterns (2026)](https://rlancemartin.github.io/2026/01/09/agent_design/) - Progressive disclosure for context management
- [Vercel Bash Tool](https://vercel.com/blog/how-to-build-agents-with-filesystems-and-bash) - Filesystem-based agent architectures
- [Oh-My-Bash Plugin System](https://github.com/ohmybash/oh-my-bash) - Configuration-driven plugin loading

### Implementation References

- [OpenCode Agent Configuration](https://deepwiki.com/anomalyco/opencode/5.1-agent-configuration) - Real-world multi-agent config cascade
- [OpenCode Custom Commands](https://deepwiki.com/anomalyco/opencode/8.4-custom-commands-and-agents) - Extensibility patterns
- [Container Orchestration Tools (2026)](https://spacelift.io/blog/container-orchestration-tools) - Runtime selection patterns

---

*Architecture research for multi-agent support: 2026-01-26*
