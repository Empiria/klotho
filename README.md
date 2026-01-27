# Klotho

Run AI coding agents in isolated, reproducible containers with persistent terminal sessions.

Klotho creates containerized workspaces for AI agents like Claude Code and OpenCode, giving you consistent development environments that persist across terminal disconnects. Close your terminal, and the agent session keeps running — reattach anytime and pick up where you left off.

## Installation

**Quick install (Linux/macOS):**
```bash
curl -fsSL https://raw.githubusercontent.com/Empiria/klotho/main/install.sh | bash
```

This downloads the correct binary for your platform to `~/.local/bin/klotho`.

**Manual download:**

Download the latest release from [GitHub Releases](https://github.com/Empiria/klotho/releases) and place the binary in your PATH.

## Quick Start

**1. Set up your agent locally first**

Klotho mounts your local configuration into containers, so you need your chosen agent working on your host machine first:

- **Claude Code:** Install and authenticate per [Claude Code docs](https://docs.anthropic.com/en/docs/claude-code)
- **OpenCode:** Install and configure per [OpenCode docs](https://opencode.ai/)

Your existing `~/.claude.json`, `~/.claude/`, `~/.config/opencode/`, etc. will be mounted into the container automatically.

**2. Build the agent image:**
```bash
klotho build claude    # or: klotho build opencode
```

**3. Start a session:**
```bash
klotho start ~/projects/my-app                 # defaults to claude
klotho start -a opencode ~/projects/my-app     # use opencode instead
```

That's it. You're now in a containerized agent session with your project mounted at `/workspace`.

**Detach anytime:** Press `Ctrl+C` or close your terminal — the session keeps running.

**Reattach later:**
```bash
klotho start
```

## Prerequisites

**Required:**

- **Podman 4.0+** (or Docker) — Container runtime
  ```bash
  # Linux (Debian/Ubuntu)
  sudo apt install podman

  # Linux (Fedora)
  sudo dnf install podman

  # macOS
  brew install podman
  podman machine init && podman machine start
  ```

- **A working AI agent** — Claude Code or OpenCode configured locally (see Quick Start)

**Optional:**
- `~/.claude/` — Custom Claude Code settings, MCP configs (mounted automatically)
- `~/.config/opencode/` — OpenCode configuration (mounted automatically)
- `~/.config/zellij/` — Custom Zellij layouts (copied into container)

## Concepts

**Podman vs Docker:** Podman runs containers without a daemon and without root. Commands are nearly identical to Docker.

**Zellij:** Terminal multiplexer that keeps sessions alive when you disconnect. Like tmux, but with a friendlier interface.

**Agents:** AI coding assistants (Claude Code, OpenCode) that run inside Klotho containers with consistent, isolated environments.

## Commands

### start

<details>
<summary>Create a new session or attach to existing one</summary>

```
klotho start [-a AGENT] [-n NAME] [project-paths...]
```

**Options:**
- `-a, --agent AGENT` — Agent to use (default: claude)
- `-n, --name NAME` — Session name (default: default)
- `--linked-dir DIR` — Directory to mount at same path (repeatable, for symlinks)

**Examples:**
```bash
klotho start                              # Current directory, default session
klotho start ~/projects/webapp            # Specific project
klotho start -n frontend ~/webapp         # Named session
klotho start -n fullstack ~/fe ~/be       # Multiple directories
klotho start -a opencode ~/project        # Different agent
```

**Linked Directories:**

When your workspace contains symlinks to external directories, those directories must be mounted at the same path inside the container for the symlinks to resolve:

```bash
# Using environment variable (colon-separated)
export KLOTHO_LINKED_DIRS="/home/user/shared-tools:/home/user/team-configs"
klotho start ~/project

# Using CLI flag (repeatable)
klotho start --linked-dir /home/user/shared-tools --linked-dir /home/user/team-configs ~/project
```

The symlinks themselves can be excluded from git via `.git/info/exclude`.

**Notes:**
- Sessions persist across terminal disconnects
- Omit `-a` to see interactive agent menu

</details>

### stop

<details>
<summary>Stop a running session</summary>

```
klotho stop [SESSION_NAME]
```

**Examples:**
```bash
klotho stop              # Stop "default" session
klotho stop frontend     # Stop named session
```

</details>

### restart

<details>
<summary>Restart a stopped session and reattach</summary>

```
klotho restart [SESSION_NAME]
```

**Examples:**
```bash
klotho restart           # Restart "default" session
klotho restart frontend  # Restart named session
```

</details>

### ls

<details>
<summary>List all sessions with status</summary>

```
klotho ls
```

**Output:**
```
NAME                 AGENT        STATUS
default              claude       running
frontend             claude       stopped
backend              opencode     running
```

</details>

### rm

<details>
<summary>Remove a stopped session</summary>

```
klotho rm [-f|--force] [SESSION_NAME]
```

**Examples:**
```bash
klotho rm frontend       # Remove with confirmation
klotho rm -f frontend    # Remove without confirmation
```

**Note:** Stop the session first with `klotho stop`.

</details>

### build

<details>
<summary>Build agent container image</summary>

```
klotho build [AGENT]
```

**Examples:**
```bash
klotho build claude      # Build Claude agent image
klotho build opencode    # Build OpenCode agent image
```

</details>

### rebuild

<details>
<summary>Rebuild agent image without cache</summary>

```
klotho rebuild [AGENT]
```

Forces a fresh build, useful when upstream tools have updated.

</details>

## Configuration

### Agent Configs

Agent configs define how to install and run AI agents. They live in `config/agents/<agent-name>/config.conf`:

```bash
# config/agents/claude/config.conf
AGENT_NAME="claude"
AGENT_DESCRIPTION="Anthropic Claude Code agent"
AGENT_INSTALL_CMD="curl -fsSL https://claude.ai/install.sh | bash"
AGENT_LAUNCH_CMD="claude --dangerously-skip-permissions"
AGENT_SHELL="/usr/bin/fish"
AGENT_ENV_VARS="PATH=/home/agent/.local/bin:\$PATH SHELL=/usr/bin/fish"
```

**Required fields:**

| Field | Purpose |
|-------|---------|
| `AGENT_NAME` | Identifier (must match directory name) |
| `AGENT_DESCRIPTION` | Shown in menus and help |
| `AGENT_INSTALL_CMD` | Shell command to install agent |
| `AGENT_LAUNCH_CMD` | Shell command to start agent |
| `AGENT_SHELL` | Default shell path |
| `AGENT_ENV_VARS` | Space-separated KEY=value pairs |

**User overrides:** Place custom configs in `~/.config/klotho/agents/<agent-name>/config.conf` to override bundled configs.

### Adding a New Agent

1. Create config: `mkdir -p config/agents/myagent && vim config/agents/myagent/config.conf`
2. Build image: `klotho build myagent`
3. Test: `klotho start -a myagent ~/project`

### Environment Variables

| Variable | Purpose |
|----------|---------|
| `KLOTHO_MOUNTS` | Additional mount specifications (comma-separated, e.g., `/host/path:/container/path:Z`) |
| `KLOTHO_LINKED_DIRS` | Directories mounted at same path for symlink resolution (colon-separated) |

See `klotho start --help` for details.

## Development

**Building from source:**
```bash
git clone https://github.com/Empiria/klotho.git
cd klotho
cargo build --release
./target/release/klotho --help
```

**Running tests:**
```bash
cargo test
```

**Project structure:**
```
src/
├── main.rs          # CLI entry point
├── commands/        # Command implementations (start, stop, ls, etc.)
├── config/          # Agent config loading
├── container/       # Podman/Docker runtime abstraction
└── resources/       # Embedded Containerfile and agent configs
config/agents/       # Agent configuration files
```

## Troubleshooting

### "podman: command not found"

Install Podman (see Prerequisites) or use Docker by setting `--runtime docker`.

### "Cannot connect to Podman" (macOS)

The podman machine isn't running:
```bash
podman machine start
```

### UID mapping errors

Podman's rootless setup is incomplete:
```bash
sudo usermod --add-subuids 100000-165535 --add-subgids 100000-165535 $USER
podman system migrate
```

### "session 'X' not found"

Check available sessions:
```bash
klotho ls
```

### "cannot remove running session"

Stop it first:
```bash
klotho stop SESSION_NAME
klotho rm SESSION_NAME
```

### Container fails to start

1. Verify your agent works locally first (run `claude` or `opencode` outside klotho)
2. Check that config files exist (`~/.claude.json` for Claude, `~/.config/opencode/` for OpenCode)
3. Rebuild the image: `klotho rebuild claude` (or `klotho rebuild opencode`)

## About

**Name origin:** In Greek mythology, [Klotho](https://en.wikipedia.org/wiki/Clotho) is one of the Three Fates who spins the thread of life — reflecting this tool's purpose of spinning up and managing AI agent session lifecycles.

**Links:**
- [GitHub Repository](https://github.com/Empiria/klotho)
- [Claude Code](https://docs.anthropic.com/en/docs/claude-code)
- [OpenCode](https://opencode.ai/)
