# External Integrations

**Analysis Date:** 2026-01-26

## APIs & External Services

**Tool Installation Services:**
- Zellij GitHub Releases - Used to download terminal multiplexer binary
  - Source: `https://github.com/zellij-org/zellij/releases/latest/download/zellij-x86_64-unknown-linux-musl.tar.gz`
  - Integration: Direct tarball download at container build time

- Starship Installation - Used to download and install prompt framework
  - Source: `https://starship.rs/install.sh`
  - Integration: Bash script execution at container build time

- uv Installer - Used to install Python packaging tool
  - Source: `https://astral.sh/uv/install.sh`
  - Integration: Bash script execution at container build time

- Claude Code Installer - Used to install AI assistant
  - Source: `https://claude.ai/install.sh`
  - Integration: Bash script execution at container build time as non-root user (agent)

- get-shit-done-cc NPM Package - Used to install GSD CLI plugin
  - Source: npm registry (`get-shit-done-cc@latest`)
  - Integration: npx execution at container runtime if not already installed
  - Location: `~/.claude/get-shit-done/VERSION` (checked to determine installation)

## Data Storage

**Configuration Storage:**
- Host filesystem - Claude configuration mounted at `/config/.claude`
  - Mount point: `/config/.claude:/home/agent/.claude`
  - Read-write access for persistent configuration
  - Implementation: Bash symlink/copy logic in `entrypoint.sh`

- Host filesystem - Zellij configuration mounted at `/config/zellij`
  - Mount point: `/config/zellij:/home/agent/.config/zellij`
  - Read-only initial mount, copied to writable location at runtime
  - Implementation: Lines 27 in `entrypoint.sh`

- Host filesystem - Claude authentication data mounted at `/config/.claude.json`
  - Mount point: `/config/.claude.json:/home/agent/.claude.json`
  - Read-write access for auth tokens/credentials

- Workspace Mounts - User project directories mounted at runtime
  - Pattern: Multiple host paths mounted at `/workspace/{dirname}`
  - Implementation: Dynamic mount generation in `agent-session` script (lines 114-135)

- Claude Share Directory - User-specific Claude data and cache
  - Mount point: `$HOME/.local/share/claude:/home/agent/.local/share/claude`
  - Read-write access for cache and session data

## Containers & Isolation

**Container Runtime:**
- Podman - Container orchestration and session management
  - Commands used: `podman run`, `podman start`, `podman stop`, `podman ps`, `podman exec`
  - Container Image: `claude-agent` (built from Containerfile)
  - User namespace: `--userns=keep-id` (preserve host UID/GID mapping)
  - Container naming: `agent-{SESSION_NAME}` (dynamic based on session)

## Session Management

**Zellij Terminal Multiplexer:**
- Integration: Runs inside container to provide persistent terminal sessions
- Configuration: Mounted from host at `/config/zellij`
- Sessions: Named sessions created and attached to dynamically
- Shell integration: Claude-session wrapper starts Claude Code then drops to Fish shell

## Authentication & Identity

**Claude Code Authentication:**
- Method: Native installer handles authentication setup
- Storage: `~/.claude.json` (mounted from host)
- Configuration directory: `~/.claude/` (mounted from host)
- Access: Host-managed credentials passed into container

## Environment Configuration

**Required Environment Variables (Container Launch):**
- `SHELL` - Set to `/usr/bin/fish` for shell environment
- `PATH` - Extended to include `/home/agent/.local/bin` (for native installers)

**Optional Host Environment Variables:**
- `AGENT_SESSION_MOUNTS` - Colon-separated list of extra host paths to mount as read-only
  - Implementation: Parsed in `agent-session` script lines 124-135
  - Format: `/path1:/path2` (paths mounted at same location in container)

**Container Environment Files:**
- `~/.claude/` - Claude configuration (symlinked from mounted `/config/.claude`)
- `~/.config/fish/config.fish` - Fish shell configuration (generated at build time)
- `~/.config/zellij/` - Zellij configuration (copied from mounted `/config/zellij`)

## Secrets & Credentials

**Credentials Storage:**
- Host-mounted at `/config/.claude.json` → `/home/agent/.claude.json` (read-write)
- Claude API tokens assumed to be stored in mounted configuration
- No hardcoded secrets in Containerfile or scripts
- Secrets handled by host environment, injected via mounts

## Persistence

**Session Persistence:**
- Zellij sessions persist in container memory across terminal detaches
- Container continues running after client disconnects (background process at line 150 of `agent-session`)
- Containers restart automatically if stopped (via `podman start`)

**Configuration Persistence:**
- Host-based configuration mounted into container
- Changes to `~/.claude` and `~/.config/zellij` persist through mount points
- Project mounts provide access to version-controlled work

## Webhooks & Callbacks

**Incoming:**
- Not applicable - this is a development environment

**Outgoing:**
- Claude Code may integrate with external APIs depending on plugins
- MCP (Model Context Protocol) servers can be run via uv for dynamic capabilities

---

*Integration audit: 2026-01-26*
