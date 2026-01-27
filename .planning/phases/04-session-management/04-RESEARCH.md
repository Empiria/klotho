# Phase 4: Session Management - Research

**Researched:** 2026-01-27
**Domain:** Bash CLI subcommand pattern, Podman container lifecycle, terminal UI
**Confidence:** HIGH

## Summary

This phase implements session lifecycle commands (stop, restart, list, remove) as subcommands of the existing `agent-session` script. The standard approach for bash subcommand CLIs uses a dispatch pattern where the first positional argument selects the function to execute. Container state management is handled entirely by Podman commands (stop, start, rm, ps) with straightforward exit code handling.

The research reveals three key architectural points:
1. **Subcommand dispatch** - Use a case statement on `$1` to route to handler functions (e.g., `cmd_stop`, `cmd_ls`)
2. **Podman is the state database** - No custom state tracking needed; `podman ps -a` queries all container state
3. **Container naming convention** - Existing `${AGENT_TYPE}-${SESSION_NAME}` pattern allows filtering by `name=claude-*|opencode-*`

**Primary recommendation:** Refactor the existing `agent-session` script into subcommand structure where current behavior becomes the `start` subcommand. Use Podman's native filtering and exit codes for all state queries. Colors via ANSI escape sequences with reset codes.

## Standard Stack

The established tools for this domain:

### Core
| Tool | Version | Purpose | Why Standard |
|------|---------|---------|--------------|
| Podman stop | 4.x+ | Stop container gracefully (SIGTERM then SIGKILL) | Official command, handles cleanup |
| Podman start | 4.x+ | Restart stopped container | Preserves container state and mounts |
| Podman rm | 4.x+ | Remove stopped container | Cleans up container and anonymous volumes |
| Podman ps | 4.x+ | Query container state | Supports filtering, custom format output |
| Bash case | builtin | Subcommand dispatch | Standard pattern for multi-command CLIs |
| ANSI escape codes | universal | Terminal colors | Supported by all modern terminals |

### Supporting
| Tool | Version | Purpose | When to Use |
|------|---------|---------|-------------|
| podman ps --filter | 4.x+ | Find agent sessions by name prefix | Listing only agent-session containers |
| podman ps --format | 4.x+ | Custom output formatting | Building ls output table |
| read -p | builtin | User prompts | Confirmation for rm command |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| ANSI codes | tput | ANSI is simpler, tput is more portable to exotic terminals |
| Case dispatch | Subshell scripts | Case dispatch keeps single file, easier to maintain |
| Direct format | JSON output | Context decided "human output only, no --json flag" |

**No new installation required** - all tools are already available (Podman, Bash builtins).

## Architecture Patterns

### Recommended Script Structure
```
agent-session                 # Main script with subcommand dispatch
  cmd_start()                 # Current script behavior (create/attach)
  cmd_stop()                  # Stop container
  cmd_restart()               # Start stopped container + attach
  cmd_ls()                    # List all sessions
  cmd_rm()                    # Remove stopped container
  show_help()                 # Help text with subcommand docs
```

### Pattern 1: Subcommand Dispatch
**What:** Route first argument to handler function via case statement
**When to use:** Main entry point of script
**Example:**
```bash
# Source: Bash CLI best practices (clig.dev, progrium/bashstyle)
main() {
    case "${1:-}" in
        start)
            shift
            cmd_start "$@"
            ;;
        stop)
            shift
            cmd_stop "$@"
            ;;
        restart)
            shift
            cmd_restart "$@"
            ;;
        ls|list)
            shift
            cmd_ls "$@"
            ;;
        rm|remove)
            shift
            cmd_rm "$@"
            ;;
        -h|--help|help|"")
            show_help
            exit 0
            ;;
        *)
            echo "error: unknown command: $1" >&2
            echo "Run 'agent-session --help' for usage." >&2
            exit 1
            ;;
    esac
}

main "$@"
```

