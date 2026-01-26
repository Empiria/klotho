# Project Research Summary

**Project:** agent-session (Multi-Agent CLI Container Orchestrator)
**Domain:** Development tooling for containerized AI coding agent environments
**Researched:** 2026-01-26
**Confidence:** HIGH

## Executive Summary

The agent-session tool is a container-based session orchestrator for AI coding agents, currently supporting Claude Code with plans to add multi-agent support (OpenCode, custom agents). Research reveals this domain follows a clear pattern: **simple shell-based orchestration with modern UI tooling beats complex rewrites**. The recommendation is to enhance the existing Bash foundation with Gum for interactive menus, ShellCheck/shfmt for quality, and BATS for testing, while introducing an agent abstraction layer through config files and multi-stage Containerfiles.

The critical insight: this tool bridges container management with terminal multiplexer sessions specifically for AI coding workflows—it's not trying to be general-purpose. The architecture should separate orchestration concerns (host script manages containers/sessions) from agent-specific concerns (config files define installation, paths, commands). Multi-stage Containerfile builds with ARG selection create agent-specific images without bloat.

Key risks center on **the transition from personal tool to team tool**: hardcoded paths/usernames (blocks immediate adoption), undocumented environmental assumptions (creates support burden), and documentation drift (erodes trust). Secondary risks involve multi-agent architecture pitfalls—specifically the "God Agent" antipattern where monolithic agents handle everything instead of specialized agents for focused tasks. Prevention requires pre-release audits for portability, comprehensive prerequisite documentation, and designing agent abstraction before implementing the second agent.

## Key Findings

### Recommended Stack

The research strongly recommends **staying with Bash and enhancing it** rather than rewriting to Go/Python/Rust. The existing Bash script works, and for a tool that orchestrates Podman containers and manages Zellij sessions with a small team (2-5 users), the complexity of rewriting buys nothing. The modern Bash ecosystem has excellent tooling that makes scripts professional-quality.

**Core technologies:**
- **Bash 5.0+**: Orchestration logic — already working, native container/session management, perfect for gluing Unix tools
- **Gum 0.14+**: Interactive UI components — modern, beautiful menus without complexity, single binary with 13 ready-to-use commands
- **ShellCheck 0.10+**: Static analysis — industry standard linter, catches bugs before runtime, essential for professional Bash
- **shfmt 3.8+**: Code formatting — enforces consistent style, integrates with CI/CD
- **BATS 1.11+**: Testing framework — TAP-compliant, test Bash like you test web apps

**When to consider rewriting:**
- Go: Only if need Windows support, single-binary distribution, or script exceeds 500 lines with complex data structures (3-5x dev time)
- Python: Only if need complex JSON/YAML parsing or Python library integration (2-3x dev time)
- Rust: Only for embedded systems or sub-millisecond performance (10x dev time, overkill)

**Distribution approach:** Git clone + `make install` with optional `curl | bash` for quick setup. No build toolchain needed, transparent for security-conscious users, easy to maintain.

### Expected Features

Features divide into **table stakes** (users expect these), **differentiators** (delight users), and **anti-features** (explicit avoid).

**Must have (table stakes):**
- Start/stop/restart containers — core container lifecycle management
- List running containers/sessions — visibility into what's active
- Interactive selection menu — modern CLIs use fzf-style selection (typing exact names is friction)
- Help text (--help, -h) — universal CLI expectation with examples
- Clear error messages — actionable errors with suggestions, not stack traces
- Session attachment — reattach to running Zellij sessions (like docker exec or tmux attach)
- Container cleanup — remove stopped containers, prevent accumulation
- Multi-agent support — Claude, OpenCode, future agents via config-driven definitions
- Basic status visibility — quick "is my agent running?" check

**Should have (competitive):**
- Auto-restart on failure — Podman `--restart=unless-stopped` policy for reliability
- Logs access — quick debugging without docker commands (`logs <agent>`)
- Dry-run mode — transparency for learning and validation (`--dry-run` shows planned commands)
- Verbose mode — show underlying podman commands for debugging (`-v` or `--verbose`)
- Shell completion — tab completion for agent names and commands, professional feel
- Bulk operations — start/stop all agents at once (`--all` flags)

