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
- `-a, --agent AGENT` — Agent to use
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
klotho build [--all] [--install PKG...] [AGENT...]
```

**Options:**
- `--all` — Build all configured agents
- `--install PKG` — Install additional package (repeatable, format: `manager:package`, e.g., `apt:gcc`, `pip:pytest`)

**Examples:**
```bash
klotho build claude                                    # Build specific agent
klotho build --all                                     # Build all agents
klotho build --install apt:gcc --install pip:pytest claude  # Build with one-time packages
```

**Note:** Packages from `--install` flags merge additively with `.klotho.toml` configuration. See [Project Configuration](#project-configuration-klothotoml) below.

</details>

### rebuild

<details>
<summary>Rebuild agent image without cache</summary>

```
klotho rebuild [--all] [--install PKG...] [AGENT...]
```

**Options:**
- `--all` — Rebuild all configured agents
- `--install PKG` — Install additional package (repeatable, format: `manager:package`, e.g., `apt:gcc`, `pip:pytest`)

Forces a fresh build, useful when upstream tools have updated.

</details>

### init

<details>
<summary>Scaffold a .klotho.toml configuration file</summary>

```
klotho init
```

Creates a `.klotho.toml` in the current directory with commented examples for all supported package managers (apt, pip, npm, cargo).

**Example:**
```bash
cd ~/projects/my-app
klotho init              # Creates .klotho.toml with template
```

Edit the generated file to add packages, then rebuild: `klotho build claude`.

See [Project Configuration](#project-configuration-klothotoml) for format details.

**Note:** Refuses to overwrite an existing `.klotho.toml`.

</details>

### mobile

<details>
<summary>Manage mobile access via hapi</summary>

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

**Custom tunnel (optional):**

Set `HAPI_PUBLIC_URL` environment variable to use your own tunnel (Cloudflare, Tailscale, etc.) instead of hapi's built-in relay.

</details>

## Configuration

### Agent Configs

Agent configs define how to install and run AI agents. Klotho comes with built-in configs for supported agents, but you can customize them by placing config files in:

```
~/.config/klotho/agents/<agent-name>/config.conf
```

User configs override the built-in defaults. For example, to customize the Claude agent:

```bash
# ~/.config/klotho/agents/claude/config.conf
AGENT_NAME="claude"
AGENT_DESCRIPTION="Anthropic Claude Code agent"
AGENT_INSTALL_CMD="curl -fsSL https://claude.ai/install.sh | bash"
AGENT_LAUNCH_CMD="claude --dangerously-skip-permissions"
AGENT_SHELL="/usr/bin/fish"
AGENT_ENV_VARS="PATH=/home/agent/.local/bin:\$PATH SHELL=/usr/bin/fish"
```

**Config fields:**

| Field | Purpose |
|-------|---------|
| `AGENT_NAME` | Identifier (must match directory name) |
| `AGENT_DESCRIPTION` | Shown in menus and help |
| `AGENT_INSTALL_CMD` | Shell command to install agent |
| `AGENT_LAUNCH_CMD` | Shell command to start agent |
| `AGENT_SHELL` | Default shell path |
| `AGENT_ENV_VARS` | Space-separated KEY=value pairs |

### Adding a New Agent

1. Create config: `mkdir -p ~/.config/klotho/agents/myagent && vim ~/.config/klotho/agents/myagent/config.conf`
2. Build image: `klotho build myagent`
3. Test: `klotho start -a myagent ~/project`

### Environment Variables

| Variable | Purpose |
|----------|---------|
| `KLOTHO_MOUNTS` | Additional mount specifications (comma-separated, e.g., `/host/path:/container/path:Z`) |
| `KLOTHO_LINKED_DIRS` | Directories mounted at same path for symlink resolution (colon-separated) |

See `klotho start --help` for details.

### Project Configuration (.klotho.toml)

Specify additional packages to install into agent containers on a per-project basis. Klotho supports `apt`, `pip`, `npm`, and `cargo` package managers.

**Location:** Place `.klotho.toml` in your project root (the directory you pass to `klotho start`). Run `klotho init` to scaffold one with commented examples.

**Format:**

```toml
[packages.apt]
package-name = "*"      # latest version

[packages.pip]
package-name = ">=1.0"  # version constraint

[packages.npm]
package-name = "^5.0"   # semver range

[packages.cargo]
package-name = "*"
```

**Example:**

The Klotho repository itself uses this configuration for Rust development:

```toml
[packages.apt]
build-essential = "*"
pkg-config = "*"

[packages.cargo]
rustup = "*"
```

**Known recipes:**

Certain package names trigger specialized installers:
- `rustup` or `rust` in `[packages.cargo]` — Installs Rust via the rustup installer
- `nvm` or `node` in `[packages.npm]` — Installs Node.js via nvm

**Workflow:**

1. `klotho init` — Scaffold `.klotho.toml` with commented examples
2. Edit the file to add your project's required packages
3. `klotho build claude` — Packages are installed during the build
4. Packages are available in all sessions using that image

**Note:** `klotho build --install apt:gcc` merges additively with `.klotho.toml` — useful for one-off packages without editing the config file.

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

1. Check package names are correct for the package manager (e.g., `build-essential` not `build_essential` for apt)
2. Verify TOML syntax: `[packages.apt]` not `[packages.APT]`
3. Try installing the package manually first: `sudo apt install <package>` to confirm it exists

### "klotho init" says file already exists

`.klotho.toml` already exists in the current directory. Edit it directly or remove it first if you want a fresh template.

## About

**Name origin:** In Greek mythology, [Klotho](https://en.wikipedia.org/wiki/Clotho) is one of the Three Fates who spins the thread of life — reflecting this tool's purpose of spinning up and managing AI agent session lifecycles.

**Links:**
- [GitHub Repository](https://github.com/Empiria/klotho)
- [Claude Code](https://docs.anthropic.com/en/docs/claude-code)
- [OpenCode](https://opencode.ai/)