### Pattern 2: Session Name Resolution
**What:** Resolve session name from argument or current directory
**When to use:** Commands that operate on a session (stop, restart, rm)
**Example:**
```bash
# Source: Existing agent-session pattern + CONTEXT.md decisions
resolve_session_name() {
    local name="${1:-}"
    if [[ -z "$name" ]]; then
        # Default to current directory name as session identifier
        name=$(basename "$(pwd)")
    fi
    echo "$name"
}

# Usage in command:
cmd_stop() {
    local session_name
    session_name=$(resolve_session_name "${1:-}")
    # ... proceed with stop
}
```

### Pattern 3: Find Container by Session Name
**What:** Search all agent containers for matching session name
**When to use:** Session commands that need container name
**Example:**
```bash
# Source: Podman ps filter documentation
find_container() {
    local session_name="$1"
    local container

    # Search for container matching any agent type with this session name
    # Container naming: ${AGENT_TYPE}-${SESSION_NAME}
    container=$(podman ps -a --format "{{.Names}}" | grep -E "^(claude|opencode)-${session_name}$" | head -1)

    if [[ -z "$container" ]]; then
        echo "error: Session '$session_name' not found. Use 'agent-session ls' to see sessions." >&2
        return 1
    fi

    echo "$container"
}
```

### Pattern 4: Colored Status Output
**What:** Use ANSI codes for running/stopped status colors
**When to use:** ls command output
**Example:**
```bash
# Source: ANSI escape codes best practices
# CONTEXT.md: green=running, red=stopped

# Define colors at script start
readonly COLOR_GREEN='\033[0;32m'
readonly COLOR_RED='\033[0;31m'
readonly COLOR_RESET='\033[0m'

format_status() {
    local status="$1"
    if [[ "$status" == "running" ]]; then
        echo -e "${COLOR_GREEN}running${COLOR_RESET}"
    else
        echo -e "${COLOR_RED}stopped${COLOR_RESET}"
    fi
}
```

### Pattern 5: Confirmation Prompt with Force Flag
**What:** Prompt before destructive action, skip with --force
**When to use:** rm command
**Example:**
```bash
# Source: Bash confirmation best practices
# CONTEXT.md: rm prompts by default, --force/-f skips

cmd_rm() {
    local force=false
    local session_name=""

    while [[ $# -gt 0 ]]; do
        case "$1" in
            -f|--force)
                force=true
                shift
                ;;
            *)
                session_name="$1"
                shift
                ;;
        esac
    done

    session_name=$(resolve_session_name "$session_name")
    local container
    container=$(find_container "$session_name") || exit 1

    # Check if running - cannot rm running container
    if podman ps --format "{{.Names}}" | grep -q "^${container}$"; then
        echo "error: Cannot remove running session. Stop it first." >&2
        exit 1
    fi

    if [[ "$force" != true ]]; then
        read -p "Remove session '$session_name'? [y/N] " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            echo "Cancelled."
            exit 0
        fi
    fi

    podman rm "$container" >/dev/null
    echo "Removed: $session_name"
}
```

### Anti-Patterns to Avoid
- **Parsing podman ps human output** - Use `--format "{{.Field}}"` for scriptable extraction
- **Custom state files** - Podman already tracks container state; don't duplicate
- **Force-removing running containers** - Context says "Cannot remove running session. Stop it first."
- **Interactive prompts on stop** - Context says "stop does not prompt (stopping is reversible)"
- **Bulk operations** - Context says "No bulk remove command - drop to podman for that use case"

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Container state tracking | JSON/SQLite database | `podman ps -a` | Container runtime is the authoritative source |
| Finding containers by pattern | Loop through all containers | `podman ps --filter name=pattern` | Native filtering is faster, more reliable |
| Graceful shutdown | Custom signal handling | `podman stop` | Handles SIGTERM, grace period, SIGKILL fallback |
| Container cleanup | rm volumes, network | `podman rm` | Handles all cleanup correctly |
| Terminal colors | Custom escape code parser | Raw ANSI codes | Simple, universal, well-documented |

**Key insight:** Podman commands have exit codes and output formats designed for scripting. The `--format` and `--filter` flags exist specifically to avoid parsing human-readable output.

## Common Pitfalls

