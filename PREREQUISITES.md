# Prerequisites

This document lists all requirements needed to use `agent-session` on your machine.

## Host System Requirements

### Operating System

**Linux** is required (tested on Debian/Ubuntu).

The container uses platform-specific binaries (Zellij for x86_64 Linux). macOS and Windows are not currently supported.

**Verify:**
```bash
uname -s
# Expected: Linux
```

### Podman

Container runtime for rootless containers. Used to build and run the agent-session environment.

**Required version:** 4.0+

**Verify:**
```bash
podman --version
# Expected: podman version 4.x.x or higher
```

**Install:**

Debian/Ubuntu:
```bash
sudo apt update
sudo apt install podman
```

Fedora:
```bash
sudo dnf install podman
```

Arch Linux:
```bash
sudo pacman -S podman
```

See [official Podman installation guide](https://podman.io/getting-started/installation) for other distributions.

### Bash

Shell interpreter required to run the `agent-session` wrapper script.

**Required version:** 4.0+

**Verify:**
```bash
bash --version
# Expected: GNU bash, version 4.x.x or higher
```

**Note:** Bash is pre-installed on most Linux distributions. If missing:

Debian/Ubuntu:
```bash
sudo apt install bash
```

### Git (Optional)

Required if you want to work with version-controlled projects inside agent-session. The container includes git, but your projects likely need it on the host system.

**Verify:**
```bash
git --version
# Expected: git version 2.x.x or higher
```

**Install:**

Debian/Ubuntu:
```bash
sudo apt install git
```

## User Configuration Requirements

### Claude API Credentials

**Required:** `~/.claude.json` file containing your Anthropic API key.

The container mounts this file at `/home/agent/.claude.json` for Claude Code to authenticate.

**Create the file:**
```bash
cat > ~/.claude.json << 'EOF'
{
  "apiKey": "your-anthropic-api-key-here"
}
EOF
chmod 600 ~/.claude.json
```

**Obtain API key:** Visit [Anthropic Console](https://console.anthropic.com/) to create an API key.

**Verify:**
```bash
test -f ~/.claude.json && echo "✓ Claude config exists" || echo "✗ Missing ~/.claude.json"
```

**Security note:** Never commit this file to version control. The API key provides access to your Anthropic account.

### Claude Settings Directory (Optional)

**Location:** `~/.claude/`

If this directory exists on your host, it will be mounted to `/config/.claude` in the container and synchronized to the container's `~/.claude/` directory.

This is useful for:
- Custom Claude Code settings
- MCP server configurations
- get-shit-done plugin data (automatically installed in container)

**Note:** The get-shit-done plugin will be automatically installed in the container if not present, so this is truly optional.

### Claude Local Data Directory (Optional)

**Location:** `~/.local/share/claude`

If this directory exists, it's mounted to persist Claude Code's local data (conversation history, cache, etc.) across container restarts.

### Zellij Configuration (Optional)

**Location:** `~/.config/zellij/`

If you have custom Zellij layouts or settings, they'll be copied into the container on startup.

**Note:** This is mounted read-only and copied to the container, so changes made inside the container won't affect your host configuration.

## Build-Time Network Requirements

The container build process downloads several tools:

- **Zellij** - Downloaded from GitHub releases
- **Starship prompt** - Installed via install script
- **uv** (Python package manager) - Downloaded from Astral.sh
- **Claude Code** - Installed via claude.ai installer script
- **get-shit-done** - Installed via npm during first container run

**Network requirements:**
- HTTPS access to github.com
- HTTPS access to claude.ai
- HTTPS access to astral.sh
- HTTPS access to npmjs.com

If you're behind a corporate proxy or firewall, you may need to configure proxy settings for Podman.

## Verification Checklist

Run all commands to verify your system is ready:

```bash
# System
uname -s                 # Should show: Linux
podman --version         # Should show: 4.0+
bash --version           # Should show: 4.0+

# Claude credentials
test -f ~/.claude.json && echo "✓ Claude config exists" || echo "✗ Missing ~/.claude.json"

# Optional but recommended
test -d ~/.config/zellij && echo "✓ Zellij config exists" || echo "○ No Zellij config (will use defaults)"
test -d ~/.claude && echo "✓ Claude settings exist" || echo "○ No Claude settings (will use defaults)"
```

## Post-Installation

After verifying prerequisites:

1. Build the container image:
   ```bash
   podman build -t claude-agent .
   ```

2. Start your first session:
   ```bash
   ./agent-session -n test $(pwd)
   ```

3. Inside the container, Claude Code will start automatically. Type `/gsd:help` to explore the get-shit-done workflow system.

## Troubleshooting

### "podman: command not found"

Podman is not installed. Follow the installation instructions above for your distribution.

### "Error: API key not found"

The `~/.claude.json` file is missing or incorrectly formatted. Verify the file exists and contains valid JSON with an `apiKey` field.

### "Permission denied" when accessing project files

Ensure the project directory you're mounting is readable by your user. The container runs with `--userns=keep-id` which maps your host UID to the container.

### Container build fails downloading dependencies

Check your network connection and firewall settings. The build requires HTTPS access to several external services (see Build-Time Network Requirements).

### "command -v zellij" fails in container

The container build failed or completed with errors. Rebuild with:
```bash
podman build --no-cache -t claude-agent .
```
