# Technology Stack

**Project:** CLI/Container Agent Environment Tooling
**Domain:** Interactive CLI for multi-agent container orchestration
**Researched:** 2026-01-26
**Confidence:** HIGH

## Executive Summary

**RECOMMENDATION: Stay with Bash + Gum**

For this use case (small team tool, Podman container orchestration, interactive agent selection), the right stack is:
- **Keep Bash** for orchestration logic
- **Add Gum** for interactive menus
- **Add ShellCheck + shfmt** for code quality
- **Add BATS** for testing

**Rationale:** Your existing Bash script works. The complexity of rewriting to Go/Python/Rust buys you nothing for a tool that orchestrates Podman containers and manages Zellij sessions. The ecosystem has excellent modern tooling for making Bash scripts professional-quality and user-friendly.

**Do NOT rewrite unless:** You need to distribute to Windows users, or you need complex data structure manipulation beyond what Bash can handle.

---

## Recommended Stack

### Core Language: Bash 5.x+

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| Bash | 5.0+ | Orchestration logic | Already have working code, native container/session management, perfect for gluing Unix tools |
| Gum | 0.14+ | Interactive UI components | Modern, beautiful menus without complexity - single binary, 13 ready-to-use commands |
| ShellCheck | 0.10+ | Static analysis | Industry standard linter, catches bugs before runtime |
| shfmt | 3.8+ | Code formatting | Enforces consistent style, integrates with CI/CD |
| BATS | 1.11+ | Testing framework | TAP-compliant, test Bash like you test web apps |

### Supporting Tools

| Tool | Version | Purpose | When to Use |
|------|---------|---------|-------------|
| fzf | 0.57+ | Fuzzy finding (optional) | If users need to search long lists of agents/containers |
| make | GNU Make 4.x | Installation/setup | Standard interface for install/uninstall/test |

---

## Installation Architecture

### Distribution Method: Git + install.sh

**Primary:** Users clone repo and run `make install`
**Alternative:** `curl -fsSL https://your-repo/install.sh | bash` for quick setup

**Why this approach:**
- Simple for Linux/Unix users (your target audience)
- No dependency hell (Bash is everywhere, Gum is single binary)
- Easy to maintain (no build toolchain required)
- Transparent (users can inspect before running)

### Installation Script Pattern

```bash
#!/usr/bin/env bash
set -euo pipefail

# 1. Check dependencies (bash 5+, podman)
# 2. Download/install Gum if missing
# 3. Copy script to /usr/local/bin or ~/.local/bin
# 4. Set up shell completion if available
```

**Security considerations:**
- Use `#!/usr/bin/env bash` for portability
- Always use `set -euo pipefail` for safety
- Check dependencies before modifying system
- Provide both system-wide and user-local install options
- Wrap entire script in a function to prevent partial execution

---

## Gum: Interactive Menu Framework

### Why Gum Over Alternatives

| Tool | Pros | Cons | Verdict |
|------|------|------|---------|
| **Gum** | Modern, beautiful, 13 commands, single binary, actively maintained | Requires external binary | **RECOMMENDED** |
| bash `select` | Built-in, no dependencies | Ugly, limited functionality, no fuzzy search | Only if zero-dependency required |
| fzf | Powerful fuzzy search, widely known | Single-purpose (just filtering), less UI variety | Good complement to Gum |
| dialog/whiptail | Pre-installed on many systems | Old-school UI, complex syntax | Legacy option |

### Gum Installation for Your Script

**Approach:** Auto-install if missing

```bash
# Check if gum exists, install if not
if ! command -v gum &> /dev/null; then
    echo "Installing gum..."
    # Detect package manager and install
    # OR download binary directly from GitHub releases
fi
```

### Gum Usage Patterns for Multi-Agent Selection

```bash
# Agent selection menu
AGENT=$(gum choose "claude-sonnet-4.5" "claude-opus-4.5" "opencode" "custom")

# Confirmation dialogs
gum confirm "Launch $AGENT in new container?" || exit 0

# Input for custom parameters
MEMORY=$(gum input --placeholder "Memory limit (default: 4GB)")

# Progress indicators
gum spin --title "Starting container..." -- podman run ...

# Styled output
gum style --foreground 212 --border-foreground 212 --border double \
    --padding "1 2" --width 50 "Agent $AGENT launched successfully"
```