### Pitfall 1: Stopping Already-Stopped Container
**What goes wrong:** `podman stop` on stopped container returns exit code 0 but prints a warning
**Why it happens:** Podman is idempotent for stop - it succeeds if container ends up stopped
**How to avoid:** Accept this behavior (matches context requirement for idempotent stop). Suppress stderr or check state first if warning is undesirable
**Warning signs:** Warning messages in stop output for already-stopped sessions

### Pitfall 2: Container Not Found vs Wrong Name Pattern
**What goes wrong:** Session exists but isn't found because agent type prefix doesn't match
**Why it happens:** Container naming is `${AGENT_TYPE}-${SESSION_NAME}`, need to check all agent types
**How to avoid:** Use grep with alternation pattern: `grep -E "^(claude|opencode)-${session_name}$"`
**Warning signs:** "Session not found" error when container clearly exists in `podman ps -a`

### Pitfall 3: Restart Doesn't Reattach Zellij
**What goes wrong:** `podman start` brings container up but user isn't attached to terminal
**Why it happens:** `podman start` without `-a` runs detached; Zellij session inside needs reattachment
**How to avoid:** After `podman start`, call the existing `attach_zellij()` function
**Warning signs:** Restart succeeds but user sees no output, has to run start again

### Pitfall 4: Colors Break in Non-Terminal Context
**What goes wrong:** Escape codes appear as literal characters in logs or pipes
**Why it happens:** Output redirected to file or piped to another command
**How to avoid:** Check `[[ -t 1 ]]` (stdout is terminal) before adding colors, or use `--no-color` flag if needed
**Warning signs:** Escape codes visible in log files, broken output in CI

### Pitfall 5: rm on Running Container Silently Fails
**What goes wrong:** User thinks session was removed but it's still running
**Why it happens:** `podman rm` on running container fails with exit code 2
**How to avoid:** Check running state explicitly before rm, provide clear error message
**Warning signs:** Session reappears in ls after rm

### Pitfall 6: Exit Code Propagation with set -e
**What goes wrong:** Script exits unexpectedly when command returns non-zero
**Why it happens:** `grep` returns 1 when no match found, `podman stop` may return non-zero in edge cases
**How to avoid:** Use `|| true` for expected failures, or handle exit codes explicitly
**Warning signs:** Script dies mid-execution on "normal" operations

## Code Examples

Verified patterns from official sources:

### Stop Command Implementation
```bash
# Source: Podman stop docs + CONTEXT.md decisions
cmd_stop() {
    local session_name
    session_name=$(resolve_session_name "${1:-}")

    local container
    container=$(find_container "$session_name") || exit 1

    # Stop is idempotent - stopping already-stopped is success
    podman stop "$container" >/dev/null 2>&1 || true
    echo "Stopped: $session_name"
}
```

### List Command with Colored Status
```bash
# Source: Podman ps docs + CONTEXT.md (colors: green=running, red=stopped)
cmd_ls() {
    local containers
    containers=$(podman ps -a --format "{{.Names}}|{{.Status}}" \
        | grep -E "^(claude|opencode)-" || true)

    if [[ -z "$containers" ]]; then
        echo "No sessions found"
        return 0
    fi

    # Header
    printf "%-20s %-12s %s\n" "NAME" "AGENT" "STATUS"

    while IFS='|' read -r name status; do
        # Extract agent type and session name from container name
        local agent_type session_name
        agent_type="${name%%-*}"
        session_name="${name#*-}"

        # Determine running/stopped from status string
        local state color
        if [[ "$status" == Up* ]]; then
            state="running"
            color="$COLOR_GREEN"
        else
            state="stopped"
            color="$COLOR_RED"
        fi

        printf "%-20s %-12s ${color}%s${COLOR_RESET}\n" "$session_name" "$agent_type" "$state"
    done <<< "$containers"
}
```

### Restart with Zellij Reattachment
```bash
# Source: Existing agent-session attach pattern + Podman start docs
cmd_restart() {
    local session_name
    session_name=$(resolve_session_name "${1:-}")

    local container
    container=$(find_container "$session_name") || exit 1

    # Check if already running
    if podman ps --format "{{.Names}}" | grep -q "^${container}$"; then
        echo "Session '$session_name' is already running. Attaching..."
    else
        echo "Starting '$session_name'..."
        podman start "$container" >/dev/null
        sleep 1  # Wait for container to be ready
    fi

    # Reattach to Zellij session
    attach_zellij
}
```

