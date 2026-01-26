# Feature Landscape

**Domain:** CLI tools for container/session management (AI coding agents)
**Researched:** 2026-01-26

## Table Stakes

Features users expect. Missing = tool feels incomplete or colleagues won't adopt.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| **Start/Stop/Restart containers** | Core container lifecycle management. Industry standard for Docker/Podman CLIs. | Low | `start <agent>`, `stop <agent>`, `restart <agent>` commands |
| **List running containers/sessions** | Users need visibility into what's currently active. Standard across all container tools. | Low | `list` or `ps` command showing agent name, status, uptime |
| **Interactive selection menu** | For small teams, typing exact names is friction. Modern CLIs use fzf-style selection (zellij, lazygit pattern). | Medium | Arrow key navigation, fuzzy search. Libraries: fzf, gum, or bash select |
| **Help text (--help, -h)** | Universal CLI expectation. Help should print to stdout and show usage for all commands. | Low | Each command needs `--help` output with examples |
| **Clear error messages** | Users need actionable errors, not stack traces. "Container 'claude' not found. Run 'tool list' to see available agents" not "Error: nil pointer" | Medium | Catch common failures (container doesn't exist, already running, port conflict) with helpful suggestions |
| **Non-zero exit codes** | Scripts and automation depend on proper exit codes (0 = success, 1+ = failure). | Low | Return appropriate codes for success/failure |
| **Session attachment** | Users expect to attach to running sessions (like `docker exec` or `tmux attach`). Zellij sessions need reattachment. | Low | `attach <agent>` command to reconnect to Zellij session |
| **Container cleanup** | Old/stopped containers accumulate. Users expect `docker rm` equivalent. | Low | `rm <agent>` or `clean` command to remove stopped containers |
| **Multi-agent support** | Explicit requirement. Users need Claude, opencode, and future agents. | Medium | Config-driven agent definitions (image, ports, volumes per agent type) |
| **Basic status visibility** | "Is my agent running?" Users need quick status check without parsing docker ps output. | Low | `status <agent>` showing running/stopped/not found |

## Differentiators

Features that improve experience but aren't expected. Build these to delight users, not to meet baseline.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| **Auto-restart on failure** | Container crashes shouldn't require manual intervention. Increases reliability. | Low | Podman `--restart=unless-stopped` policy in container config |
| **Resource usage display** | "Is my agent using too much memory?" Helpful for debugging slow agents. | Medium | ASCII graphs like lazydocker, or simple `docker stats` output |
| **Configuration profiles** | Different setups per user (ports, volumes, resource limits). "Dev" vs "production" profiles. | Medium | Config file with profiles, switch with `--profile=<name>` |
| **Logs access** | Quick access to agent logs without docker commands. Debugging tool. | Low | `logs <agent>` streaming container logs (tail -f equivalent) |
| **Port conflict detection** | Prevents cryptic "bind: address already in use" errors. Auto-suggest next available port. | Medium | Check port availability before starting, suggest alternatives |
| **Health checks** | Is the agent actually responding, or just container running? Useful for automation. | Medium | HTTP health endpoint check, exit code indicates health |
| **Bulk operations** | Start/stop all agents at once. Useful when rebooting or switching contexts. | Low | `start --all`, `stop --all` flags |
| **Shell completion** | Tab completion for agent names and commands. Reduces typing, feels professional. | Medium | Bash/Zsh completion scripts. Generate with library or static file |
| **Upgrade command** | Update agent images without manual docker pull. Convenience feature. | Low | `upgrade <agent>` pulls latest image and restarts container |
| **Session templates** | Pre-configured Zellij layouts per agent type. Different panes for logs, editor, shell. | Medium | Template Zellij layout files per agent, loaded on start |
| **Dry-run mode** | See what would happen without executing. Good for learning and validation. | Low | `--dry-run` flag showing planned docker/podman commands |
| **Verbose mode** | Show underlying docker/podman commands for transparency and debugging. | Low | `--verbose` or `-v` flag printing actual commands executed |

## Anti-Features

Features to explicitly NOT build. Common mistakes or overengineering for small team context.

| Anti-Feature | Why Avoid | What to Do Instead |
|--------------|-----------|-------------------|
| **Full TUI (lazydocker-style)** | Overengineering for small team. Adds maintenance burden, testing complexity. Interactive menu is enough. | Use simple interactive selection for agent choice. Keep commands CLI-first. |
| **Database for state** | Containers already track state. Adding DB introduces dependency, backup complexity, sync issues. | Query container runtime (podman ps) for current state. Stateless tool. |
| **User authentication** | Small team = trust boundary. Auth adds complexity without security gain (local tool, shared infra). | Rely on system user permissions and container runtime access control. |
| **Plugin system** | Premature. No evidence of need. Adds API surface, versioning complexity, testing burden. | Hard-code agent types. Easy to add new agents by editing config. |
| **GUI/Web UI** | Scope creep. Users are developers comfortable with CLI. Web UI adds server, ports, security concerns. | Keep it CLI. Let users use existing tools (Zellij, lazydocker) for visual needs. |
| **Remote container management** | SSH/remote docker contexts are complex (auth, networking, latency). Local-first keeps it simple. | Focus on local Podman. Users can SSH + run tool remotely if needed. |
| **Config auto-sync** | Git is the sync mechanism. Building custom sync adds conflict resolution, versioning nightmares. | Store config in repo. Users commit/pull changes explicitly. |
| **Metrics/Analytics** | Small team doesn't need usage tracking. Privacy concern, data storage burden. | If needed later, users can parse logs. Don't build dashboards. |
| **Complex dependency graph** | Assuming agents depend on each other (service A before B) adds orchestration complexity like docker-compose. | Keep agents independent. If dependencies exist, document manual order. |
| **Container composition** | Don't try to replicate docker-compose. That's already solved. Focus on single-agent management. | One container per agent session. Use docker-compose if multi-container needed. |
| **Automatic image building** | Building images on the fly (Dockerfile in config) adds CI/CD complexity to a simple tool. | Assume images exist (pre-built). Document build process separately. |
| **Session sharing/collaboration** | Real-time pair programming features (screen sharing, concurrent editing) are complex (CRDT, websockets). | Agents are single-user sessions. Use tmux/Zellij native sharing if needed. |

## Feature Dependencies

```
Core Lifecycle
  ├─ start → requires multi-agent config
  ├─ stop → requires status check
  └─ restart → depends on stop + start

Interactive Selection
  └─ requires list functionality

Session Attachment
  └─ requires Zellij session tracking (container label or name convention)

Cleanup
  └─ requires list stopped containers

Configuration Profiles
  └─ requires config file parsing

Logs/Status/Health
  └─ all depend on container existence check
```

## MVP Recommendation

For colleague adoption (minimize friction, maximize utility):

### Phase 1: Core Table Stakes (Week 1)
1. **Multi-agent config** - Define Claude, opencode agents (image, ports, volumes)
2. **Start/stop/restart** - Basic lifecycle commands
3. **List/status** - Visibility into running agents
4. **Session attachment** - Connect to Zellij session
5. **Help text** - `--help` for all commands with examples

### Phase 2: Usability (Week 2)
6. **Interactive selection** - fzf-style menu for agent choice
7. **Error handling** - Friendly messages for common failures
8. **Cleanup command** - Remove stopped containers
9. **Logs access** - Quick debugging without docker commands

### Phase 3: Polish (Week 3)
10. **Dry-run mode** - Transparency for learning
11. **Auto-restart policy** - Reliability improvement
12. **Shell completion** - Professional feel

Defer to post-MVP:
- **Resource usage**: Useful but not blocking adoption
- **Configuration profiles**: Add when users request different setups
- **Health checks**: Add when automation needs emerge
- **Port conflict detection**: Nice-to-have, can be manual initially
- **Session templates**: Optimize after usage patterns emerge

## Small Team Context

**Why some "standard" features aren't table stakes here:**

- **Authentication**: 2-5 developers, shared trust, local tool
- **Metrics**: Small scale, manual observation sufficient
- **Remote management**: SSH works, don't over-complicate
- **Backup/restore**: Containers are ephemeral, config in git

**Why some features ARE more important for small teams:**

- **Interactive selection**: Typing exact names is friction when you have 2-5 agents total
- **Clear documentation**: No dedicated tooling team, needs to be obvious
- **Error messages with suggestions**: No support team, tool must teach itself
- **Dry-run mode**: Builds trust when adopting new tool

## Comparison to Similar Tools

| Feature Category | lazydocker | podman-compose | This Tool |
|-----------------|------------|----------------|-----------|
| **Interface** | Full TUI | CLI only | CLI + interactive select |
| **Scope** | All containers | Compose stacks | AI agent sessions |
| **Session integration** | No | No | Zellij built-in |
| **Target user** | Visual preference | Docker-compose users | Small dev teams |
| **Complexity** | High (full UI) | Medium (YAML) | Low (simple config) |

**Key differentiation**: This tool bridges container management with terminal multiplexer sessions specifically for AI coding workflows. Not trying to be general-purpose container tool.

## Sources

**Container Management Research:**
- [10 best container management tools to simplify deployment in 2026](https://northflank.com/blog/container-management-tools)
- [16 Most Useful Container Orchestration Tools in 2026](https://spacelift.io/blog/container-orchestration-tools)
- [Podman vs Docker: Complete 2026 Comparison Guide](https://www.xurrent.com/blog/podman-vs-docker-complete-2025-comparison-guide-for-devops-teams)
- [Docker Container Lifecycle: Key States and Best Practices](https://last9.io/blog/docker-container-lifecycle/)

**CLI Design & UX:**
- [Command Line Interface Guidelines](https://clig.dev/)
- [CLI Help pages | BetterCLI.org](https://bettercli.org/design/cli-help-page/)
- [10 design principles for delightful CLIs](https://www.atlassian.com/blog/it-teams/10-design-principles-for-delightful-clis)
- [Error Handling in CLI Tools: A Practical Pattern](https://medium.com/@czhoudev/error-handling-in-cli-tools-a-practical-pattern-thats-worked-for-me-6c658a9141a9)

**Interactive CLI Tools:**
- [13 CLI Tools Every Developer Should Master in 2025](https://itsfoss.gitlab.io/post/13-cli-tools-every-developer-should-master-in-2025/)
- [7 Modern CLI Tools You Must Try in 2026](https://medium.com/the-software-journal/7-modern-cli-tools-you-must-try-in-2026-c4ecab6a9928)

**Session Management:**
- [Zellij: The Impressions of a Casual tmux User](https://keyholesoftware.com/zellij-the-impressions-of-a-casual-tmux-user/)
- [Zellij vs Tmux: Complete Comparison](https://rrmartins.medium.com/zellij-vs-tmux-complete-comparison-or-almost-8e5b57d234ae)

**Similar Tools:**
- [Discover the Magic of Lazygit, Lazydocker, and LazyVim](https://jsingizi.medium.com/discover-the-magic-of-lazygit-lazydocker-and-lazyvim-b68f02a794ff)
- [Lazygit Turns 5: Musings on Git, TUIs, and Open Source](https://jesseduffield.com/Lazygit-5-Years-On/)

**Anti-Patterns:**
- [Top 5 Software Anti Patterns to Avoid](https://www.bairesdev.com/blog/software-anti-patterns/)
- [Top Software Anti-Patterns to Avoid in Development](https://medium.com/@chirag.dave/top-software-anti-patterns-to-avoid-in-development-c9791b603a35)

**Small Team Tools:**
- [Best Project Management Tools for Engineering Teams in 2026](https://www.shortcut.com/blog/best-project-management-tools-for-engineering-teams-in-2026)
- [8 Developer Tools That Will Boost Your Workflow in 2026](https://dev.to/anthonymax/8-developer-tools-that-will-boost-your-workflow-in-2026-3gp8k)

**AI Coding Agents:**
- [Best AI Coding Agents for 2026: Real-World Developer Reviews](https://www.faros.ai/blog/best-ai-coding-agents-2026)
- [Compare 50+ AI Agent Tools in 2026](https://research.aimultiple.com/ai-agent-tools/)

**Configuration Management:**
- [Basic Configuration | google-gemini/gemini-cli](https://deepwiki.com/google-gemini/gemini-cli/2.3-configuration)
- [Managing gcloud CLI configurations](https://docs.cloud.google.com/sdk/docs/configurations)
