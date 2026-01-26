# Domain Pitfalls

**Domain:** Containerized multi-agent development environment tooling
**Researched:** 2026-01-26
**Confidence:** HIGH (verified with official sources and team collaboration research)

## Executive Summary

Transforming a personal development tool into a team-shared tool introduces three critical failure modes: **the hardcoded assumption trap** (personal paths/config leak into shared code), **the "works on my machine" syndrome** (environmental differences), and **documentation drift** (outdated docs causing more harm than help). Multi-agent support adds **the God Agent antipattern** (monolithic agents instead of specialized ones) and **coordination failures** (79% of multi-agent system breakdowns). These pitfalls are not theoretical—they represent the majority failure cases in 2026.

---

## Critical Pitfalls

Mistakes that cause rewrites, security incidents, or make the tool unusable for colleagues.

### Pitfall 1: Hardcoded Personal Paths and Usernames

**What goes wrong:**
Personal tools evolve with hardcoded assumptions that work only for the original developer. Paths like `/home/owen/`, usernames in config files, and absolute paths to personal directories make the tool fail immediately for colleagues.

**Why it happens:**
Development iteration rewards "make it work now" over "make it portable." Quick fixes like `SOME_PATH=/home/owen/.config` ship to production because they worked locally.

**Consequences:**
- Tool fails on first run for colleagues with different usernames
- Silent failures where wrong paths are created or ignored
- Trust erosion—colleagues assume tool is "Owen's personal thing, not for us"
- Time wasted debugging path issues instead of using the tool

**Prevention:**
1. **Audit pass before release:**
   - Search codebase for your username: `grep -r "owen" .` (case-insensitive)
   - Search for absolute paths: `grep -rE "^/home/[^/]+" .`
   - Check environment variable defaults for personal paths

2. **Use portable path resolution:**
   ```bash
   # BAD
   CONFIG_DIR="/home/owen/.config/agent-session"

   # GOOD
   CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/agent-session"

   # BAD
   CLAUDE_CONFIG="/home/owen/.claude"

   # GOOD
   CLAUDE_CONFIG="${HOME}/.claude"
   ```

3. **Script-relative paths for bundled resources:**
   ```bash
   # Get script directory portably
   SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
   CONFIG_TEMPLATE="$SCRIPT_DIR/templates/config.toml"
   ```

4. **Environment variable validation:**
   ```bash
   # Fail fast if HOME is unset (rare but possible in CI)
   : "${HOME:?HOME environment variable must be set}"
   ```

**Detection:**
- Colleague reports "file not found" on first run
- Different behavior between your machine and theirs
- Config files created in wrong locations
- grep shows hardcoded paths or usernames in code

**Phase assignment:** Phase 1 (Release Audit) - MUST be completed before any distribution

**Severity:** CRITICAL - blocks adoption

---

### Pitfall 2: Secrets and API Keys in Shared Code

**What goes wrong:**
Personal tools accumulate API keys, tokens, and credentials in config files, environment variables, or even hardcoded in scripts. Sharing the tool without scrubbing secrets leaks credentials to colleagues and potentially to version control.

**Why it happens:**
Personal tools start with "it's just for me" mentality where embedding API keys in `.env` or config files is convenient. The transition to team tool forgets these secrets exist.

**Consequences:**
- API keys leaked to colleagues who shouldn't have access
- Secrets accidentally committed to version control
- Security audit failures if tool is open-sourced
- Credential rotation required after leak discovery
- Violation of organizational security policies

**Prevention:**
1. **Pre-release security scan:**
   ```bash
   # Use secret scanning tools
   git ls-files | xargs grep -E '(api[_-]?key|secret|token|password|credential)' -i

   # Tools: gitleaks, trufflehog, detect-secrets
   docker run --rm -v $(pwd):/path zricethezav/gitleaks:latest detect --source="/path" -v
   ```

2. **Require individual API keys per team member:**
   - Never share Claude API keys (Anthropic explicitly prohibits this)
   - Document: "Each team member must provide their own API key"
   - Mount `~/.claude.json` from user's home directory (already implemented)

3. **Environment variable best practices:**
   ```bash
   # Provide .env.example WITHOUT real values
   # .env.example:
   CLAUDE_API_KEY=sk-ant-api03-xxxx...  # Get from https://console.anthropic.com

   # Document in README:
   # 1. Copy .env.example to .env
   # 2. Add your personal API key
   # 3. Never commit .env (verify .gitignore)
   ```

4. **Never log or echo secrets:**
   ```bash
   # BAD
   echo "Using API key: $CLAUDE_API_KEY"

   # GOOD
   echo "Using API key: ${CLAUDE_API_KEY:0:10}..." # Only show prefix
   ```

5. **Verify .gitignore before release:**
   ```bash
   # Add to .gitignore
   .env
   .claude.json
   *.key
   *secret*
   ```

**Detection:**
- Secret scanning tool alerts
- Colleague asks "should I use this API key?"
- Same API key working across multiple users
- Security team audit flags shared credentials

**Phase assignment:** Phase 1 (Release Audit) - MUST scan before any distribution

**Severity:** CRITICAL - security incident risk