**Key Gum commands for your use case:**
- `gum choose` - Agent/container selection
- `gum confirm` - Yes/no prompts
- `gum input` - Collect configuration values
- `gum spin` - Show progress during container startup
- `gum style` - Pretty output formatting
- `gum filter` - Fuzzy search if you have many agents

---

## Code Quality Stack

### ShellCheck: Static Analysis

**Purpose:** Catch common Bash mistakes before runtime

**Integration:**
```bash
# Local development
shellcheck script.sh

# CI/CD
shellcheck **.sh || exit 1

# Pre-commit hook
#!/bin/sh
shellcheck $(git diff --cached --name-only --diff-filter=ACM | grep '\.sh$')
```

**Why essential:** Bash has many gotchas (unquoted variables, incorrect conditionals, etc.). ShellCheck catches 90% of common bugs.

### shfmt: Code Formatting

**Purpose:** Consistent style, readable code

**Usage:**
```bash
# Format files in place
shfmt -w -i 2 **.sh

# Check formatting in CI
shfmt -d -i 2 **.sh
```

**Configuration:** Use 2-space indents (standard for Bash), `-l` for POSIX compliance checking

---

## Testing Strategy

### BATS: Bash Automated Testing System

**Why BATS:** TAP-compliant, integrates with CI/CD, test at the interface level (what users experience)

**Test structure:**
```bash
# test/agent-selection.bats

@test "script shows available agents" {
  run ./your-script --list-agents
  [ "$status" -eq 0 ]
  [[ "$output" =~ "claude-sonnet" ]]
}

@test "script launches container with selected agent" {
  run ./your-script --agent claude-sonnet --test-mode
  [ "$status" -eq 0 ]
  [[ "$output" =~ "Container launched" ]]
}
```

**Helper libraries:**
- `bats-assert` (v2.1.0+) - Better assertion messages
- `bats-support` (v0.3.0+) - Helper functions

**CI Integration:**
```bash
# In GitHub Actions / GitLab CI
- name: Run tests
  run: |
    bats test/*.bats
```

---

## Alternative Stack (When to Upgrade)

### Rewrite to Go If:

**Trigger conditions:**
- Need Windows support (Go cross-compiles easily)
- Script exceeds 500 lines and has complex data structures
- Performance matters (startup time <50ms critical)
- Need to distribute as single binary to non-technical users

**Go Stack:**
| Technology | Purpose |
|------------|---------|
| Cobra | CLI framework (what kubectl uses) |
| Bubble Tea + Lip Gloss | TUI components (same team as Gum) |
| go-podman | Podman API bindings |

**Effort:** 3-5x development time vs enhancing Bash script
**Benefit:** Single binary, better Windows support, faster execution
**Cost:** Build toolchain, cross-compilation complexity, loss of shell script transparency

### Rewrite to Python If:

**Trigger conditions:**
- Need complex JSON/YAML parsing
- Team already maintains Python tooling
- Need to integrate with Python libraries (AI SDKs, etc.)

**Python Stack:**
| Technology | Purpose |
|------------|---------|
| Click or Typer | CLI framework |
| Rich | Terminal formatting |
| podman-py | Podman Python bindings |

**Effort:** 2-3x development time vs enhancing Bash script
**Benefit:** Better data structure handling, easier testing, larger ecosystem
**Cost:** Dependency management (virtualenv/pip), runtime requirement, slower startup

### DO NOT Rewrite to Rust Unless:

**Only if:** You need embedded systems support or sub-millisecond performance
**Cost:** 10x development time, steep learning curve, overkill for this use case

---

## Distribution Checklist

### Essential Components

```
your-agent-tool/
├── script.sh              # Main script
├── install.sh             # Installation script
├── Makefile               # Standard interface
├── README.md              # Documentation
├── test/
│   └── *.bats            # Tests
└── completions/
    ├── bash
    ├── zsh
    └── fish
```

### Makefile Targets

```makefile
.PHONY: install uninstall test lint format

PREFIX ?= /usr/local

install:
	@./install.sh --prefix $(PREFIX)

uninstall:
	rm -f $(PREFIX)/bin/your-script

test:
	bats test/*.bats

lint:
	shellcheck **.sh

format:
	shfmt -w -i 2 **.sh
```

### Installation Options

**System-wide (requires sudo):**
```bash
make install PREFIX=/usr/local
```

**User-local (no sudo):**
```bash
make install PREFIX=~/.local
```

