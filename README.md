# Agent Session

Run AI coding agents in isolated, reproducible environments with persistent terminal sessions.

Agent Session creates containerized workspaces for AI agents like Claude Code and OpenCode, giving you consistent development environments that persist across terminal disconnects. Work with your AI pair programmer in a clean, managed space with your projects mounted exactly where you need them. When you close your terminal, the agent session keeps running in the background — reattach anytime and pick up right where you left off.

Perfect for developers who want reliable AI assistance without worrying about environment drift, dependency conflicts, or losing work when switching tasks.

## Prerequisites

### Podman

Container runtime for rootless containers. Used to build and run the agent-session environment.

**Required version:** 4.0+

**Verify:**
```bash
podman --version
```

Expected: `podman version 4.x.x` or higher

**Install:**

**Linux (Debian/Ubuntu):**
```bash
sudo apt update
sudo apt install podman
```

**Linux (Fedora):**
```bash
sudo dnf install podman
```

**macOS:**
```bash
brew install podman
```

After installation on macOS, initialize and start the podman machine:
```bash
podman machine init
podman machine start
```

**macOS-specific notes:**
- Podman runs containers inside a lightweight Linux VM on macOS (the "podman machine")
- The machine must be running before building or running containers
- Check machine status: `podman machine list`
- Stop the machine when done: `podman machine stop`

See the [official Podman installation guide](https://podman.io/getting-started/installation) for other Linux distributions.

### Bash

Shell interpreter required to run the `agent-session` wrapper script.

**Required version:** 4.0+

**Verify:**
```bash
bash --version
```

Expected: `GNU bash, version 4.x.x` or higher

**Install:**

**Linux:**

Pre-installed on most distributions. If missing:

```bash
sudo apt install bash
```

**macOS:**

macOS includes Bash 3.x by default. Install a modern version via Homebrew:

```bash
brew install bash
```

### Claude API Credentials

**Required:** `~/.claude.json` file containing your Anthropic API key.

The container mounts this file so Claude Code can authenticate.

**Create the file:**
```bash
cat > ~/.claude.json << 'EOF'
{
  "apiKey": "your-anthropic-api-key-here"
}
EOF
chmod 600 ~/.claude.json
```

**Obtain API key:** Visit the [Anthropic Console](https://console.anthropic.com/) to create an API key.

**Verify:**
```bash
test -f ~/.claude.json && echo "Claude config exists" || echo "Missing ~/.claude.json"
```

**Security note:** Never commit this file to version control. The API key provides access to your Anthropic account.

## Quick Start

Start your first agent session in under 5 minutes:

**1. Clone and enter the repository:**
```bash
git clone https://github.com/your-username/agent-session.git
cd agent-session
```

**2. Build the default agent (Claude):**
```bash
./scripts/build.sh claude
```

This downloads and configures Claude Code, Zellij, and supporting tools inside a container image. Takes 2-3 minutes on first build.

**3. Start a session with your current directory:**
```bash
./agent-session start
```

This creates a container, mounts your current directory to `/workspace`, and drops you into a Zellij terminal session.

**4. Inside the container:**

Claude Code starts automatically. Try these commands:
```
/help
```

Explore the get-shit-done workflow plugin:
```
/gsd:help
```

**5. Detach anytime:**

Press `Ctrl+C` or close your terminal — the session keeps running in the background.

**6. Reattach later:**
```bash
./agent-session start
```

**7. Use with your own projects:**

Mount specific directories:
```bash
./agent-session start ~/projects/my-app
```

Mount multiple repositories in one session:
```bash
./agent-session start -n fullstack ~/frontend ~/backend ~/shared-libs
```

**8. Choose your agent:**

Agent Session supports multiple AI agents. Use the `-a` flag to select:

```bash
./agent-session start -a opencode ~/project
```

Or omit the `-a` flag to see an interactive menu of available agents:
```bash
./agent-session start
```

You'll see:
```
Available agents:
  1. Claude (ready)
  2. Opencode (ready)

Select agent (default: claude):
```

Press Enter for the default, or type a number to select a different agent.

## Concepts

New to some of the technologies? Here's what you need to know:

**Podman vs Docker:** Podman is like Docker but runs without a daemon and doesn't require root privileges. Commands are nearly identical — if you know `docker run`, you know `podman run`.

**Zellij vs tmux:** Zellij is a modern terminal multiplexer like tmux or screen. The key feature: your terminal sessions persist even when you disconnect. Close your laptop, SSH drops, terminal crashes? Your work is still there when you reconnect.

**Agents:** AI coding assistants like Claude Code and OpenCode that help you write, debug, and understand code. Agent Session creates isolated container environments for these agents so they have consistent, reproducible workspaces every time.
