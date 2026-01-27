# Prerequisites

This document lists all requirements needed to use Klotho on your machine.

## Host System Requirements

### Operating System

**Linux** or **macOS** is required.

**Tested on:**
- Linux: Debian/Ubuntu, Fedora, Arch Linux
- macOS: macOS 12 (Monterey) or later (Intel and Apple Silicon)

**Note:** The container uses platform-specific binaries (Zellij for x86_64 Linux). Windows is not currently supported.

**Verify:**
```bash
uname -s
# Expected: Linux or Darwin (macOS)
```

### Podman

Container runtime for rootless containers. Used to build and run the Klotho environment.

**Required version:** 4.0+

**Verify:**
```bash
podman --version
# Expected: podman version 4.x.x or higher
```

**Install:**

**Linux:**

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

**macOS:**

Option 1 - Podman Desktop (recommended for beginners):
```bash
brew install --cask podman-desktop
```
Then launch Podman Desktop and follow the setup wizard to initialize the podman machine.

Option 2 - CLI only:
```bash
brew install podman
```

After installation, initialize and start the podman machine:
```bash
podman machine init
podman machine start
```

**macOS-specific notes:**
- Podman runs containers inside a lightweight Linux VM on macOS (the "podman machine")
- The machine must be running before building or running containers: `podman machine start`
- To check machine status: `podman machine list`
- To stop the machine when done: `podman machine stop`

### Bash

Shell interpreter required to run the `klotho` wrapper script.

**Required version:** 4.0+

**Verify:**
```bash
bash --version
# Expected: GNU bash, version 4.x.x or higher
```

**Install:**

**Linux:**

Bash is pre-installed on most Linux distributions. If missing:

Debian/Ubuntu:
```bash
sudo apt install bash
```

**macOS:**

macOS includes Bash 3.x by default (for licensing reasons). Install a modern version via Homebrew:

```bash
brew install bash
```

After installation, verify you're using the Homebrew version:
```bash
/opt/homebrew/bin/bash --version  # Apple Silicon
# or
/usr/local/bin/bash --version     # Intel
```

To use the newer bash as your default shell (optional):
```bash
sudo bash -c 'echo /opt/homebrew/bin/bash >> /etc/shells'  # Apple Silicon
# or
sudo bash -c 'echo /usr/local/bin/bash >> /etc/shells'     # Intel

chsh -s /opt/homebrew/bin/bash  # or /usr/local/bin/bash for Intel
```

### Git (Optional)

Required if you want to work with version-controlled projects inside Klotho. The container includes git, but your projects likely need it on the host system.

**Verify:**
```bash
git --version
# Expected: git version 2.x.x or higher
```

**Install:**

**Linux:**

Debian/Ubuntu:
```bash
sudo apt install git
```

Fedora:
```bash
sudo dnf install git
```

Arch Linux:
```bash
sudo pacman -S git
```

**macOS:**

Git is included with Xcode Command Line Tools. If not installed:

```bash
xcode-select --install
```

Or install via Homebrew for the latest version:
```bash
brew install git
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
uname -s                 # Should show: Linux or Darwin (macOS)
podman --version         # Should show: 4.0+
bash --version           # Should show: 4.0+

# macOS only - verify podman machine is running
podman machine list      # Should show a running machine (macOS only)

# Claude credentials
test -f ~/.claude.json && echo "✓ Claude config exists" || echo "✗ Missing ~/.claude.json"

# Optional but recommended
test -d ~/.config/zellij && echo "✓ Zellij config exists" || echo "○ No Zellij config (will use defaults)"
test -d ~/.claude && echo "✓ Claude settings exist" || echo "○ No Claude settings (will use defaults)"
```

## Post-Installation

After verifying prerequisites:

1. **(macOS only)** Ensure the podman machine is running:
   ```bash
   podman machine start
   ```

2. Build the container image:
   ```bash
   ./scripts/build.sh claude
   ```

3. Start your first session:
   ```bash
   ./klotho start
   ```

4. Inside the container, Claude Code will start automatically. Type `/gsd:help` to explore the get-shit-done workflow system.

## Troubleshooting

### "podman: command not found"

Podman is not installed. Follow the installation instructions above for your distribution or operating system.

### macOS: "Cannot connect to Podman" or "Error: unable to connect to Podman socket"

The podman machine is not running. Start it with:
```bash
podman machine start
```

Verify it's running:
```bash
podman machine list
```

### macOS: "No such file or directory" when running podman commands

Make sure you initialized the podman machine:
```bash
podman machine init
podman machine start
```

### "Error: API key not found"

The `~/.claude.json` file is missing or incorrectly formatted. Verify the file exists and contains valid JSON with an `apiKey` field.

### "Permission denied" when accessing project files

Ensure the project directory you're mounting is readable by your user. The container runs with `--userns=keep-id` which maps your host UID to the container.

### Container build fails downloading dependencies

Check your network connection and firewall settings. The build requires HTTPS access to several external services (see Build-Time Network Requirements).

### "command -v zellij" fails in container

The container build failed or completed with errors. Rebuild with:
```bash
./scripts/build.sh claude
```
