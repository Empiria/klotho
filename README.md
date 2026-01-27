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

## Commands

### start

<details>
<summary>Create a new session or attach to existing one</summary>

**Syntax:**
```
agent-session start [-a AGENT] [-n NAME] [project-paths...]
```

**Options:**
- `-a, --agent AGENT` - Agent to use (default: "claude")
- `-n, --name NAME` - Session name (default: "default")
- `-h, --help` - Show help message

**Examples:**
```bash
agent-session start
```
Start a new session with the current directory.

```bash
agent-session start ~/projects/webapp
```
Start a new session with a specific project.

```bash
agent-session start -n frontend ~/projects/webapp
```
Start a named session called "frontend".

```bash
agent-session start -n fullstack ~/frontend ~/backend ~/shared-libs
```
Start a session with multiple repositories mounted.

```bash
agent-session start -a opencode ~/project
```
Start a session using the OpenCode agent.

```bash
agent-session start -n frontend
```
Reattach to an existing session named "frontend" (no path needed when session exists).

**Notes:**
- Sessions persist across terminal disconnects — press `Ctrl+C` or close your terminal to detach
- If no `-a` flag provided, shows an interactive menu of available agents
- Omit the session name to reattach to the "default" session
- Environment variables `AGENT_SESSION_MOUNTS` and `AGENT_SESSION_KOB` available for advanced use cases (see `agent-session start --help`)

</details>

### stop

<details>
<summary>Stop a running session</summary>

**Syntax:**
```
agent-session stop [SESSION_NAME]
```

**Arguments:**
- `SESSION_NAME` - Name of session to stop (default: "default")

**Options:**
- `-h, --help` - Show help message

**Examples:**
```bash
agent-session stop
```
Stop the "default" session.

```bash
agent-session stop frontend
```
Stop the session named "frontend".

**Notes:**
- Stopping an already-stopped session succeeds silently (idempotent)
- Stopped sessions can be restarted with `agent-session restart`
- Use `agent-session ls` to see session status

</details>

### restart

<details>
<summary>Restart a stopped session and reattach</summary>

**Syntax:**
```
agent-session restart [SESSION_NAME]
```

**Arguments:**
- `SESSION_NAME` - Name of session to restart (default: "default")

**Options:**
- `-h, --help` - Show help message

**Examples:**
```bash
agent-session restart
```
Restart the "default" session.

```bash
agent-session restart frontend
```
Restart the session named "frontend".

**Notes:**
- If session is already running, simply reattaches (no restart needed)
- Use `agent-session ls` to see session status

</details>

### ls

<details>
<summary>List all sessions with their status</summary>

**Syntax:**
```
agent-session ls
```

**Options:**
- `-h, --help` - Show help message

**Output columns:**
- `NAME` - Session name
- `AGENT` - Agent type (claude, opencode, etc.)
- `STATUS` - running (green) or stopped (red)

**Example output:**
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

**Syntax:**
```
agent-session rm [-f|--force] [SESSION_NAME]
```

**Arguments:**
- `SESSION_NAME` - Name of session to remove (default: "default")

**Options:**
- `-f, --force` - Skip confirmation prompt
- `-h, --help` - Show help message

**Examples:**
```bash
agent-session rm frontend
```
Remove the "frontend" session with confirmation prompt.

```bash
agent-session rm -f frontend
```
Remove the "frontend" session without confirmation.

**Notes:**
- Cannot remove a running session — stop it first with `agent-session stop`
- Removal is permanent — container and its state are deleted
- Use `agent-session ls` to see which sessions exist

</details>

## Troubleshooting

### "podman: command not found"

**Cause:** Podman is not installed or not in your PATH.

**Solution:**

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

**Verify installation:**
```bash
podman --version
```

Expected: `podman version 4.x.x` or higher

### "Cannot connect to Podman" or "unable to connect to Podman socket" (macOS)

**Symptom:** Error message when trying to run podman commands on macOS.

**Cause:** The podman machine is not running. On macOS, Podman runs containers inside a lightweight Linux VM that must be started before use.

**Solution:**
```bash
podman machine start
```

**Verify:**
```bash
podman machine list
```

You should see a running machine in the output.

### "Error: unable to look up current user" or UID mapping errors

**Symptom:** Errors related to user ID mapping or rootless container configuration.

**Cause:** Podman's rootless setup is incomplete or misconfigured.

**Solution:**

1. Check that your user has subordinate UID/GID ranges configured:
```bash
grep $USER /etc/subuid
grep $USER /etc/subgid
```

Each should show a range like `username:100000:65536`.

2. If missing, add them (requires root):
```bash
sudo usermod --add-subuids 100000-165535 --add-subgids 100000-165535 $USER
```

3. Run the migration command:
```bash
podman system migrate
```

**Verify:**
```bash
podman run --rm alpine id
```

Should show uid and gid mappings without errors.

### "session 'X' not found"

**Symptom:** Error message when trying to stop, restart, or remove a session.

**Cause:** The session doesn't exist, or there's a typo in the session name.

**Solution:**
```bash
agent-session ls
```

This shows all available sessions and their current status.

**Verify:** Check that the session name matches exactly (case-sensitive).

### "cannot remove running session"

**Symptom:** Error when trying to `agent-session rm` a session.

**Cause:** You're trying to remove a session that's currently running.

**Solution:**

Stop the session first:
```bash
agent-session stop SESSION_NAME
```

Then remove it:
```bash
agent-session rm SESSION_NAME
```

**Verify:**
```bash
agent-session ls
```

The session should show as "stopped" before removal.

### "Image not built. Build now?"

**Symptom:** Prompt appears when trying to start a session for the first time.

**Cause:** The container image for the selected agent hasn't been built yet.

**Solution:**

Option 1 - Answer 'y' to the prompt and the image will be built automatically.

Option 2 - Build manually before starting:
```bash
./scripts/build.sh claude
```

or for OpenCode:
```bash
./scripts/build.sh opencode
```

**Note:** First build takes 2-3 minutes to download and configure all tools.

**Verify:**
```bash
podman images | grep agent-session
```

You should see `agent-session-claude` and/or `agent-session-opencode` in the output.

### Container fails to start or won't attach

**Symptom:** Session starts but immediately exits, or `agent-session start` hangs.

**Cause:** Missing required credentials or configuration files.

**Solution:**

1. Verify Claude API credentials exist:
```bash
test -f ~/.claude.json && echo "Found" || echo "Missing"
```

If missing, create it (see Prerequisites section).

2. Check that the file has correct permissions:
```bash
ls -l ~/.claude.json
```

Should show `-rw-------` (600 permissions).

3. Verify the JSON format is valid:
```bash
cat ~/.claude.json
```

Should look like:
```json
{
  "apiKey": "your-api-key-here"
}
```

**Verify:**
```bash
agent-session start
```

Should create and attach to the session without errors.
