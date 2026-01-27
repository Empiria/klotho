# Codebase Structure

**Analysis Date:** 2026-01-26

## Directory Layout

```
agent-session/
├── agent-session           # Host-side CLI client (executable shell script)
├── Containerfile           # Container image definition
├── entrypoint.sh           # Container initialization script
├── .claude/
│   └── settings.local.json # Claude Code permission settings
└── .planning/
    └── codebase/           # Architecture documentation
```

## Directory Purposes

**Root Directory:**
- Purpose: Project root containing all executable components
- Contains: Main CLI, container definition, initialization logic, configuration
- Key files: `agent-session`, `Containerfile`, `entrypoint.sh`

**.claude/ Directory:**
- Purpose: Local Claude Code configuration and permissions
- Contains: JSON permission settings for Claude Code CLI
- Key files: `settings.local.json` (permissions for Podman and web search)

**.planning/codebase/ Directory:**
- Purpose: Architecture and structure documentation for development reference
- Contains: ARCHITECTURE.md, STRUCTURE.md, and other analysis documents
- Key files: Generated analysis documents

## Key File Locations

**Entry Points:**

- `agent-session`: Main user-facing CLI; parses arguments, manages container lifecycle, handles attach/reattach
- `entrypoint.sh`: Container initialization; called by Containerfile ENTRYPOINT; runs before command
- `Containerfile`: Image definition; base OS, dependency installation, user setup

**Configuration:**

- `.claude/settings.local.json`: Permissions policy; grants Podman and web search access to Claude Code CLI
- `/config/.claude/` (mounted): Host's `.claude` config directory mounted into container
- `/config/zellij/` (mounted): Host's Zellij config mounted into container (read-only)

**Core Logic:**

- Lines 39-155 in `agent-session`: Argument parsing and help display
- Lines 74-83 in `agent-session`: Zellij session attachment logic
- Lines 85-112 in `agent-session`: Container state detection and lifecycle
- Lines 114-154 in `agent-session`: Mount construction and container creation
- Lines 9-24 in `entrypoint.sh`: Config symlink and directory setup
- Lines 29-32 in `entrypoint.sh`: GSD plugin installation

**Container Setup:**

- Lines 4-11 in `Containerfile`: OS dependency installation (curl, git, fish, nodejs, npm)
- Lines 13-18 in `Containerfile`: Tool installation (Zellij, Starship)
- Lines 20-43 in `Containerfile`: User setup, shell config, Claude wrapper script
- Lines 45-56 in `Containerfile`: Claude Code installation, environment configuration

## Naming Conventions

**Files:**

- Executable scripts: lowercase with hyphens (`agent-session`, `entrypoint.sh`, `Containerfile`)
- Configuration: lowercase with dots and hyphens (`settings.local.json`, `config.fish`)
- Documentation: UPPERCASE with `.md` extension (ARCHITECTURE.md, STRUCTURE.md)

**Directories:**

- Dotfiles: prefixed with dot (`.claude/`, `.planning/`)
- User-created workspace: generic name (`/workspace/` in container, mounted from `~/projects/...`)
- Config subdirectories: tool names (`.config/fish/`, `.config/zellij/`, `.local/bin/`)

**Variables:**

- Shell script variables: UPPERCASE_WITH_UNDERSCORES (SESSION_NAME, CONTAINER_NAME, MOUNTS)
- Bash inline: lowercase_with_underscores (abs_path, dir_name, mount_path)

**Functions:**

- Bash functions: lowercase_with_underscores (show_help, attach_zellij)

## Where to Add New Code

**New Feature (Host-side):**
- Primary code: Add logic to `agent-session` script
- Parse new CLI options: Extend while loop around line 43
- Add new container operations: Add helper functions after line 74

**New Feature (Container-side):**
- Primary code: Add commands to `Containerfile` or `entrypoint.sh`
- New tools: Add to apt-get install or custom installation section (lines 4-11, 45-56)
- New initialization: Add setup logic to `entrypoint.sh` before final exec (line 39)

**Configuration/Permissions:**
- Settings: Update `.claude/settings.local.json` to grant new Bash permissions
- Format: Add entries to `permissions.allow` array with pattern (e.g., "Bash(new-command:*)")

**Documentation:**
- Architecture docs: `.planning/codebase/ARCHITECTURE.md`
- Structure docs: `.planning/codebase/STRUCTURE.md`
- Add specific concerns: `.planning/codebase/CONCERNS.md`

## Special Directories

**Container Working Directory:**
- Location: `/workspace/` (configured in Containerfile line 54)
- Purpose: Standard location where project directories are mounted
- Generated: No (mount points are created at runtime)
- Committed: No (runtime filesystem)

**Container Home Directory:**
- Location: `/home/agent/` (created in Containerfile line 21)
- Purpose: Non-root user environment; contains .local, .config, .claude
- Generated: Yes (useradd creates initially)
- Committed: No (container filesystem)

**Host Config Mounts:**
- Mount path: `/config/.claude/`, `/config/zellij/`
- Source: `~/.claude/`, `~/.config/zellij/` on host
- Purpose: Read configuration from host without copying
- Committed: No (external filesystem)

**GSD Plugin Directory:**
- Location: `~/.claude/get-shit-done/` (inside container)
- Purpose: Global GSD plugin for /gsd commands
- Generated: Yes (installed by entrypoint.sh line 32)
- Committed: No (installed at runtime)

---

*Structure analysis: 2026-01-26*
