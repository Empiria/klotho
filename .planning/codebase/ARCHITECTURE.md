# Architecture

**Analysis Date:** 2026-01-26

## Pattern Overview

**Overall:** Container-based session orchestrator with client-server separation.

**Key Characteristics:**
- Host-side CLI client controls container lifecycle and terminal multiplexing
- Container-side runtime environment with persistent configuration management
- Stateful session persistence across disconnects using Podman and Zellij
- Configuration injection via bind mounts and entrypoint initialization

## Layers

**Host Layer (Client):**
- Purpose: Manage container lifecycle, handle user input, attach/reattach to sessions
- Location: `agent-session` (shell script)
- Contains: Argument parsing, container orchestration logic, Zellij session management
- Depends on: Podman CLI, Zellij
- Used by: End user shell

**Container Layer (Runtime):**
- Purpose: Isolated execution environment with Claude Code, multiplexing, and development tools
- Location: `Containerfile`
- Contains: Base OS (Debian), CLI tools, Claude Code installation, Zellij, Starship
- Depends on: Debian bookworm-slim base image
- Used by: Host layer via Podman

**Initialization Layer (Setup):**
- Purpose: Bootstrap container environment, configure persistent state, install plugins
- Location: `entrypoint.sh`
- Contains: Config symlink setup, directory creation, GSD plugin installation
- Depends on: Mounted volumes from host
- Used by: Container startup via ENTRYPOINT

## Data Flow

**Session Creation Flow:**

1. User runs `agent-session -n NAME /path/to/project`
2. Host layer validates paths, constructs Podman run command with volume mounts
3. Podman creates container with project directories mounted at `/workspace/`
4. Entrypoint script runs: symlinks `.claude` config, copies Zellij config, installs GSD plugin
5. Container exec starts Zellij with named session
6. Zellij initializes claude-session wrapper (starts Claude Code, then shells to Fish)
7. User sees Claude Code ready for input in multiplexed terminal

**Session Reattach Flow:**

1. User runs `agent-session -n NAME` (no paths)
2. Host layer checks if container exists and is running
3. If running: exec into existing Zellij session
4. If stopped: restart container and reattach to existing Zellij session

**Configuration State:**

- Host `.claude/` mounted to container `/config/.claude/` (read-write)
- Container creates symlinks to mounted config files in `~/.claude/`
- Zellij config mounted read-only, copied to writable location for use
- State persists as long as container exists (survives terminal detach/reattach)

## Key Abstractions

**Session:**
- Purpose: Represents a persistent Claude Code workspace with multiplexed terminal
- Examples: Named container + Zellij session + mounted workspace directories
- Pattern: Each session is an independently named Podman container with internal Zellij session

**Mount Binding:**
- Purpose: Connect host filesystem paths to container workspace paths
- Examples: `/home/user/projects/repo` → `/workspace/repo`
- Pattern: Automatic directory naming via `basename`, read-write with SELinux context (`:Z`)

**Configuration Injection:**
- Purpose: Bootstrap container with user's local Claude Code config without copying
- Examples: `.claude/` directory, Zellij config
- Pattern: Symlink or copy mounted configs to standard home directory locations

## Entry Points

**User-Facing:**
- Location: `agent-session` (executable shell script)
- Triggers: Shell invocation with arguments or by name for reattach
- Responsibilities: Parse options, validate paths, manage container lifecycle, attach terminal

**Container-Side:**
- Location: `entrypoint.sh` (ENTRYPOINT in Containerfile)
- Triggers: Container starts (new or restarted)
- Responsibilities: Set up `.claude` directory structure, copy Zellij config, install GSD plugin

**Shell Entry:**
- Location: `/home/agent/.local/bin/claude-session` (wrapper script)
- Triggers: Zellij initializes shell
- Responsibilities: Launch Claude Code with permission flag, fallback to Fish shell

## Error Handling

**Strategy:** Fail fast with clear error messages; validate before committing resources.

**Patterns:**
- Path validation before container creation (check existence with `-e`)
- Option validation with explicit error messages for missing required arguments
- Container state checks (running, stopped, missing) before operations
- Graceful handling of broken symlinks in config setup (create directories instead)

## Cross-Cutting Concerns

**Logging:** Direct output to stderr for user feedback (echo commands)

**State Persistence:** Container name-based tracking; Zellij handles session persistence internally

**Authentication:** SSH keys and credentials mounted via `.local/share/claude/` for Claude Code access

**Isolation:** Podman user namespace (--userns=keep-id) preserves host UID/GID, file permissions

---

*Architecture analysis: 2026-01-26*