---

## Podman Integration Patterns

### Best Practices for Bash + Podman (2025)

**1. Use Podman's Bash-Friendly CLI**
```bash
# Podman is designed for scripting
podman run --name agent-${AGENT_ID} \
  --env AGENT_TYPE="${AGENT}" \
  --volume "${PWD}:/workspace:z" \
  --detach \
  your-agent-image:latest
```

**2. Check Container Status**
```bash
# Podman returns 0 if container running, non-zero otherwise
if podman container inspect agent-${AGENT_ID} &>/dev/null; then
  echo "Agent already running"
fi
```

**3. Systemd Integration (Optional)**
```bash
# Generate systemd service for long-running agents
podman generate systemd --name agent-${AGENT_ID} \
  > ~/.config/systemd/user/agent-${AGENT_ID}.service
systemctl --user enable agent-${AGENT_ID}
```

**4. Kubernetes Compatibility**
```bash
# If users want to move to k8s later
podman generate kube agent-${AGENT_ID} > agent.yaml
```

**Why Bash for Podman:** Podman's CLI is designed for scripting. You're not fighting the tool - you're using it as intended. Go/Python bindings exist but add complexity without benefit.

---

## Complexity Decision Matrix

**Use this to decide when to rewrite:**

| Metric | Stay Bash | Consider Go/Python | Definitely Rewrite |
|--------|-----------|-------------------|-------------------|
| Lines of code | < 500 | 500-1000 | > 1000 |
| Data structures | Simple vars/arrays | Nested arrays/maps | Complex hierarchies |
| Target OS | Linux/Unix only | Linux + macOS | Windows required |
| Distribution | Git clone OK | Need .deb/.rpm | Need single binary |
| Team Bash skill | Comfortable | Mixed | Python/Go preferred |
| Startup time | < 500ms OK | < 100ms needed | < 50ms critical |

**Your project:** All metrics point to "Stay Bash" zone

---

## Recommended Workflow

### Development Cycle

1. **Write:** Edit .sh files
2. **Format:** `make format` (runs shfmt)
3. **Lint:** `make lint` (runs shellcheck)
4. **Test:** `make test` (runs BATS)
5. **Commit:** Pre-commit hooks run shellcheck

### Pre-commit Hook Setup

```bash
# .git/hooks/pre-commit
#!/bin/bash
shellcheck $(git diff --cached --name-only --diff-filter=ACM | grep '\.sh$')
```

### CI/CD Pipeline

```yaml
# GitHub Actions example
name: Quality
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install shellcheck
        run: sudo apt-get install -y shellcheck
      - name: Install shfmt
        run: go install mvdan.cc/sh/v3/cmd/shfmt@latest
      - name: Install BATS
        run: |
          git clone https://github.com/bats-core/bats-core.git
          cd bats-core && ./install.sh /usr/local
      - name: Lint
        run: make lint
      - name: Test
        run: make test
```

---

## Dependencies Management

### Core Dependencies (Users Must Have)

- Bash 5.0+ (check: `bash --version`)
- Podman 4.0+ (check: `podman --version`)
- Zellij (for session management)

### Optional Dependencies (Script Can Install)

- Gum (download binary if missing)
- fzf (enhance with fuzzy search if available)

### Dependency Check Pattern

```bash
#!/usr/bin/env bash
set -euo pipefail

check_dependencies() {
  local missing=()

  # Check required
  command -v podman >/dev/null || missing+=("podman")
  command -v zellij >/dev/null || missing+=("zellij")

  # Check Bash version
  if [ "${BASH_VERSINFO[0]}" -lt 5 ]; then
    echo "Error: Bash 5.0+ required (you have $BASH_VERSION)"
    exit 1
  fi

  if [ ${#missing[@]} -gt 0 ]; then
    echo "Missing dependencies: ${missing[*]}"
    exit 1
  fi

  # Auto-install Gum if missing
  if ! command -v gum >/dev/null; then
    install_gum
  fi
}
```

---

## Security Best Practices

### Script Security Checklist

- [ ] Use `set -euo pipefail` at top of every script
- [ ] Quote all variables: `"${var}"` not `$var`
- [ ] Use `[[ ]]` not `[ ]` for conditionals
- [ ] Avoid `eval` and `source` of untrusted input
- [ ] Validate user input before using in commands
- [ ] Use ShellCheck to catch common mistakes