**Defer to v2+:**
- Resource usage display (ASCII graphs, docker stats output) — useful but not blocking adoption
- Configuration profiles (dev vs production setups) — add when users request different configurations
- Health checks (HTTP endpoint verification) — add when automation needs emerge
- Port conflict detection — nice-to-have, can be manual initially
- Session templates (pre-configured Zellij layouts) — optimize after usage patterns emerge

**Anti-features (explicitly avoid):**
- Full TUI like lazydocker — overengineering for small team, adds maintenance burden
- Database for state — containers already track state, adds unnecessary dependency
- User authentication — small team trust boundary, auth adds complexity without security gain
- Plugin system — premature, no evidence of need, adds API surface and testing burden
- GUI/Web UI — scope creep, users are CLI-comfortable developers
- Remote container management — SSH/remote contexts are complex (auth, networking, latency)
- Complex dependency graphs — don't replicate docker-compose, keep agents independent

### Architecture Approach

The recommended architecture follows a **container-based session orchestrator with agent abstraction** pattern. The key principle: **separate orchestration (host script) from agent definitions (config files) from container builds (multi-stage Containerfile)**.

**Major components:**

1. **Host orchestration script (agent-session)** — Container lifecycle management via Podman, session naming and reattachment, volume mount construction, agent-aware naming (`agent-${AGENT}-${SESSION}`)