### Podman ps Format Fields
```bash
# Source: Podman ps --format documentation
# Available fields for Go template:
#   .ID          - Container ID
#   .Image       - Image name
#   .Names       - Container name
#   .Status      - Status string (e.g., "Up 2 hours", "Exited (0) 3 hours ago")
#   .State       - State (running, exited, etc.)
#   .Ports       - Port mappings
#   .CreatedAt   - Creation timestamp

# Example: Get name and state for all agent containers
podman ps -a --format "{{.Names}}|{{.State}}" --filter "name=^(claude|opencode)-"
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Raw podman commands in docs | Subcommand wrapper | Phase 4 | User-friendly session management |
| Single main() flow | Subcommand dispatch | Phase 4 | Extensible command structure |
| No session listing | Formatted ls output | Phase 4 | Users can see all sessions at a glance |

**Deprecated/outdated:**
- Help text referencing raw `podman stop agent-NAME` - Will be replaced with `agent-session stop NAME`
- Single-purpose script - Becomes multi-command tool

## Open Questions

Things that couldn't be fully resolved:

1. **Zellij Session Persistence After Container Restart**
   - What we know: Zellij sessions run inside the container; when container stops, Zellij process stops
   - What's unclear: Whether Zellij session state persists (scrollback, pane layout) across stop/start
   - Recommendation: Test during implementation; likely lost on stop (transient process state), but session name survives

2. **Agent Type Discovery for find_container**
   - What we know: Containers named `${AGENT_TYPE}-${SESSION_NAME}`, need to check all agent types
   - What's unclear: Whether to hardcode agent types or discover from `config/agents/`
   - Recommendation: Use `find config/agents -type d` to build dynamic alternation pattern, avoiding hardcoding

3. **Exit Codes for User Scripts**
   - What we know: Standard exit codes (0=success, 1=error, 125+ reserved)
   - What's unclear: Which specific codes to use for "session not found" vs "cannot rm running"
   - Recommendation: Use 1 for all user-facing errors (simple), document behavior in help text

## Sources

### Primary (HIGH confidence)
- [Podman stop documentation](https://docs.podman.io/en/latest/markdown/podman-stop.1.html) - Stop behavior, options, timeouts
- [Podman start documentation](https://docs.podman.io/en/latest/markdown/podman-start.1.html) - Start options, attach behavior
- [Podman rm documentation](https://docs.podman.io/en/latest/markdown/podman-rm.1.html) - Exit codes (0, 1, 2, 125)
- [Podman ps documentation](https://docs.podman.io/en/latest/markdown/podman-ps.1.html) - Filter, format, status fields
- [Docker CLI reference](https://docs.docker.com/reference/cli/docker/) - Subcommand organization patterns
- [Command Line Interface Guidelines](https://clig.dev/) - CLI design best practices

### Secondary (MEDIUM confidence)
- [Bash subcommand patterns (GitHub gist)](https://gist.github.com/waylan/4080362) - Subcommand function dispatch
- [ANSI color codes (FLOZz MISC)](https://misc.flogisoft.com/bash/tip_colors_and_formatting) - Color escape sequences
- [Bash confirmation prompts (linuxconfig.org)](https://linuxconfig.org/bash-script-yes-no-prompt-example) - Yes/no patterns
- [Exit codes (tldp.org)](https://tldp.org/LDP/abs/html/exitcodes.html) - Reserved exit code ranges
- [Bash best practices (bertvv)](https://bertvv.github.io/cheat-sheets/Bash.html) - Error handling patterns

### Tertiary (LOW confidence)
- Community discussions on subcommand organization - Various approaches exist
- Blog posts on colorized output - Portability concerns vary by source

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - Official Podman documentation, well-established bash patterns
- Architecture: HIGH - Subcommand dispatch is standard practice, existing codebase patterns apply
- Pitfalls: HIGH - Based on documented Podman behavior and exit codes
- Zellij persistence: MEDIUM - Logical inference, needs testing

**Research date:** 2026-01-27
**Valid until:** 2026-02-27 (30 days - stable Podman/bash APIs)
