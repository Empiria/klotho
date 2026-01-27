# Technology Stack

**Analysis Date:** 2026-01-26

## Languages

**Primary:**
- Bash - Container orchestration and session management (`/home/owen/projects/personal/agent-session/agent-session`, `entrypoint.sh`)

**Secondary:**
- Shell/Fish - Interactive shell for container environment

## Runtime

**Environment:**
- Debian 12 (bookworm-slim) - Base Linux container image
- Podman - Container runtime for session management

**Package Manager:**
- npm - Node.js package manager (installed in Containerfile)
- uv - Python package installer/manager (installed via curl)
- apt-get - Debian package manager

## Frameworks

**Core:**
- Zellij - Terminal multiplexer for session management (installed from GitHub releases)
- Starship - Cross-shell prompt framework (installed via curl)

**Development Tools:**
- Claude Code - AI coding assistant (installed via native installer)
- Fish shell - Command shell (installed via apt)
- Node.js - JavaScript runtime (installed via apt)

**Utilities:**
- Git - Version control (installed via apt)
- curl - HTTP client (installed via apt)
- ca-certificates - SSL/TLS certificates (installed via apt)

## Key Dependencies

**Critical:**
- `Zellij` - Terminal session multiplexer, essential for persistent development sessions in containers
- `Claude Code` - AI assistant core, installed at runtime via native installer
- `Node.js` - Required for npm-based tool installation

**Infrastructure:**
- `Starship` - Prompt customization for better terminal UX
- `uv` - Python tool installer for MCP (Model Context Protocol) servers
- `Fish shell` - Interactive shell environment for user sessions

## Configuration

**Environment:**
- `SHELL` - Set to `/usr/bin/fish` in Containerfile
- `PATH` - Extended to include `/home/agent/.local/bin` for native installer tools
- `CLAUDE_` configuration - Mounted from host at `/config/.claude` and symlinked to `~/.claude`
- `ZELLIJ` configuration - Mounted from host at `/config/zellij`

**Build:**
- Containerfile - Container image definition using Podman/Docker
- entrypoint.sh - Startup script that configures claude and zellij directories
- agent-session - Bash orchestration script for container lifecycle management

**Container Defaults:**
- Working Directory: `/workspace`
- Non-root User: `agent` (UID 1000)
- Default Shell: Fish shell

## Platform Requirements

**Development:**
- Podman (or Docker compatible runtime)
- Host directories must exist before session creation
- Claude Code native installer access
- Network access for downloading tools (Zellij, Starship, uv, Claude)

**Production:**
- Deployment target: Podman/Docker container environments
- Linux/amd64 platform required (specified in Containerfile)
- Container runs as non-root user (uid 1000) with keep-id namespace flag

## External Tool Installations

**Installed at Container Build Time:**
- Zellij (latest stable from GitHub releases)
- Starship (from https://starship.rs/install.sh)
- Claude Code (from https://claude.ai/install.sh)
- uv (from https://astral.sh/uv/install.sh)

**Installed at Container Runtime:**
- get-shit-done CLI (via npx when not present) - `get-shit-done-cc@latest`

---

*Stack analysis: 2026-01-26*