2. **Agent definition files (agents/*.conf)** — Bash-sourceable config files defining agent metadata, installation commands, config paths, runtime invocation, initialization hooks. This makes adding new agents ~20 minutes: copy template, fill in installation command/paths, add Containerfile stage, test.

3. **Multi-stage Containerfile** — Base stage with common tools (Bash, Fish, Zellij, Starship, uv), agent-specific stages (FROM base AS agent-claude, FROM base AS agent-opencode), ARG-based final stage selection. BuildKit skips unused branches, so only selected agent builds.

4. **Agent-agnostic entrypoint** — Initialization script that handles generic config setup from environment variables, runs agent-specific init commands, detects which agent is present for backward compatibility.

**Key pattern:** Don't make containers multi-agent (image bloat, dependency conflicts). Make orchestration agent-aware through config-driven selection and build-time branching.

**Migration path:** Phase 1 extracts agent definition for Claude (no breaking changes), Phase 2 adds multi-stage Containerfile (same image, different structure), Phase 3 adds second agent (OpenCode) to validate abstraction, Phase 4 adds interactive selection.

### Critical Pitfalls

Research identified 12 domain pitfalls, 5 critical severity:

1. **Hardcoded Personal Paths and Usernames** — Personal tools evolve with hardcoded assumptions (`/home/owen/`, absolute paths) that fail for colleagues. Prevention: grep audit for username/paths, use `$HOME` and `${XDG_CONFIG_HOME}` patterns, script-relative paths for bundled resources. This blocks adoption immediately if not caught.

2. **Secrets and API Keys in Shared Code** — API keys, tokens, credentials accumulated in config files or scripts leak when sharing. Prevention: pre-release secret scanning (gitleaks, trufflehog), require individual API keys per team member (mount `~/.claude.json` from user's home), never log or echo secrets, verify .gitignore. Security incident risk.

3. **Undocumented Environmental Assumptions** — Tool works for creator because their machine has specific packages/environment variables/configs that aren't documented. Colleagues encounter "command not found: zellij" failures. Prevention: test on fresh container, document ALL prerequisites explicitly with verification commands and per-distro installation instructions, fail fast with helpful messages and links, document platform differences (macOS, WSL2, Linux). Blocks adoption.

4. **The "God Agent" Antipattern** — Building monolithic agent handling all tasks instead of specialized agents. Leads to context pollution (15x more tokens used), poor tool selection, cascading failures. Prevention: identify specialization opportunities (code generation vs debugging vs research), design for agent composition, start simple but monitor for context pollution, document when NOT to use multi-agent. Impacts core architecture.

5. **Documentation Becomes Outdated Before Release** — Docs written at project start diverge from actual implementation by release time. More harmful than no docs. Prevention: write docs last after features stabilize, test documentation like code (script that follows README step-by-step), include expected command output in docs, review docs during code review. Blocks adoption and erodes trust.

**Moderate pitfalls** include shared container state causing conflicts (prevention: immutable images, version tags, document rebuild process), poor agent framework extensibility (prevention: design abstraction early, validate with Claude + opencode implementation), interactive-only CLI without non-interactive fallback (prevention: always provide --agent flag alongside interactive mode, detect terminal type), and undocumented environment variables (prevention: create .env.example, document in --help, validate early).

## Implications for Roadmap

Based on research, suggested phase structure:

### Phase 1: Release Audit & Portability
**Rationale:** Before any colleague adoption, must eliminate personal tool assumptions. All critical pitfalls around hardcoded paths, secrets, and environmental assumptions must be resolved first. This is a quality gate, not feature development.

**Delivers:** Portable, secure codebase ready for team distribution. No hardcoded personal paths, no secrets in code, all prerequisites documented, tested on fresh environment.

**Addresses (from PITFALLS.md):**
- Pitfall 1: Hardcoded paths/usernames
- Pitfall 2: Secrets in code
- Pitfall 3: Undocumented environmental assumptions

**Actions:**
- Grep audit for username (`owen`) and absolute paths (`/home/owen/`)
- Run secret scanning tool (gitleaks or trufflehog)
- Replace hardcoded paths with `$HOME` and `${XDG_CONFIG_HOME}` patterns
- Test on fresh Debian bookworm-slim container
- Document all prerequisites with verification commands
- Add dependency checks to script with helpful error messages
- Verify .gitignore excludes secrets

**Avoids:** Immediate adoption blockers, security incidents, "works on my machine" syndrome.

### Phase 2: Documentation & Quick Start
**Rationale:** With portable code, documentation becomes source of truth. Must be written AFTER features stabilize (not before) and tested with fresh-eye colleague. Documentation quality determines adoption rate.

**Delivers:** Comprehensive documentation with 5-minute quick start, tested prerequisites, .env.example, command examples with expected output.

**Addresses (from PITFALLS.md):**
- Pitfall 5: Outdated documentation
- Pitfall 9: Environment variables without examples
- Pitfall 11: No onboarding quick win
- Pitfall 12: Platform-specific assumptions

**Uses (from STACK.md):**
- Documentation follows Bash best practices
- Installation via Makefile with standard targets
- Prerequisites clearly listed (Podman 4.0+, Bash 5.0+, Zellij)

**Actions:**
- Write README with Quick Start section (goal: < 5 minutes to first success)
- Create .env.example for all environment variables
- Document all command-line flags in --help output
- Test documentation on fresh machine with colleague
- Document platform support (Linux tested distros, macOS with Podman Desktop, WSL2)
- Include expected command output in examples
- Document container rebuild workflow

**Avoids:** Documentation drift, support burden from undocumented config, slow adoption.

### Phase 3: Agent Abstraction & Multi-Agent Foundation
**Rationale:** Before implementing second agent (OpenCode), design and validate the agent abstraction layer. This prevents pitfall of retrofitting abstraction after hardcoding Claude-specific logic. Design with two agents forces proper abstraction.

**Delivers:** Agent definition format (agents/*.conf files), multi-stage Containerfile with ARG selection, agent-agnostic entrypoint, CLI flag for agent selection.

**Addresses (from PITFALLS.md):**
- Pitfall 4: God Agent antipattern (design for specialization from start)
- Pitfall 7: Poor agent framework extensibility
- Pitfall 10: Inconsistent naming conventions

**Implements (from ARCHITECTURE.md):**
- Agent abstraction pattern with Bash-sourceable config files
- Multi-stage Containerfile (base + agent-claude + agent-opencode stages)
- Agent-agnostic entrypoint accepting ENV vars
- Container naming: `agent-${AGENT}-${SESSION}`

**Uses (from FEATURES.md):**
- Multi-agent support (table stakes feature)
- Agent selection via --agent flag (non-interactive mode)

**Actions:**
- Create agents/ directory and agent definition format
- Extract Claude config to agents/claude.conf
- Refactor Containerfile into multi-stage with base + agent-specific stages
- Add --agent flag to agent-session script
- Update container naming to include agent type
- Create agents/opencode.conf definition
- Build both claude-agent and opencode-agent images
- Test switching between agents

**Avoids:** Hardcoded agent logic, poor extensibility, God Agent antipattern.

### Phase 4: Interactive Selection & Polish
**Rationale:** With solid agent abstraction, add user-friendly interactive selection without sacrificing scriptability. Gum provides modern UI without complexity.

**Delivers:** Interactive agent selection menu (Gum-based), --list-agents command, agent descriptions in help text, both interactive and non-interactive modes working.

**Addresses (from PITFALLS.md):**
- Pitfall 8: Interactive-only CLI without non-interactive fallback

**Uses (from STACK.md):**
- Gum for interactive menus (auto-install if missing)
- fzf as optional enhancement for fuzzy search

**Uses (from FEATURES.md):**
- Interactive selection menu (table stakes)
- Dry-run mode (differentiator)
- Verbose mode (differentiator)
- Help text with examples

**Actions:**
- Install/check for Gum in agent-session script
- Implement interactive agent selection with Gum choose
- Add --list-agents flag showing available agents
- Detect terminal type (interactive vs non-interactive)
- Add --dry-run flag showing planned commands
- Add --verbose/-v flag for debugging
- Update help text with environment variables and examples
- Generate help text dynamically from agent configs

**Avoids:** Broken scripting/automation, inconsistent UX.

### Phase 5: Quality Tooling & Testing
**Rationale:** With features stable, add professional-quality tooling for maintainability. Testing ensures reliability as tool evolves. This phase prevents technical debt accumulation.

**Delivers:** ShellCheck integration, shfmt formatting, BATS test suite, Makefile with standard targets, CI/CD pipeline, pre-commit hooks.

**Uses (from STACK.md):**
- ShellCheck for static analysis
- shfmt for code formatting
- BATS for testing framework
- Make for standard interface

**Uses (from FEATURES.md):**
- Non-zero exit codes (table stakes) — validated by tests
- Error messages (table stakes) — tested for common failure modes

**Actions:**
- Add ShellCheck to development workflow
- Add shfmt for consistent formatting
- Write BATS tests for agent selection, container lifecycle, error handling
- Create Makefile with install/uninstall/test/lint/format targets
- Add pre-commit hook running ShellCheck
- Set up CI pipeline (GitHub Actions or GitLab CI)
- Document development workflow in CONTRIBUTING.md

**Avoids:** Bash script bugs, inconsistent style, broken changes.

### Phase 6: Table Stakes Features
**Rationale:** Core container lifecycle features needed for daily usage. These are expected by users and blocking adoption if missing.

**Delivers:** Stop/restart commands, list/status commands, session attachment, container cleanup, logs access.

**Uses (from FEATURES.md):**
- Start/stop/restart containers (table stakes)
- List running containers/sessions (table stakes)
- Session attachment (table stakes)
- Container cleanup (table stakes)
- Logs access (differentiator, but easy to implement)
- Basic status visibility (table stakes)

**Actions:**
- Implement stop command (podman stop)
- Implement restart command (stop + start)
- Implement list command (podman ps with filtering)
- Implement status command (check container state)
- Implement attach command (zellij attach to session)
- Implement rm/clean command (remove stopped containers)
- Implement logs command (podman logs -f)
- Add --all flag for bulk operations

**Avoids:** Feature gaps that cause users to fall back to raw podman commands.

### Phase 7: Differentiators & Polish
**Rationale:** Features that improve experience beyond baseline. Not blocking adoption, but increase tool value and professional feel.

**Delivers:** Auto-restart policy, shell completion, upgrade command, bulk operations enhanced, session templates (optional).

**Uses (from FEATURES.md):**
- Auto-restart on failure (differentiator)
- Shell completion (differentiator)
- Upgrade command (differentiator)
- Bulk operations (differentiator)
- Session templates (differentiator, optional)

**Actions:**
- Add --restart=unless-stopped to container config
- Generate bash/zsh/fish completion scripts
- Implement upgrade command (pull latest image, restart container)
- Enhance bulk operations (start/stop all with filtering)
- (Optional) Create Zellij layout templates per agent type
- Document advanced usage patterns

**Avoids:** Feature creep — these are nice-to-have, implement after core is solid.

### Phase Ordering Rationale

**Dependencies discovered:**
- Phase 1 (audit) MUST come before any distribution — security and portability blockers
- Phase 2 (docs) requires Phase 1 complete — can't document broken/insecure tool
- Phase 3 (agent abstraction) MUST come before Phase 4 (interactive selection) — interactive menu needs abstraction to work with
- Phase 5 (quality tooling) should come after features stabilize (Phase 3-4) but before feature expansion (Phase 6-7) — establishes quality baseline

**Architecture-driven grouping:**
- Phases 1-2 focus on portability and documentation (prerequisites for team tool)
- Phases 3-4 focus on multi-agent architecture (core value proposition)
- Phases 5-7 focus on quality and feature completeness (professional tool)

**Pitfall avoidance:**
- Phase 1 directly addresses critical pitfalls 1-3 (hardcoded paths, secrets, environment assumptions)
- Phase 2 directly addresses critical pitfall 5 (outdated documentation) and moderate pitfall 9 (undocumented env vars)
- Phase 3 directly addresses critical pitfall 4 (God Agent) and moderate pitfall 7 (poor extensibility)
- Phase 4 directly addresses moderate pitfall 8 (interactive-only CLI)
- Phase 5 prevents accumulation of technical debt and catches Bash pitfalls early

**Parallel opportunities:**
- Phase 6 and 7 could be developed in parallel with different developers (core features vs polish)
- Within Phase 6, individual commands (stop, restart, list, logs) are independent and could be parallelized

### Research Flags

Phases likely needing deeper research during planning:

- **Phase 3 (Agent Abstraction):** Complex integration between Bash, multi-stage Containerfile, and agent definitions. May need specific research on Containerfile ARG patterns with BuildKit, Bash config file parsing best practices, and Podman image tagging strategies. However, architecture research already provides detailed patterns.

- **Phase 4 (Interactive Selection):** Gum usage patterns are straightforward, but may need research on terminal detection edge cases, fzf integration if fuzzy search desired, or alternative menu libraries if Gum unavailable on target platforms.

Phases with standard patterns (skip research-phase):

- **Phase 1 (Release Audit):** Standard security scanning and portability patterns, well-documented in sources. Execution-focused, not research-needed.

- **Phase 2 (Documentation):** Standard documentation best practices, template-driven. More about testing docs than researching.

- **Phase 5 (Quality Tooling):** ShellCheck, shfmt, BATS are well-documented with standard integration patterns. Stack research already covers usage.

- **Phase 6 (Table Stakes Features):** Standard Podman CLI commands (stop, restart, ps, logs). No research needed, just implementation.

- **Phase 7 (Differentiators):** Standard completion generation, restart policies, container operations. Well-documented in Podman/Bash docs.

**Overall:** Most phases have clear patterns from research. Phase 3 is the highest-complexity phase architecturally but already has detailed patterns in ARCHITECTURE.md. No phase requires extensive new research.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | Multiple official sources (Bash best practices, Gum documentation, ShellCheck/shfmt/BATS official docs). Recommendation based on clear criteria (tool simplicity, team size, existing codebase). Alternative stacks (Go/Python/Rust) also researched with clear decision criteria. |
| Features | HIGH | CLI best practices from Command Line Interface Guidelines (clig.dev), container management patterns from official Docker/Podman documentation, feature categorization validated against similar tools (lazydocker, podman-compose). Anti-features identified from software anti-pattern literature. |
| Architecture | HIGH | Agent abstraction pattern verified with official sources (Docker multi-stage builds, asdf plugin patterns, Google ADK agent configs). Alternative patterns evaluated and rejected with clear rationale. Migration path provides low-risk incremental approach. |
| Pitfalls | HIGH | Critical pitfalls verified with official security sources (OWASP, GitGuardian), Anthropic multi-agent guidance, CLI UX case studies. Phase-specific warnings map directly to suggested roadmap phases. All pitfalls include detection criteria and prevention strategies. |

**Overall confidence:** HIGH

Research methodology used current (2026) sources, cross-referenced multiple authorities, verified claims against official documentation where available, and provided specific implementation guidance rather than generic advice. All four research dimensions (Stack, Features, Architecture, Pitfalls) reached HIGH confidence with concrete, actionable recommendations.

### Gaps to Address

While overall confidence is high, some areas need validation during implementation:

- **Platform testing**: Research covers portability patterns, but actual testing on macOS and different Linux distributions hasn't been performed. Phase 2 should include testing on target platforms if team uses macOS.

- **Gum availability**: Stack research recommends Gum for interactive menus, but installation strategy (system package manager vs binary download) may vary by platform. Phase 4 should validate auto-install approach across target platforms.

- **Multi-agent coordination**: Research identifies the God Agent antipattern, but actual team usage may reveal specific coordination needs (e.g., "use Claude for initial generation, OpenCode for iteration"). Phase 3-4 implementation should monitor for these patterns and document discovered workflows.

- **Zellij session integration**: Architecture assumes Zellij session naming convention works with multi-agent setup. Phase 3 should validate that session names remain unique and reattachment works correctly when multiple agent types have sessions with same name.

- **Container image size**: Multi-stage Containerfile design prevents bloat, but actual image sizes should be monitored. If base image + agent exceeds 1GB, may need optimization (smaller base image like Alpine, though Debian bookworm-slim is already minimal).

- **Team-specific workflow variations**: Small team context (2-5 developers) may have specific needs not covered in general research. Early phases should include feedback loops to identify team-specific requirements before implementing all features.

**Mitigation approach:** All gaps are validation-focused rather than fundamental unknowns. Each gap maps to a specific phase where validation should occur. No gap blocks initial implementation; they represent "verify assumption during execution" items rather than "research before proceeding" blockers.

## Sources

### Primary (HIGH confidence)

**Stack Research:**
- [Gum GitHub Repository](https://github.com/charmbracelet/gum) — Interactive menu framework, official documentation
- [ShellCheck GitHub Repository](https://github.com/koalaman/shellcheck) — Bash static analysis, official tool
- [BATS Core GitHub Repository](https://github.com/bats-core/bats-core) — Bash testing framework, official documentation
- [Bash Scripting Best Practices 2025](https://medium.com/@prasanna.a1.usage/best-practices-we-need-to-follow-in-bash-scripting-in-2025-cebcdf254768) — Modern Bash patterns
- [Microsoft Bash Code Review Guidelines](https://microsoft.github.io/code-with-engineering-playbook/code-reviews/recipes/bash/) — Enterprise Bash standards

**Features Research:**
- [Command Line Interface Guidelines (clig.dev)](https://clig.dev/) — Authoritative CLI UX patterns
- [Docker Container Lifecycle Documentation](https://last9.io/blog/docker-container-lifecycle/) — Container state management best practices
- [Zellij vs Tmux Comparison](https://rrmartins.medium.com/zellij-vs-tmux-complete-comparison-or-almost-8e5b57d234ae) — Session management patterns

**Architecture Research:**
- [Docker Multi-Stage Builds (Official)](https://docs.docker.com/build/building/multi-stage/) — Containerfile conditional patterns
- [Advanced Multi-Stage Build Patterns](https://medium.com/@tonistiigi/advanced-multi-stage-build-patterns-6f741b852fae) — BuildKit branching with ARG
- [asdf Plugin Creation Guide](https://asdf-vm.com/plugins/create.html) — Command directory patterns for extensibility
- [Google Agent Development Kit: Agent Config](https://google.github.io/adk-docs/agents/config/) — YAML-based agent definition structure

**Pitfalls Research:**
- [OWASP Secrets Management Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Secrets_Management_Cheat_Sheet.html) — Official security guidance
- [GitGuardian Secrets Sprawl Report 2021](https://www.gitguardian.com/state-of-secrets-sprawl-on-github-2021) — 39M+ secrets leaked, industry data
- [Why Multi-Agent LLM Systems Fail (ORQ)](https://orq.ai/blog/why-do-multi-agent-llm-systems-fail) — 79% coordination failure rate
- [Anthropic: When to Use Multi-Agent Systems](https://claude.com/blog/building-multi-agent-systems-when-and-how-to-use-them) — Official guidance from model provider
- [Google's Eight Essential Multi-Agent Design Patterns (InfoQ)](https://www.infoq.com/news/2026/01/multi-agent-design-patterns/) — Google's official patterns

### Secondary (MEDIUM confidence)

**Stack Research:**
- [Bash vs Python vs Go in 2025 Comparison](https://medium.com/@build_break_learn/bash-vs-python-vs-go-2025-scripting-automation-cli-tools-a3934c3aa95b) — Language selection criteria
- [Building Great CLIs in 2025: Comparison](https://medium.com/@no-non-sense-guy/building-great-clis-in-2025-node-js-vs-go-vs-rust-e8e4bf7ee10e) — Framework options
- [Makefile Best Practices 2025](https://paiml.com/blog/2025-01-25-makefiles-modern-development/) — Build system patterns

**Features Research:**
- [10 Best Container Management Tools 2026](https://northflank.com/blog/container-management-tools) — Feature benchmarking
- [10 Design Principles for Delightful CLIs (Atlassian)](https://www.atlassian.com/blog/it-teams/10-design-principles-for-delightful-clis) — UX patterns
- [7 Modern CLI Tools You Must Try in 2026](https://medium.com/the-software-journal/7-modern-cli-tools-you-must-try-in-2026-c4ecab6a9928) — Feature trends

**Architecture Research:**
- [AWS CLI Agent Orchestrator](https://aws.amazon.com/blogs/opensource/introducing-cli-agent-orchestrator-transforming-developer-cli-tools-into-a-multi-agent-powerhouse/) — Multi-agent orchestration patterns
- [OpenCode vs Claude Code Architecture](https://www.builder.io/blog/opencode-vs-claude-code) — Client/server vs CLI comparison
- [OpenCode Agent Configuration](https://deepwiki.com/anomalyco/opencode/5.1-agent-configuration) — Real-world multi-agent config

**Pitfalls Research:**
- [Developer Onboarding Documentation Mistakes](https://www.multiplayer.app/blog/5-developer-onboarding-documentation-doc-mistakes/) — Documentation pitfalls
- [5 Tips for Managing Environment Variables](https://medium.com/@arunangshudas/5-tips-for-managing-environment-variables-across-environments-8a8216176baf) — Configuration management
- [14 AI Agent Frameworks Compared 2025](https://softcery.com/lab/top-14-ai-agent-frameworks-of-2025-a-founders-guide-to-building-smarter-systems) — Extensibility patterns

### Tertiary (LOW confidence, needs validation)

- [Podman vs Docker 2025 Comparison](https://uptrace.dev/comparisons/podman-vs-docker) — Feature parity claims, should verify during implementation
- [Container Security in 2026 Risks](https://www.cloud4c.com/blogs/container-security-in-2026-risks-and-strategies) — Forward-looking security trends, validate against current threats
- [Containerized Development Environment Pitfalls](https://testdouble.com/insights/the-slippery-slope-of-docker-dev-environments) — Anecdotal experiences, validate with team usage

---
*Research completed: 2026-01-26*
*Ready for roadmap: yes*