**Sources:**
- [OWASP Secrets Management Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Secrets_Management_Cheat_Sheet.html)
- [Best Secret Scanning Tools 2026](https://www.sentinelone.com/cybersecurity-101/cloud-security/secret-scanning-tools/)
- [GitGuardian Secrets Sprawl Report](https://www.gitguardian.com/state-of-secrets-sprawl-on-github-2021) - 39M+ secrets leaked in 2024

---

### Pitfall 3: Undocumented Environmental Assumptions

**What goes wrong:**
Tool works perfectly for the creator because their machine has specific packages, environment variables, or system configurations. Colleagues encounter cryptic failures because documentation doesn't mention these prerequisites.

**Why it happens:**
Your development environment evolved organically—tools installed months ago, shell configs tweaked, environment variables set and forgotten. You don't remember what's essential because it "just works."

**Consequences:**
- Colleagues spend hours debugging missing dependencies
- Tool errors are cryptic: "command not found: zellij"
- Different behavior on different Linux distributions
- Silent failures when optional dependencies missing
- Support burden—constant "how do I install X?" questions

**Prevention:**
1. **Test on fresh environment:**
   ```bash
   # Spin up clean container to verify prerequisites
   podman run -it --rm debian:bookworm-slim bash
   # Try running your tool install steps
   ```

2. **Document ALL prerequisites explicitly:**
   ```markdown
   ## Prerequisites

   ### Required
   - Podman 4.0+ (container runtime)
   - Bash 4.0+ (scripting)
   - curl (downloading installers)

   ### Verification
   ```bash
   podman --version    # Should show 4.0 or higher
   bash --version      # Should show 4.0 or higher
   ```

   ### Installation
   **Debian/Ubuntu:**
   ```bash
   sudo apt install podman curl
   ```

   **Fedora:**
   ```bash
   sudo dnf install podman
   ```
   ```

3. **Fail fast with helpful messages:**
   ```bash
   # Check for required commands
   command -v podman >/dev/null 2>&1 || {
       echo "error: podman not found" >&2
       echo "Install: https://podman.io/getting-started/installation" >&2
       exit 1
   }
   ```

4. **Version compatibility checks:**
   ```bash
   # Check podman version
   PODMAN_VERSION=$(podman --version | grep -oE '[0-9]+\.[0-9]+' | head -1)
   if [[ $(echo "$PODMAN_VERSION < 4.0" | bc) -eq 1 ]]; then
       echo "error: podman 4.0+ required (found $PODMAN_VERSION)" >&2
       exit 1
   fi
   ```

5. **Document platform differences:**
   ```markdown
   ## Platform Notes

   ### macOS
   Podman requires Podman Desktop or podman machine. See: https://podman.io/docs/installation#macos

   ### Windows (WSL2)
   Tested on Ubuntu 22.04 in WSL2. Native Windows not supported.

   ### Linux
   Tested on Debian 12, Ubuntu 22.04, Fedora 38. Should work on any systemd-based distro.
   ```

**Detection:**
- Colleague reports "command not found" errors
- Tool works for you but not others
- Bug reports with stack traces showing missing dependencies
- Questions like "what version of X do I need?"

**Phase assignment:** Phase 2 (Documentation) - Document during setup guide creation

**Severity:** CRITICAL - blocks adoption

**Sources:**
- [Bash Script Portability Best Practices](https://moldstud.com/articles/p-maximize-your-bash-scripts-a-guide-to-portability-across-systems)
- [Shell Script Best Practices 2026](https://linuxvox.com/blog/use-of-read-only-variables-in-shell-scripts/)

---

### Pitfall 4: The "God Agent" Antipattern

**What goes wrong:**
Building a single monolithic agent that handles all tasks instead of specialized agents for different purposes. Leads to context pollution, poor tool selection, and cascading failures.

**Why it happens:**
Starting with one agent (Claude) works fine for personal use. When adding multi-agent support, the temptation is to keep the same pattern—just swap which model runs. This misses the architectural benefit of specialization.

**Consequences:**
- Context pollution where irrelevant information degrades performance
- Poor tool selection when agent has too many tools available
- Cascading failures—one mistake propagates through subsequent steps
- Performance 15× worse than necessary (multi-agent uses 15× more tokens)
- Inability to run tasks in parallel

**Prevention:**
1. **Identify specialization opportunities:**
   - Code generation vs debugging vs testing
   - Research vs implementation vs documentation
   - Quick tasks vs long-running analysis

2. **Design for agent composition:**
   ```bash
   # GOOD: Specialized agents
   agent-session --agent claude-coder   # Code generation specialist
   agent-session --agent claude-researcher  # Research and documentation
   agent-session --agent opencode       # Alternative code model

   # BAD: Single agent doing everything
   agent-session  # Just runs Claude for everything
   ```

3. **Use multi-agent patterns when beneficial:**
   - **Parallel tasks:** Multiple agents work simultaneously
   - **Specialization:** Agent selects best tool for specific task
   - **Context separation:** Prevent unrelated context pollution

4. **Start simple, specialize when needed:**
   - Phase 1: Single agent selection (Claude vs opencode)
   - Phase 2: Monitor for context pollution signs
   - Phase 3: Add specialization if data shows benefit

5. **Document when NOT to use multi-agent:**
   ```markdown
   ## Agent Selection Guide

   **Use single agent when:**
   - Task is self-contained and focused
   - Context is directly relevant
   - Sequential steps with tight coupling

   **Consider multiple agents when:**
   - Tasks can run in parallel
   - Different tasks need different tool sets
   - Context for task A pollutes task B
   ```

**Detection:**
- Agents struggle with large context windows
- Tool selection becomes erratic or wrong
- Same agent handles vastly different task types poorly
- Performance degradation over time in session

**Phase assignment:** Phase 3 (Multi-agent Architecture) - Design agent specialization strategy

**Severity:** CRITICAL - impacts core architecture

**Sources:**
- [Why Multi-Agent LLM Systems Fail](https://orq.ai/blog/why-do-multi-agent-llm-systems-fail)
- [Google's Eight Essential Multi-Agent Design Patterns](https://www.infoq.com/news/2026/01/multi-agent-design-patterns/)
- [When to Use Multi-Agent Systems (Anthropic)](https://claude.com/blog/building-multi-agent-systems-when-and-how-to-use-them)

---

### Pitfall 5: Documentation Becomes Outdated Before Release

**What goes wrong:**
Documentation written at the beginning of the project diverges from the actual implementation. By release time, docs are wrong, incomplete, or misleading—more harmful than no docs.

**Why it happens:**
Documentation is written once and rarely updated during development. Code evolves, features change, command-line flags are renamed—but docs stay frozen in time.

**Consequences:**
- Colleagues follow outdated instructions and get errors
- Trust erodes—"the docs are wrong, don't trust them"
- Support burden increases—"the docs say X but it doesn't work"
- Onboarding becomes frustrating instead of smooth
- Tool appears unmaintained or broken

**Prevention:**
1. **Write docs last, not first:**
   - Phase 1: Build and stabilize features
   - Phase 2: Write documentation when features are final
   - Phase 3: Verify docs match current implementation

2. **Test documentation like code:**
   ```bash
   # Create test script that follows README step-by-step
   # If any step fails, docs are wrong

   # test-documentation.sh
   #!/bin/bash
   set -euo pipefail

   # Follow README installation steps exactly
   # Step 1: Install podman
   command -v podman >/dev/null 2>&1 || exit 1

   # Step 2: Clone repository
   # Step 3: Run agent-session
   # etc.
   ```

3. **Include command output in docs:**
   ```markdown
   # Instead of:
   Run `agent-session -n test`

   # Write:
   Run `agent-session -n test`. You should see:
   ```
   Starting new session 'test'...
   Claude Code 0.3.0 ready
   ```
   ```

4. **Document with version context:**
   ```markdown
   ## Version Compatibility

   These instructions are for agent-session 1.0.0.
   Last updated: 2026-01-26
   ```

5. **Use inline help as source of truth:**
   ```bash
   # Keep --help output synchronized with implementation
   # Generate docs FROM --help, not separately

   # In README generation:
   agent-session --help > docs/help-output.txt
   ```

6. **Review docs during code review:**
   - PR checklist: "Does this change require doc updates?"
   - Reviewer verifies docs match implementation
   - Automated checks for broken links or outdated examples

**Detection:**
- Colleague reports "this command doesn't work"
- Commands in README return errors or unexpected output
- GitHub issues showing people confused by docs
- Multiple support requests for same installation step

**Phase assignment:** Phase 2 (Documentation) - Write docs AFTER features stabilize, verify before release

**Severity:** CRITICAL - blocks adoption and erodes trust

**Sources:**
- [Developer Onboarding Documentation Mistakes](https://www.multiplayer.app/blog/5-developer-onboarding-documentation-doc-mistakes/)
- [CLI Documentation Checklist](https://www.infrasity.com/blog/cli-docs-checklist)
- [Top 7 Code Documentation Best Practices 2026](https://www.qodo.ai/blog/code-documentation-best-practices-2026/)

---

## Moderate Pitfalls

Mistakes that cause delays, technical debt, or reduced usability.

### Pitfall 6: Shared Container State Causes Conflicts

**What goes wrong:**
Multiple team members sharing the same container image without proper isolation causes config conflicts, version mismatches, and "works for me but not you" issues.

**Why it happens:**
Containerization gives false sense that "it's isolated, so it's consistent." But mutable state (installed packages, config files, cached data) accumulates in containers and differs across users.

**Consequences:**
- User A's session works, User B's doesn't
- Tool versions drift between team members
- Config changes by one user affect others unexpectedly
- Debugging becomes "works on my machine" syndrome

**Prevention:**
1. **Immutable container images:**
   - Build container from Containerfile, don't modify running containers
   - Document: "Never `podman exec` to install packages manually"
   - Rebuild image instead of modifying running containers

2. **User-specific mounts for config:**
   ```bash
   # ALREADY IMPLEMENTED CORRECTLY
   -v "$HOME/.claude:/config/.claude:Z"        # User's Claude config
   -v "$HOME/.claude.json:/home/agent/.claude.json:Z"  # User's API key
   ```

3. **Version-tag container images:**
   ```bash
   # Build with version tag
   podman build -t claude-agent:1.0.0 .
   podman tag claude-agent:1.0.0 claude-agent:latest

   # Use specific version in production
   podman run claude-agent:1.0.0
   ```

4. **Document image rebuild process:**
   ```markdown
   ## Updating the Environment

   If you need to update tool versions or add dependencies:

   1. Edit `Containerfile`
   2. Rebuild image: `podman build -t claude-agent .`
   3. Stop old containers: `podman stop agent-*`
   4. Remove old containers: `podman rm agent-*`
   5. Start fresh: `agent-session -n default ~/project`
   ```

**Detection:**
- Colleague's container has different behavior despite "same code"
- Tool version differences between users
- Config changes don't propagate to some users
- "Did you rebuild the container?" becomes common question

**Phase assignment:** Phase 2 (Documentation) - Document container rebuild workflow

**Severity:** MEDIUM - causes confusion and support burden

**Sources:**
- [Containerized Development Environment Pitfalls](https://testdouble.com/insights/the-slippery-slope-of-docker-dev-environments)
- [Container Security Best Practices](https://www.cloud4c.com/blogs/container-security-in-2026-risks-and-strategies)

---

### Pitfall 7: Poor Agent Framework Extensibility

**What goes wrong:**
Building a proprietary agent integration mechanism that makes it hard to add new agents or requires duplicating code for each agent. Results in fragile, hard-to-maintain agent-specific logic.

**Why it happens:**
Starting with Claude-only implementation creates patterns optimized for one agent. Adding second agent (opencode) reveals the abstraction is wrong, but it's too late to refactor cleanly.

**Consequences:**
- Each new agent requires rewriting integration logic
- Agent-specific code scattered throughout codebase
- Testing becomes exponentially harder with each agent
- Can't adopt emerging agent frameworks or standards

**Prevention:**
1. **Design agent abstraction early:**
   ```bash
   # BAD: Hardcoded Claude
   claude --dangerously-skip-permissions

   # GOOD: Agent abstraction
   $AGENT_COMMAND --agent-specific-flags

   # Configuration mapping
   case "$AGENT_TYPE" in
       claude)
           AGENT_COMMAND="claude --dangerously-skip-permissions"
           ;;
       opencode)
           AGENT_COMMAND="opencode --workspace /workspace"
           ;;
   esac
   ```

2. **Use configuration files for agent definitions:**
   ```bash
   # ~/.config/agent-session/agents.toml
   [claude]
   command = "claude"
   args = ["--dangerously-skip-permissions"]
   setup_script = "claude-setup.sh"

   [opencode]
   command = "opencode"
   args = ["--workspace", "/workspace"]
   setup_script = "opencode-setup.sh"
   ```

3. **Implement agent plugin system:**
   ```bash
   # agents/claude/init.sh
   # agents/opencode/init.sh
   # agents/AGENT_NAME/init.sh

   # Load agent from plugin directory
   source "agents/$AGENT_TYPE/init.sh"
   ```

4. **Consider standard protocols:**
   - Model Context Protocol (MCP) for tool access standardization
   - Avoid creating yet another proprietary agent format

5. **Start with two agents to validate abstraction:**
   - Implementing Claude + opencode forces proper abstraction
   - Don't optimize for single agent, then retrofit

**Detection:**
- Copy-paste code for each new agent
- Agent-specific logic scattered in multiple files
- Can't add agent without modifying core scripts
- Testing requires running actual agents (no mocking)

**Phase assignment:** Phase 3 (Multi-agent Support) - Design abstraction before implementing second agent

**Severity:** MEDIUM - creates technical debt and limits extensibility

**Sources:**
- [Extensibility in AI Agent Frameworks](https://www.gocodeo.com/post/extensibility-in-ai-agent-frameworks-hooks-plugins-and-custom-logic)
- [14 AI Agent Frameworks Compared](https://softcery.com/lab/top-14-ai-agent-frameworks-of-2025-a-founders-guide-to-building-smarter-systems)

---

### Pitfall 8: Interactive CLI Without Non-Interactive Fallback

**What goes wrong:**
Building interactive prompts for agent selection without providing command-line flags for automation. Breaks scripting, CI/CD integration, and power user workflows.

**Why it happens:**
Interactive menus feel user-friendly during design. Developer focuses on "good UX" for beginners, forgets that advanced users and scripts need non-interactive mode.

**Consequences:**
- Tool can't be scripted or automated
- CI/CD pipelines can't use the tool
- Power users frustrated by forced interactive prompts
- Can't integrate with other tools via pipes

**Prevention:**
1. **Always provide flag-based alternatives:**
   ```bash
   # Interactive mode
   agent-session -n test ~/project  # Prompts for agent choice

   # Non-interactive mode
   agent-session -n test --agent claude ~/project  # No prompts
   ```

2. **Detect non-interactive terminal:**
   ```bash
   if [[ -t 0 ]]; then
       # stdin is a terminal - interactive mode OK
       select_agent_interactive
   else
       # stdin is not a terminal - require flags
       [[ -z "$AGENT_TYPE" ]] && {
           echo "error: --agent required in non-interactive mode" >&2
           exit 1
       }
   fi
   ```

3. **Support environment variables:**
   ```bash
   # Allow configuration via env vars
   AGENT_TYPE="${AGENT_TYPE:-claude}"

   # Command-line flag overrides env var
   while [[ $# -gt 0 ]]; do
       case $1 in
           --agent)
               AGENT_TYPE="$2"
               shift 2
               ;;
       esac
   done
   ```

4. **Document both modes:**
   ```markdown
   ## Usage

   ### Interactive Mode
   ```bash
   agent-session -n test ~/project
   # Prompts for agent selection
   ```

   ### Non-Interactive Mode
   ```bash
   agent-session -n test --agent claude ~/project
   # No prompts, suitable for scripts
   ```

   ### Environment Variable
   ```bash
   export AGENT_TYPE=opencode
   agent-session -n test ~/project  # Uses opencode
   ```
   ```

5. **Never require prompts in error paths:**
   ```bash
   # BAD
   echo "Choose action: [r]etry or [q]uit"
   read -r action

   # GOOD
   echo "error: operation failed" >&2
   echo "Retry with --retry flag or check logs at /path/to/log" >&2
   exit 1
   ```

**Detection:**
- Tool hangs in CI/CD or scripts
- Error: "Inappropriate ioctl for device" when piping
- Users request --yes or --no-prompt flags
- Can't automate common workflows

**Phase assignment:** Phase 3 (Multi-agent Support) - Implement flag-based agent selection alongside interactive mode

**Severity:** MEDIUM - limits automation and power users

**Sources:**
- [Command Line Interface Guidelines](https://clig.dev/)
- [UX Patterns for CLI Tools](https://lucasfcosta.com/2022/06/01/ux-patterns-cli-tools.html)
- [10 Design Principles for Delightful CLIs](https://www.atlassian.com/blog/it-teams/10-design-principles-for-delightful-clis)

---

### Pitfall 9: Environment Variables Without .env.example

**What goes wrong:**
Team tool uses environment variables for configuration but doesn't document them or provide templates. Colleagues don't know what to set, use wrong values, or skip configuration entirely.

**Why it happens:**
Developer sets env vars once in their shell and forgets they exist. Documentation mentions "set SOME_VAR" but doesn't explain what values are valid or why it's needed.

**Consequences:**
- Colleagues run tool with missing or wrong env vars
- Silent failures when optional env vars affect behavior
- Support burden: "What environment variables do I need?"
- Trial-and-error to discover configuration options

**Prevention:**
1. **Provide .env.example file:**
   ```bash
   # .env.example - Copy to .env and customize

   # Agent type: claude, opencode
   AGENT_TYPE=claude

   # Extra paths to mount (colon-separated)
   # Example: /home/user/dotfiles:/home/user/scripts
   AGENT_SESSION_MOUNTS=

   # Claude API key location (default: ~/.claude.json)
   # CLAUDE_CONFIG_PATH=$HOME/.claude.json
   ```

2. **Document environment variables in --help:**
   ```bash
   show_help() {
       cat << 'EOF'
   Environment variables:
     AGENT_TYPE              Agent to use (claude, opencode). Default: claude
     AGENT_SESSION_MOUNTS    Extra paths to mount (colon-separated)
                             Example: /home/user/dotfiles:/home/user/scripts
   EOF
   }
   ```

3. **Use consistent naming convention:**
   ```bash
   # Good: Scoped prefix
   AGENT_SESSION_MOUNTS
   AGENT_SESSION_CONFIG
   AGENT_SESSION_LOG_LEVEL

   # Bad: Generic names
   MOUNTS
   CONFIG
   LOG_LEVEL
   ```

4. **Validate environment variables early:**
   ```bash
   # Validate AGENT_TYPE if set
   if [[ -n "${AGENT_TYPE:-}" ]]; then
       case "$AGENT_TYPE" in
           claude|opencode)
               ;;
           *)
               echo "error: invalid AGENT_TYPE='$AGENT_TYPE'" >&2
               echo "Valid values: claude, opencode" >&2
               exit 1
               ;;
       esac
   fi
   ```

5. **Use zod/envalid-style validation:**
   ```bash
   # Define expected env vars with types and defaults
   AGENT_TYPE="${AGENT_TYPE:-claude}"  # String, default claude
   DEBUG="${DEBUG:-false}"             # Boolean, default false
   MAX_RETRIES="${MAX_RETRIES:-3}"     # Number, default 3

   # Validate boolean
   [[ "$DEBUG" =~ ^(true|false)$ ]] || {
       echo "error: DEBUG must be 'true' or 'false'" >&2
       exit 1
   }
   ```

**Detection:**
- Colleagues ask "what environment variables exist?"
- Tool behaves differently for different users unexpectedly
- Support requests show wrong env var values
- Documentation mentions env vars but no examples

**Phase assignment:** Phase 2 (Documentation) - Create .env.example and document all env vars

**Severity:** MEDIUM - causes confusion and support burden

**Sources:**
- [Environment Variable Management Best Practices](https://humanitec.com/blog/how-to-manage-environment-variables)
- [5 Tips for Managing Environment Variables](https://medium.com/@arunangshudas/5-tips-for-managing-environment-variables-across-environments-8a8216176baf)
- [How Dev Teams Keep Environment Variables Synchronized](https://dev.to/espoir/how-do-you-keep-your-environment-variable-synchronized-among-your-development-team-175a)

---

## Minor Pitfalls

Mistakes that cause annoyance but are fixable without major changes.

### Pitfall 10: Inconsistent Command Naming and Behavior

**What goes wrong:**
Command-line flags, agent names, or session names use inconsistent conventions. Some use kebab-case, some snake_case, some camelCase. Behavior differs unexpectedly between similar operations.

**Why it happens:**
Commands and flags added incrementally without style guide. Each addition follows whatever felt right at the time.

**Consequences:**
- Users forget correct flag names
- Tab completion becomes less useful
- Documentation harder to write and maintain
- Tool feels unprofessional or unpolished

**Prevention:**
1. **Establish naming convention early:**
   ```bash
   # Command-line flags: kebab-case
   --agent-type
   --session-name
   --config-file

   # Environment variables: SCREAMING_SNAKE_CASE with prefix
   AGENT_SESSION_MOUNTS
   AGENT_SESSION_CONFIG

   # Agent names: lowercase
   claude
   opencode

   # Session names: allow alphanumeric + dash
   frontend
   backend-api
   ```

2. **Document conventions:**
   ```markdown
   ## Naming Conventions

   - Flags: kebab-case (--agent-type, --config-file)
   - Environment variables: AGENT_SESSION_* prefix
   - Agent names: lowercase (claude, opencode)
   - Session names: alphanumeric + dash
   ```

3. **Validate inputs match conventions:**
   ```bash
   # Session name validation
   [[ ! "$SESSION_NAME" =~ ^[a-zA-Z0-9-]+$ ]] && {
       echo "error: session name must be alphanumeric + dash" >&2
       exit 1
   }
   ```

4. **Be consistent with inputs and outputs:**
   ```bash
   # If --agent flag accepts "claude", output should say "claude"
   # Don't accept "claude" but output "Claude Code" inconsistently
   ```

**Detection:**
- Users typo flag names frequently
- Documentation shows inconsistent examples
- Tab completion suggestions are confusing
- Support requests show confusion about naming

**Phase assignment:** Phase 3 (Multi-agent Support) - Establish conventions when adding agent selection

**Severity:** LOW - affects polish and usability

**Sources:**
- [Command Line Interface Guidelines](https://clig.dev/)
- [CLI User Experience Case Study](https://www.tweag.io/blog/2023-10-05-cli-ux-in-topiary/)

---

### Pitfall 11: No Onboarding Quick-Win Experience

**What goes wrong:**
First-time user experience requires too much setup before seeing value. Installation takes 30 minutes, configuration is complex, first command fails with error.

**Why it happens:**
Developer knows the tool deeply and doesn't experience it fresh. Documentation written in logical order (architecture, concepts, advanced usage) instead of task order (quick start, common tasks).

**Consequences:**
- Colleagues give up during installation
- Tool perceived as "too complicated" before seeing value
- Slow adoption—people try it once and abandon
- Support burden during initial setup

**Prevention:**
1. **Design for 5-minute quick win:**
   ```markdown
   ## Quick Start

   Get running in 5 minutes:

   ```bash
   # 1. Install (one command)
   curl -sSL https://example.com/install.sh | bash

   # 2. Start session (defaults work)
   agent-session

   # 3. See it work
   # You're now in a containerized Claude Code environment!
   # Type: claude
   ```

   That's it! You're ready to code with Claude in an isolated environment.
   ```

2. **Provide sensible defaults:**
   ```bash
   # Don't require configuration for basic usage
   SESSION_NAME="${SESSION_NAME:-default}"
   AGENT_TYPE="${AGENT_TYPE:-claude}"
   PROJECT_PATH="${PROJECT_PATH:-$(pwd)}"
   ```

3. **Structure docs for tasks, not concepts:**
   ```markdown
   # GOOD order
   1. Quick Start (5 min to first success)
   2. Common Tasks (what most users need)
   3. Configuration (when defaults aren't enough)
   4. Advanced Usage (power users)
   5. Architecture (curious developers)

   # BAD order
   1. Architecture Overview
   2. Design Philosophy
   3. Configuration Reference
   4. Installation
   5. Usage Examples
   ```

4. **Test first-time experience:**
   - Have colleague follow docs on fresh machine
   - Time how long to first successful command
   - Note every point of confusion or error
   - Iterate until < 5 minutes to success

**Detection:**
- Colleagues don't finish installation
- Support requests during initial setup
- Slow adoption despite tool being useful
- Documentation starts with advanced concepts

**Phase assignment:** Phase 2 (Documentation) - Design quick-start experience, test with colleague

**Severity:** LOW - affects adoption rate but not functionality

**Sources:**
- [Developer Onboarding Best Practices](https://www.cortex.io/post/developer-onboarding-guide)
- [Seven Deadly Sins of Developer Onboarding](https://developerrelations.com/talks/the-seven-deadly-sins-of-developer-onboarding/)

---

### Pitfall 12: Platform-Specific Script Assumptions

**What goes wrong:**
Bash scripts use GNU-specific flags or Linux-specific paths that break on macOS or other Unix systems.

**Why it happens:**
Development happens on one platform (Linux). Platform-specific commands work locally, so portability issues aren't discovered until colleague tries different OS.

**Consequences:**
- Tool fails on macOS with cryptic errors
- Some features work, others don't—inconsistent
- Requires platform-specific workarounds
- Documentation becomes complex with per-platform instructions

**Prevention:**
1. **Use portable shebang:**
   ```bash
   # GOOD - finds bash in PATH
   #!/usr/bin/env bash

   # BAD - assumes bash location
   #!/bin/bash
   ```

2. **Avoid GNU-specific flags:**
   ```bash
   # BAD - GNU sed specific
   sed -i 's/foo/bar/' file.txt

   # GOOD - portable
   sed 's/foo/bar/' file.txt > file.txt.tmp && mv file.txt.tmp file.txt

   # OR: detect platform
   if [[ "$OSTYPE" == "darwin"* ]]; then
       sed -i '' 's/foo/bar/' file.txt  # macOS
   else
       sed -i 's/foo/bar/' file.txt      # Linux
   fi
   ```

3. **Test on multiple platforms:**
   ```bash
   # CI matrix testing
   test:
     runs-on:
       - ubuntu-latest
       - macos-latest
   ```

4. **Document platform support:**
   ```markdown
   ## Platform Support

   ✓ Linux (tested: Debian 12, Ubuntu 22.04, Fedora 38)
   ✓ macOS (tested: macOS 13 Ventura, requires Podman Desktop)
   ✗ Windows (use WSL2 with Ubuntu)
   ```

5. **Use portable path resolution:**
   ```bash
   # Use $HOME not ~/ in scripts
   CONFIG_DIR="$HOME/.config"  # Works everywhere

   # Use command substitution for script directory
   SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
   ```

**Detection:**
- macOS users report "illegal option" errors
- Commands work on Linux but fail on macOS
- Path separators cause issues
- Features work inconsistently across platforms

**Phase assignment:** Phase 2 (Documentation) - Test on macOS if team uses it, document platform support

**Severity:** LOW - affects portability but workarounds exist

**Sources:**
- [Maximize Bash Script Portability](https://moldstud.com/articles/p-maximize-your-bash-scripts-a-guide-to-portability-across-systems)
- [Bash Portable Shebangs](https://thelinuxcode.com/absolute-path-script-bash/)

---

## Phase-Specific Warnings

Which pitfalls are most likely to occur in each milestone phase.

| Phase | Likely Pitfalls | Mitigation Strategy |
|-------|----------------|---------------------|
| **Phase 1: Release Audit** | Pitfall 1 (Hardcoded paths)<br>Pitfall 2 (Secrets)<br>Pitfall 3 (Environment assumptions) | Run grep audit for username/paths<br>Use secret scanning tools<br>Test on fresh container<br>Document all prerequisites |
| **Phase 2: Documentation** | Pitfall 5 (Outdated docs)<br>Pitfall 9 (Env var documentation)<br>Pitfall 11 (No quick win) | Write docs AFTER features stabilize<br>Create .env.example<br>Design 5-minute quick start<br>Test with colleague on fresh machine |
| **Phase 3: Multi-agent Support** | Pitfall 4 (God Agent antipattern)<br>Pitfall 7 (Poor extensibility)<br>Pitfall 8 (No non-interactive mode)<br>Pitfall 10 (Inconsistent naming) | Design agent abstraction early<br>Implement Claude + opencode to validate<br>Provide --agent flag alongside interactive<br>Establish naming conventions |
| **Phase 4: Interactive CLI** | Pitfall 8 (Interactive-only)<br>Pitfall 10 (Inconsistent UX) | Support both interactive and flag-based modes<br>Detect terminal type<br>Use consistent flag naming<br>Follow CLI best practices |

---

## Research Quality Assessment

**Methodology:**
- Web searches performed with year 2026 for current information
- Cross-referenced multiple sources for critical claims
- Verified against official documentation where available
- Specific codebase concerns from CONCERNS.md incorporated

**Confidence by category:**

| Category | Confidence | Evidence |
|----------|------------|----------|
| Hardcoded paths and secrets | HIGH | Multiple official sources (OWASP, GitGuardian, security tools) |
| Multi-agent architecture pitfalls | HIGH | Anthropic official blog, Google design patterns, academic research |
| CLI UX best practices | HIGH | Command Line Interface Guidelines, multiple UX case studies |
| Environment variable management | MEDIUM | Dev community best practices, multiple tool documentation |
| Documentation pitfalls | HIGH | Developer onboarding research, team collaboration studies |
| Platform portability | MEDIUM | Bash scripting guides, community best practices |

**Known gaps:**
- Platform-specific testing not performed (only researched)
- Multi-agent coordination patterns are emerging (2026 research still evolving)
- Team-specific workflow variations may introduce pitfalls not covered

**Sources verification:**
- Critical security claims verified with OWASP and security tool documentation
- Multi-agent architecture patterns verified with Anthropic and Google official sources
- CLI UX patterns verified with Command Line Interface Guidelines (clig.dev)
- Environment and container patterns verified with official tool documentation

---

## Summary of Critical Actions

**Before any release or sharing with colleagues:**

1. **Security audit (Phase 1):**
   - [ ] Grep for personal username in codebase
   - [ ] Run secret scanning tool (gitleaks, trufflehog)
   - [ ] Verify .gitignore excludes secrets
   - [ ] Test with fresh API key (not yours)

2. **Portability audit (Phase 1):**
   - [ ] Grep for hardcoded absolute paths
   - [ ] Replace with $HOME or portable variables
   - [ ] Test on fresh container/machine
   - [ ] Document all prerequisites explicitly

3. **Documentation audit (Phase 2):**
   - [ ] Write docs AFTER features stabilize
   - [ ] Test docs on fresh machine with colleague
   - [ ] Verify command examples actually work
   - [ ] Create .env.example for all env vars

4. **Architecture audit (Phase 3):**
   - [ ] Design agent abstraction before implementing second agent
   - [ ] Implement both CLI flags AND interactive mode
   - [ ] Establish naming conventions
   - [ ] Test agent switching actually works

**Quality gate:** Tool should install and run successfully for a colleague who has never seen it before, with < 5 minutes from docs to first successful command.

---

## Sources

### Security and Secrets Management
- [OWASP Secrets Management Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Secrets_Management_Cheat_Sheet.html)
- [Best Secret Scanning Tools 2026](https://www.sentinelone.com/cybersecurity-101/cloud-security/secret-scanning-tools/)
- [GitGuardian Secrets Sprawl Report](https://www.gitguardian.com/state-of-secrets-sprawl-on-github-2021)
- [Best Practices for API Key Safety (OpenAI)](https://help.openai.com/en/articles/5112595-best-practices-for-api-key-safety)
- [API Key Management Best Practices](https://www.legitsecurity.com/aspm-knowledge-base/api-key-security-best-practices)

### Multi-Agent Systems
- [Why Multi-Agent LLM Systems Fail (ORQ)](https://orq.ai/blog/why-do-multi-agent-llm-systems-fail)
- [Google's Eight Essential Multi-Agent Design Patterns (InfoQ)](https://www.infoq.com/news/2026/01/multi-agent-design-patterns/)
- [When to Use Multi-Agent Systems (Anthropic)](https://claude.com/blog/building-multi-agent-systems-when-and-how-to-use-them)
- [How to Build Multi-Agent Systems: Complete 2026 Guide](https://dev.to/eira-wexford/how-to-build-multi-agent-systems-complete-2026-guide-1io6)
- [Extensibility in AI Agent Frameworks](https://www.gocodeo.com/post/extensibility-in-ai-agent-frameworks-hooks-plugins-and-custom-logic)
- [14 AI Agent Frameworks Compared](https://softcery.com/lab/top-14-ai-agent-frameworks-of-2025-a-founders-guide-to-building-smarter-systems)

### CLI User Experience
- [Command Line Interface Guidelines](https://clig.dev/)
- [UX Patterns for CLI Tools](https://lucasfcosta.com/2022/06/01/ux-patterns-cli-tools.html)
- [CLI User Experience Case Study: Topiary](https://www.tweag.io/blog/2023-10-05-cli-ux-in-topiary/)
- [10 Design Principles for Delightful CLIs (Atlassian)](https://www.atlassian.com/blog/it-teams/10-design-principles-for-delightful-clis)
- [CLI Documentation Checklist](https://www.infrasity.com/blog/cli-docs-checklist)

### Documentation and Onboarding
- [Developer Onboarding Documentation Mistakes](https://www.multiplayer.app/blog/5-developer-onboarding-documentation-doc-mistakes/)
- [Developer Onboarding: Checklist & Best Practices](https://www.cortex.io/post/developer-onboarding-guide)
- [Top 7 Code Documentation Best Practices 2026](https://www.qodo.ai/blog/code-documentation-best-practices-2026/)
- [Seven Deadly Sins of Developer Onboarding](https://developerrelations.com/talks/the-seven-deadly-sins-of-developer-onboarding/)

### Environment and Configuration Management
- [How to Manage Environment Variables (Humanitec)](https://humanitec.com/blog/how-to-manage-environment-variables)
- [5 Tips for Managing Environment Variables](https://medium.com/@arunangshudas/5-tips-for-managing-environment-variables-across-environments-8a8216176baf)
- [How Dev Teams Keep Environment Variables Synchronized](https://dev.to/espoir/how-do-you-keep-your-environment-variable-synchronized-among-your-development-team-175a)
- [Environment Variables: 4 Critical Best Practices](https://configu.com/blog/environment-variables-how-to-use-them-and-4-critical-best-practices/)

### Containerization and Team Sharing
- [Using Docker for Local Development: A Practical Guide](https://testdouble.com/insights/the-slippery-slope-of-docker-dev-environments)
- [Containerized Development Environments](https://www.getambassador.io/blog/containerized-development-environments-build-faster)
- [Container Security in 2026](https://www.cloud4c.com/blogs/container-security-in-2026-risks-and-strategies)

### Bash Script Portability
- [Maximize Bash Script Portability](https://moldstud.com/articles/p-maximize-your-bash-scripts-a-guide-to-portability-across-systems)
- [Shell Script Best Practices: Read-Only Variables](https://linuxvox.com/blog/use-of-read-only-variables-in-shell-scripts/)
- [Understanding Absolute and Relative Paths](https://www.linuxbash.sh/post/understanding-absolute-and-relative-paths)

### Dotfiles and Configuration Sharing
- [Manage Team and Personal Dotfiles Together with rcm](https://robots.thoughtbot.com/manage-team-and-personal-dotfiles-together-with-rcm)
- [Dotfiles Management](https://mitxela.com/projects/dotfiles_management)
- [chezmoi - Manage your dotfiles](https://www.chezmoi.io/)