### Installation Security

- [ ] Use HTTPS for downloads (not HTTP)
- [ ] Verify checksums for downloaded binaries
- [ ] Wrap install script in function (prevents partial execution)
- [ ] Provide both system and user install options
- [ ] Don't require sudo unless necessary

---

## Sources

### Interactive Menu Tools
- [Gum GitHub Repository](https://github.com/charmbracelet/gum)
- [Gum Overview and Examples 2025](https://best-of-web.builder.io/library/charmbracelet/gum)
- [fzf GitHub Repository](https://github.com/junegunn/fzf)
- [Improving Shell Workflows with fzf](https://seb.jambor.dev/posts/improving-shell-workflows-with-fzf/)
- [Whiptail Interactive Shell Scripts](https://www.linuxfordevices.com/tutorials/shell-script/interactive-shell-scripts-whiptail)
- [Linux Fu: Gum Up Your Script](https://hackaday.com/2023/03/29/linux-fu-gum-up-your-script/)

### CLI Framework Comparisons
- [Building Great CLIs in 2025: Node.js vs Go vs Rust](https://medium.com/@no-non-sense-guy/building-great-clis-in-2025-node-js-vs-go-vs-rust-e8e4bf7ee10e)
- [Bash vs Python vs Go in 2025](https://medium.com/@build_break_learn/bash-vs-python-vs-go-2025-scripting-automation-cli-tools-a3934c3aa95b)
- [When to Rewrite Bash in Go](https://stackoverflow.blog/2022/03/09/rewriting-bash-scripts-in-go-using-black-box-testing/)
- [Rust vs Go in 2026](https://bitfieldconsulting.com/posts/rust-vs-go)

### Go CLI Frameworks
- [Cobra GitHub Repository](https://github.com/spf13/cobra)
- [Building CLI Apps in Go with Cobra & Viper](https://www.glukhov.org/post/2025/11/go-cli-applications-with-cobra-and-viper/)
- [Why Every Go Developer Needs Cobra](https://medium.com/@monikasinghal713/why-every-go-developer-needs-to-know-cobra-for-cli-development-a66b81711ce2)

### Bash Best Practices
- [Bash Scripting Best Practices 2025](https://medium.com/@prasanna.a1.usage/best-practices-we-need-to-follow-in-bash-scripting-in-2025-cebcdf254768)
- [Tips for Better Bash Scripts](https://medium.com/@rafal.kedziorski/tips-for-better-bash-scripts-36a9ce88dfa8)
- [Distributing Bash Scripts with Dependencies](https://www.linuxbash.sh/post/installing-software-and-managing-dependencies-in-scripts)
- [Microsoft Bash Code Review Guidelines](https://microsoft.github.io/code-with-engineering-playbook/code-reviews/recipes/bash/)

### Testing & Quality Tools
- [BATS Core GitHub Repository](https://github.com/bats-core/bats-core)
- [Testing Bash Scripts with BATS](https://www.hackerone.com/blog/testing-bash-scripts-bats-practical-guide)
- [Effective End-to-End Testing with BATS](https://blog.cubieserver.de/2025/effective-end-to-end-testing-with-bats/)
- [ShellCheck GitHub Repository](https://github.com/koalaman/shellcheck)
- [Enhancing Shell Script Quality](https://medium.com/continuous-insights/enhancing-shell-script-quality-with-sanity-check-s-29834d38a99f)

### Installation & Distribution
- [Curl to Bash Security Considerations](https://medium.com/@esotericmeans/the-truth-about-curl-and-installing-software-securely-on-linux-63cd12e7befd)
- [Best Practices When Using Curl in Shell Scripts](https://www.joyfulbikeshedding.com/blog/2020-05-11-best-practices-when-using-curl-in-shell-scripts.html)
- [Makefile Best Practices 2025](https://paiml.com/blog/2025-01-25-makefiles-modern-development/)
- [Why Makefiles Still Matter](https://docs.cloudposse.com/best-practices/developer/makefile/)

### Podman & Container Orchestration
- [Create Containers with Podman and Shell Scripts](https://www.redhat.com/en/blog/create-containers-podman-quickly)
- [Containers in 2025: Docker vs Podman](https://www.linuxjournal.com/content/containers-2025-docker-vs-podman-modern-developers)
- [Podman vs Docker 2025 Comparison](https://uptrace.dev/comparisons/podman-vs-docker)
