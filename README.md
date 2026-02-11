# Klotho

Run AI coding agents in isolated containers — then control them from your phone.

Klotho creates containerized workspaces for AI agents like Claude Code and OpenCode, giving you reproducible environments that persist across terminal disconnects. Sessions keep running when you close your laptop, and you can reattach from any terminal — or pick up right where you left off from your phone via the built-in [mobile hub](#mobile-access).

## Quick Start

**1. Build the agent image:**
```bash
klotho build claude    # or: klotho build opencode
```

**2. Start a session:**
```bash
klotho start ~/projects/my-app
```

You're now in a containerized agent session with your project mounted at `/workspace`.

**3. Detach and reattach:**

Press `Ctrl+C` or close your terminal — the session keeps running. Reattach later:
```bash
klotho start
```

**4. Go mobile (optional):**
```bash
klotho mobile start      # Scan the QR code with your phone
```

## Installation

**Quick install (Linux/macOS):**
```bash
curl -fsSL https://raw.githubusercontent.com/Empiria/klotho/main/install.sh | bash
```

This downloads the correct binary for your platform to `~/.local/bin/klotho`.

**Manual download:**

Download the latest release from [GitHub Releases](https://github.com/Empiria/klotho/releases) and place the binary in your PATH.

**Prerequisites:**

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

- **A working AI agent** — Klotho mounts your local agent configuration into containers, so you need your chosen agent working on your host machine first:
  - **Claude Code:** Install and authenticate per [Claude Code docs](https://docs.anthropic.com/en/docs/claude-code)
  - **OpenCode:** Install and configure per [OpenCode docs](https://opencode.ai/)

## Commands

### start

Create a new session or attach to an existing one.

```
klotho start [-a AGENT] [-n NAME] [project-paths...]
```

| Flag | Description |
|------|-------------|
| `-a, --agent AGENT` | Agent to use (interactive selection if omitted) |
| `-n, --name NAME` | Session name (default: `default`) |

```bash
klotho start                              # Current directory, default session
klotho start ~/projects/webapp            # Specific project
klotho start -n frontend ~/webapp         # Named session
klotho start -n fullstack ~/fe ~/be       # Multiple directories
klotho start -a opencode ~/project        # Different agent
```

Sessions persist across terminal disconnects. Omit `-a` to see an interactive agent menu.

### stop

Stop a running session.

```
klotho stop [SESSION_NAME]
```

```bash
klotho stop              # Stop "default" session
klotho stop frontend     # Stop named session
```

### restart

Restart a stopped session and reattach.

```
klotho restart [SESSION_NAME]
```

```bash
klotho restart           # Restart "default" session
klotho restart frontend  # Restart named session
```

### ls

List all sessions with status.

```
klotho ls
```

```
NAME                 AGENT        STATUS
default              claude       running
frontend             claude       stopped
backend              opencode     running
```

### rm

Remove a stopped session.

```
klotho rm [-f|--force] [SESSION_NAME]
```

```bash
klotho rm frontend       # Remove with confirmation
klotho rm -f frontend    # Remove without confirmation
```

Stop the session first with `klotho stop`.

### build

Build agent container image.

```
klotho build [--all] [--install PKG...] [AGENT...]
```

| Flag | Description |
|------|-------------|
| `--all` | Build all configured agents |
| `--install PKG` | Install additional package (repeatable, format: `manager:package`, e.g. `apt:gcc`, `pip:pytest`) |

```bash
klotho build claude                                        # Build specific agent
klotho build --all                                         # Build all agents
klotho build --install apt:gcc --install pip:pytest claude  # With extra packages
```

Packages from `--install` merge additively with `.klotho.toml` packages.

### rebuild

Rebuild agent image without cache. Same options as `build`.

```
klotho rebuild [--all] [--install PKG...] [AGENT...]
```

Forces a fresh build, useful when upstream tools have updated.

### init

Scaffold a configuration file with commented examples.

```
klotho init [--global]
```

| Flag | Description |
|------|-------------|
| `--global` | Create global config (`~/.config/klotho/config.toml`) instead of project config |

```bash
klotho init              # Creates .klotho.toml in current directory
klotho init --global     # Creates ~/.config/klotho/config.toml
```

Refuses to overwrite an existing file. Edit the generated file, then rebuild: `klotho build claude`.

## Configuration

Klotho uses a layered configuration system: **global config** sets user-wide defaults, **project config** customizes per-project, and **agent configs** define how each agent is installed and run.

### Project Config (`.klotho.toml`)

Place `.klotho.toml` in your project root (the directory you pass to `klotho start`). Run `klotho init` to scaffold one with commented examples.

**`[project]` section** — Project metadata:

```toml
[project]
agent = "claude"           # Default agent (skips interactive menu)
name = "my-project"        # Session name template
workdir = "/workspace/src" # Working directory inside container
```

**`[packages]` section** — Additional packages installed during `klotho build`:

```toml
[packages.apt]
build-essential = "*"      # Latest version
pkg-config = "*"

[packages.pip]
pytest = ">=7.0"           # Version constraint

[packages.npm]
typescript = "^5.0"        # Semver range

[packages.cargo]
ripgrep = "*"
```

Known recipes: `rustup` or `rust` in `[packages.cargo]` installs Rust via rustup. `nvm` or `node` in `[packages.npm]` installs Node.js via nvm.

**`[[volumes]]` section** — Extra directories to mount into the container:

```toml
# Simple: mounts at the same path inside the container
volumes = [
    "/home/user/shared-libs",
]

# Detailed: different source and destination, optional readonly
[[volumes]]
src = "~/data"
dest = "/workspace/data"
readonly = true
```

Tilde (`~`) is expanded to `$HOME`. Simple volumes mount at the same path in the container.

**`[mcp]` section** — MCP servers injected into agent configs at runtime:

```toml
# Shared servers (available to all agents)
[mcp.servers.context7]
command = "uvx"
args = ["@upstash/context7-mcp"]

# Agent-specific servers (replaces shared for that agent)
[mcp.claude.custom-tool]
command = "npx"
args = ["-y", "my-tool"]
env = { API_KEY = "..." }
```

If an agent-specific section exists (e.g. `[mcp.claude.*]`), it completely replaces the shared servers for that agent.

**Full example:**

```toml
[project]
agent = "claude"

[packages.apt]
build-essential = "*"
pkg-config = "*"

[packages.cargo]
rustup = "*"

[[volumes]]
src = "~/data"
dest = "/workspace/data"
readonly = true

[mcp.servers.context7]
command = "uvx"
args = ["@upstash/context7-mcp"]
```

### Global Config (`~/.config/klotho/config.toml`)

User-wide defaults that apply to all projects. Run `klotho init --global` to scaffold one.

```toml
# Container runtime ("podman", "docker", or "auto")
runtime = "podman"

# Default agent for new sessions
default_agent = "claude"

# Global volumes mounted in every session
volumes = [
    "/home/user/shared-libs",
]
[[volumes]]
src = "/etc/ssl/certs"
dest = "/etc/ssl/certs"
readonly = true

# Global MCP servers
[mcp.servers.context7]
command = "uvx"
args = ["@upstash/context7-mcp"]
```

**Merging behavior:** Global and project configs are merged at runtime. Project config takes precedence for scalar values (`agent`, `name`, `workdir`). Volumes are additive (global + project, deduplicated by source path). MCP shared servers are additive; agent-specific MCP sections in the project config completely replace the global ones for that agent.

### Agent Configs

Agent configs define how to install and run AI agents. Klotho ships with built-in configs for `claude` and `opencode`. You can customize them by placing a config file at:

```
~/.config/klotho/agents/<agent-name>/config.toml
```

User configs completely replace the built-in defaults for that agent.

**Example** (`~/.config/klotho/agents/claude/config.toml`):

```toml
name = "claude"
description = "Anthropic Claude Code agent"
install_cmd = "curl -fsSL https://claude.ai/install.sh | bash"
launch_cmd = "claude --dangerously-skip-permissions"
shell = "/usr/bin/fish"
env_vars = ["PATH=/home/agent/.local/bin:$PATH", "SHELL=/usr/bin/fish"]
hapi_cmd = "hapi --dangerously-skip-permissions"

[[optional_mounts]]
src = "~/.claude"
dest = "/home/agent/.claude"
```

**Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `name` | string | Identifier (must match directory name) |
| `description` | string | Shown in menus and help |
| `install_cmd` | string | Shell command to install agent during image build |
| `launch_cmd` | string | Shell command to start agent |
| `shell` | string | Default shell path |
| `env_vars` | string array | Environment variables (`KEY=value` format) |
| `hapi_cmd` | string (optional) | Launch command for mobile PTY bridging via hapi |
| `optional_mounts` | volume array (optional) | Host paths to mount if they exist |

**Adding a custom agent:**

1. Create config: `mkdir -p ~/.config/klotho/agents/myagent`
2. Write `~/.config/klotho/agents/myagent/config.toml` with the fields above
3. Build image: `klotho build myagent`
4. Start: `klotho start -a myagent ~/project`

## Mobile Access

Control klotho sessions from your phone using [hapi](https://github.com/tiann/hapi/). A single hapi sidecar container provides a mobile hub for all your agent sessions.

**Start the mobile hub:**
```bash
klotho mobile start
```
Displays a QR code and URL. Scan with your mobile device to connect.

**Check status:**
```bash
klotho mobile status
```
Shows connection URL, QR code, connected devices, and active sessions.

**Stop the mobile hub:**
```bash
klotho mobile stop
```

**Revoke a device:**
```bash
klotho mobile revoke
```
Unpairs a connected mobile device (interactive selection).

**How it works:**
- Hapi runs in a separate sidecar container on the `klotho` network
- Uses built-in relay (WireGuard + TLS) for secure remote access
- Connection persists across restarts — scan QR once, reconnect automatically
- All sessions automatically register with the hub when it's running
- Set `HAPI_PUBLIC_URL` to use your own tunnel (Cloudflare, Tailscale, etc.) instead of the built-in relay

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
├── main.rs            # CLI entry point
├── cli.rs             # Command definitions (clap)
├── agent.rs           # Agent config loading
├── config.rs          # Global config + config merging
├── project_config.rs  # Project config, volumes, MCP, packages
├── container.rs       # Podman/Docker runtime abstraction
├── resources.rs       # Embedded resource loader
├── resources/         # Embedded Containerfile and agent configs
│   └── agents/        # Built-in agent config.toml files
└── commands/          # Command implementations
    ├── start.rs
    ├── stop.rs
    ├── restart.rs
    ├── ls.rs
    ├── rm.rs
    ├── build.rs
    ├── init.rs
    └── mobile/
```

## Troubleshooting

### "podman: command not found"

Install Podman (see [Installation](#installation)) or use Docker by setting `--runtime docker`.

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
2. Check that config files exist (`~/.claude/` for Claude, `~/.config/opencode/` for OpenCode)
3. Rebuild the image: `klotho rebuild claude`

### "klotho mobile start" shows no QR code

The hapi container may not have started properly:
```bash
klotho mobile status     # Check hub state
klotho mobile stop       # Stop and retry
klotho mobile start
```

If the issue persists, check that the `klotho` network exists:
```bash
podman network ls | grep klotho
```

### Mobile device can't connect

- Verify your device and machine are on the same network (or using hapi's relay)
- Try `klotho mobile revoke` and re-scan the QR code
- Set `HAPI_PUBLIC_URL` if behind a custom tunnel

### Build fails with custom packages

If `klotho build` fails after adding packages to `.klotho.toml`:

1. Check package names are correct for the package manager (e.g. `build-essential` not `build_essential` for apt)
2. Verify TOML syntax: `[packages.apt]` not `[packages.APT]`
3. Try installing the package manually first to confirm it exists

### "klotho init" says file already exists

`.klotho.toml` already exists in the current directory. Edit it directly or remove it first if you want a fresh template.

## About

**Name origin:** In Greek mythology, [Klotho](https://en.wikipedia.org/wiki/Clotho) is one of the Three Fates who spins the thread of life — reflecting this tool's purpose of spinning up and managing AI agent session lifecycles.

**Links:**
- [GitHub Repository](https://github.com/Empiria/klotho)
- [Claude Code](https://docs.anthropic.com/en/docs/claude-code)
- [OpenCode](https://opencode.ai/)
