# Agent Configuration Reference

Agent configs define how to install, configure, and run AI agents in containers. Each agent gets a config file that specifies its requirements and behavior.

## Directory Structure

Agent configs live under `config/agents/<agent-name>/config.conf`:

```
config/agents/
└── claude/
    └── config.conf
```

**XDG Override:** User-specific agent configs can be placed in `~/.config/agent-session/agents/<agent-name>/config.conf` to override bundled configs without modifying the repository.

## Required Fields

Each agent config must define these fields:

| Field | Type | Purpose |
|-------|------|---------|
| `AGENT_NAME` | String | Agent identifier - must match directory name and Containerfile stage name |
| `AGENT_DESCRIPTION` | String | Human-readable description shown in menus and help output |
| `AGENT_INSTALL_CMD` | Command | Shell command to install the agent in the container (runs as agent user during build) |
| `AGENT_LAUNCH_CMD` | Command | Shell command to start the agent in an interactive session |
| `AGENT_SHELL` | Path | Full path to the agent's default shell (e.g., `/usr/bin/fish`, `/bin/bash`) |
| `AGENT_REQUIRED_MOUNTS` | Space-separated paths | Mount points that must exist for the agent to function |
| `AGENT_ENV_VARS` | Space-separated KEY=value | Environment variables to set in the container runtime |

## Config Format

Config files use shell-sourceable `KEY=value` format with these constraints:

- One variable per line: `VARIABLE="value"`
- Comments allowed: lines starting with `#`
- Variable expansion allowed: `$VAR` or `${VAR}`
- Command substitution forbidden: no `$()` or backticks (security requirement)
- Values with spaces must be quoted

## Example: Claude Agent

```bash
# Claude Agent Configuration
AGENT_NAME="claude"
AGENT_DESCRIPTION="Anthropic Claude Code agent"
AGENT_INSTALL_CMD="curl -fsSL https://claude.ai/install.sh | bash"
AGENT_LAUNCH_CMD="claude --dangerously-skip-permissions"
AGENT_SHELL="/usr/bin/fish"
AGENT_REQUIRED_MOUNTS="/workspace /config/.claude /config/zellij"
AGENT_ENV_VARS="PATH=/home/agent/.local/bin:\$PATH SHELL=/usr/bin/fish"
```

## Adding a New Agent

1. Create config directory: `mkdir -p config/agents/<agent-name>`
2. Write config file: `config/agents/<agent-name>/config.conf`
3. Add Containerfile stage for agent installation (if needed)
4. Rebuild container: `podman build -t agent-session .`
5. Test with: `agent-session -n test-<agent-name> /path/to/project`

## Security Note

Config files are validated before use. Command substitution (`$()`, backticks) is rejected to prevent code injection. Only simple variable expansion is allowed.

Variable expansion is processed by the shell when sourcing the config, so escaped variables (`\$VAR`) will be preserved for later expansion in the container environment.
