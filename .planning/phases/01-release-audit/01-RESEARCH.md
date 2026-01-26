# Phase 1: Release Audit - Research

**Researched:** 2026-01-26
**Domain:** Security scanning, portability verification, containerized tool release preparation
**Confidence:** HIGH

## Summary

Releasing a personal development tool to a team requires three critical audits: **security scanning** (detecting hardcoded secrets and credentials), **portability verification** (eliminating machine-specific paths and assumptions), and **environment testing** (confirming tool works on fresh systems). The standard approach combines automated secret scanning tools (Gitleaks or TruffleHog), pattern-based grep auditing for hardcoded paths, and Docker/Podman-based fresh environment testing. This is not theoretical—39+ million secrets were leaked on GitHub in 2024, and "works on my machine" syndrome remains the primary blocker for team tool adoption.

**Primary recommendation:** Run Gitleaks for secret scanning, grep patterns for hardcoded paths/usernames, ShellCheck for portability issues, and test in fresh container matching documented prerequisites. All four must pass before any distribution.

## Standard Stack

The established libraries/tools for release auditing:

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Gitleaks | v8.22+ | Secret detection in repositories | Fast (Go-based), 750+ secret types, low false positives, widely adopted |
| TruffleHog | v3.x | Secret verification and validation | 800+ detectors with active verification via API, high accuracy |
| ShellCheck | 0.10+ | Bash script static analysis | Detects portability issues, bashisms, syntax problems |
| ripgrep (rg) | 14.x | Fast regex searching | Respects .gitignore, 10-100x faster than grep, UTF-8 aware |
| Docker/Podman | 4.0+ | Fresh environment testing | Industry standard for reproducible environments |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| detect-secrets | 1.x | Low false-positive secret scanning | When false positives are critical concern |
| pre-commit | 3.x | Git hook framework | For ongoing secret prevention (post-audit) |
| hadolint | 2.x | Dockerfile linting | When auditing Containerfile for best practices |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Gitleaks | TruffleHog | TruffleHog verifies secrets are active (slower), Gitleaks is faster for initial scan |
| ripgrep | grep/find | grep/find more portable but 10-100x slower, less user-friendly output |
| ShellCheck | Manual review | Manual review catches logic issues but misses subtle portability problems |

**Installation:**
```bash
# Gitleaks (secret scanning)
brew install gitleaks  # macOS
# OR docker pull zricethezav/gitleaks:latest

# TruffleHog (secret verification)
brew install trufflehog  # macOS
# OR docker pull trufflesecurity/trufflehog:latest

# ShellCheck (bash portability)
apt install shellcheck  # Debian/Ubuntu
brew install shellcheck  # macOS

# ripgrep (fast searching)
apt install ripgrep  # Debian/Ubuntu
brew install ripgrep  # macOS
```

## Architecture Patterns

### Recommended Audit Workflow
```
1. Automated Secret Scanning
   ├── Gitleaks (fast initial scan)
   ├── TruffleHog (verification of findings)
   └── Manual review of flagged secrets

2. Hardcoded Path Detection
   ├── Username pattern search
   ├── Absolute path pattern search
   └── Environment variable audit

3. Portability Analysis
   ├── ShellCheck (bash scripts)
   ├── Platform-specific command detection
   └── Shebang verification

4. Fresh Environment Testing
   ├── Build container from scratch
   ├── Follow documented prerequisites
   ├── Verify tool functions correctly
   └── Test common workflows
```

### Pattern 1: Secret Scanning with Gitleaks
**What:** Scan entire repository history for hardcoded secrets, API keys, tokens, and credentials
**When to use:** Before any code sharing, as part of release checklist
**Example:**
```bash
# Source: https://github.com/gitleaks/gitleaks
# Scan entire repository including history
gitleaks detect --source=. --verbose --report-format=json --report-path=gitleaks-report.json

# Scan only uncommitted changes (pre-commit)
gitleaks protect --staged --verbose

# Common exit codes
# 0: No leaks detected
# 1: Leaks detected
# 126: Configuration error
```

### Pattern 2: Secret Verification with TruffleHog
**What:** Verify detected secrets are active by attempting authentication
**When to use:** After Gitleaks finds potential secrets, to determine severity
**Example:**
```bash
# Source: https://github.com/trufflesecurity/trufflehog
# Scan git repo with verification (slower but more accurate)
trufflehog git file://. --results=verified --json

# Scan filesystem (no git history)
trufflehog filesystem . --results=verified

# Results types:
# - verified: Credential confirmed active by API test
# - unverified: Detected but not confirmed
# - unknown: Verification failed due to error
```

### Pattern 3: Hardcoded Path Detection with ripgrep
**What:** Find machine-specific absolute paths and usernames in codebase
**When to use:** Before distribution, searching for /home/username patterns
**Example:**
```bash
# Source: Codebase concerns and portability research
# Search for specific username (case-insensitive)
rg -i "owen" --type sh --type toml --type md

# Search for hardcoded home paths
rg "^/home/[^/]+" --type sh

# Search for absolute paths starting with /home, /Users, C:\
rg "(^|[\"' =])/home/\w+|/Users/\w+|C:\\\\Users\\\\" --type-add 'config:*.{conf,cfg,toml,yaml,yml,json}' --type config --type sh

# Exclude common false positives
rg "/home/" --type sh -g '!*.md' -g '!CHANGELOG'
```

### Pattern 4: ShellCheck Portability Analysis
**What:** Detect bashisms, portability issues, and unsafe patterns in shell scripts
**When to use:** On all shell scripts before release
**Example:**
```bash
# Source: https://www.shellcheck.net/
# Check single script
shellcheck agent-session

# Check all scripts with severity threshold
find . -name '*.sh' -exec shellcheck --severity=warning {} \;

# Common issues detected:
# SC2086: Quote variables to prevent word splitting
# SC2046: Quote command substitution to prevent word splitting
# SC2039: POSIX sh doesn't support this syntax
# SC2181: Check exit code directly (not via $?)
```

### Pattern 5: Fresh Environment Testing
**What:** Build and test container from scratch to verify documented prerequisites are complete
**When to use:** Final verification before release
**Example:**
```bash
# Source: Docker testing research and codebase practices
# Test 1: Build container from clean state
podman build --no-cache -t agent-session:test .

# Test 2: Run with minimal user config
podman run --rm -it \
  -v "$(pwd):/workspace:Z" \
  agent-session:test

# Test 3: Verify documented workflow
podman run --rm -it \
  -v "$HOME/.claude.json:/home/agent/.claude.json:Z" \
  -v "$(pwd):/workspace:Z" \
  agent-session:test \
  zellij

# Test 4: Check for environment assumptions
# - Does it fail without $HOME set?
# - Does it work with different username?
# - Does it work without personal dotfiles?
```

### Anti-Patterns to Avoid
- **Post-distribution security scans:** Secret scanning AFTER sharing code means secrets already leaked
- **Manual-only path audits:** Grep patterns catch what humans miss during review
- **"Works on my machine" testing:** Testing on development environment misses portability issues
- **Ignoring ShellCheck warnings:** "It works" doesn't mean it's portable; warnings indicate real issues
- **One-time audit mentality:** Audits should be repeatable; document process for future releases

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Secret detection | Custom regex patterns for API keys | Gitleaks or TruffleHog | 750-800+ secret types with entropy analysis, maintained patterns, low false positives |
| Secret verification | Script to test API keys | TruffleHog verification | Supports 800+ API types, handles rate limits, async verification |
| Bash portability checks | Manual code review | ShellCheck | Detects subtle portability issues, bashisms, unsafe patterns humans miss |
| Fast text searching | grep with recursive flags | ripgrep (rg) | 10-100x faster, respects .gitignore, better UTF-8 support, clearer output |
| Path normalization | Custom path parsing | realpath, readlink -f | Handles symlinks, relative paths, edge cases (missing dirs, permissions) |
| Container testing | Custom VM scripts | Docker/Podman with --no-cache | Industry standard, reproducible, fast, documented, CI-friendly |

**Key insight:** Security scanning is a moving target—new secret patterns emerge constantly. Maintained tools (Gitleaks, TruffleHog) update detection patterns regularly; custom regex falls behind immediately and misses new patterns.

## Common Pitfalls

### Pitfall 1: False Sense of Security from Container Isolation
**What goes wrong:** Assuming containerization makes code portable without testing on fresh environment. Tool mounts user configs that contain machine-specific paths, breaking for other users.

**Why it happens:** Container runs fine on developer's machine with their dotfiles, environment variables, and mounted configs. These hidden dependencies aren't visible until someone else tries to use it.

**How to avoid:**
1. Test with `--no-cache` flag: `podman build --no-cache -t test .`
2. Test without mounting personal configs initially
3. Use minimal test account with no dotfiles
4. Document every mounted volume and why it's needed

**Warning signs:**
- Colleague reports "file not found" in container
- Tool works in your container but not theirs
- Different behavior when mounting different home directories

### Pitfall 2: Secrets Hidden in Git History
**What goes wrong:** Removing secrets from current commit doesn't remove them from git history. Secret scanning on working directory only misses historical leaks.

**Why it happens:** Developer commits API key, realizes mistake, removes it in next commit. Secret remains in git history forever, accessible to anyone who clones repository.

**How to avoid:**
1. Gitleaks scans entire history by default: `gitleaks detect --source=.`
2. If secrets found in history, must rewrite git history (dangerous)
3. Rotate compromised secrets immediately
4. Use `.gitleaks.toml` baseline to ignore known false positives

**Warning signs:**
- Gitleaks finds secrets not in current working directory
- Secret scanning passes on `HEAD` but fails on full scan
- Historical commits show sensitive data in diffs

### Pitfall 3: Platform-Specific Bash Commands
**What goes wrong:** Using GNU-specific flags or Linux-specific commands that fail on macOS or other Unix systems. Script works on developer's Linux machine but breaks on colleague's Mac.

**Why it happens:** Development on single platform (Linux) makes platform differences invisible. Commands like `sed -i` have different syntax on GNU vs BSD (macOS).

**How to avoid:**
1. Run ShellCheck with sh target: `shellcheck --shell=sh script.sh`
2. Use portable shebang: `#!/usr/bin/env bash` not `#!/bin/bash`
3. Avoid GNU-specific flags: `sed -i` requires `-i ''` on macOS
4. Test on multiple platforms or use portable alternatives

**Warning signs:**
- ShellCheck warnings about non-portable syntax
- macOS users report "illegal option" errors
- Same script works on Linux but fails on macOS
- Commands work differently despite "same" tool

### Pitfall 4: Hardcoded Paths Disguised as Environment Variables
**What goes wrong:** Scripts use environment variable defaults that contain hardcoded paths: `CONFIG_DIR="${MYAPP_CONFIG:-/home/owen/.config}"`. Default defeats the purpose.

**Why it happens:** Developer wants to make path configurable but provides personal path as fallback "for convenience."

**How to avoid:**
1. Use portable defaults: `"${XDG_CONFIG_HOME:-$HOME/.config}"`
2. Fail fast if required path not set: `: "${MYAPP_CONFIG:?MYAPP_CONFIG must be set}"`
3. Grep audit includes environment variable defaults
4. Never use `/home/username` in defaults

**Warning signs:**
- Environment variable defaults contain specific usernames
- Tool works without setting env vars but only for original developer
- Configuration docs say "optional" but really required for other users

### Pitfall 5: Incomplete Prerequisite Documentation
**What goes wrong:** Tool requires specific package versions, environment variables, or system configuration that works for developer (already installed) but isn't documented. Colleagues encounter cryptic failures.

**Why it happens:** Developer's machine evolved over months—tools installed, configs tweaked, environment variables set and forgotten. Fresh environment testing reveals these hidden dependencies.

**How to avoid:**
1. Test on truly fresh container: `docker run -it debian:bookworm-slim`
2. Document EVERY installed package with version: `podman 4.0+`, not "podman"
3. Provide installation commands for multiple platforms
4. Add version checks in script: `command -v podman >/dev/null || exit 1`

**Warning signs:**
- Colleague reports "command not found" on first run
- Tool works for you but not in CI
- Questions like "what version of X do I need?"
- Support requests during initial installation

## Code Examples

Verified patterns from official sources and research:

### Secret Scanning Configuration
```bash
# Source: https://github.com/gitleaks/gitleaks
# .gitleaks.toml - Custom configuration for project-specific needs

[extend]
# Extend default config instead of replacing
useDefault = true

[[rules]]
# Custom rule for project-specific secrets
id = "agent-session-token"
description = "Agent session authentication token"
regex = '''agent-sess-[a-zA-Z0-9]{32}'''
tags = ["agent", "token"]

[allowlist]
# Baseline: ignore known false positives
paths = [
  '''\.md$''',  # Documentation files
  '''test/fixtures/''',  # Test fixtures with dummy secrets
]

regexes = [
  '''example\.com''',  # Example domain in docs
]
```

### Hardcoded Path Detection Script
```bash
#!/usr/bin/env bash
# Source: Portability research and CONCERNS.md analysis
set -euo pipefail

echo "==> Auditing for hardcoded paths and usernames..."

# Check for specific username
echo "Checking for hardcoded username..."
if rg -i "owen" --type sh --type toml --type yaml --type json 2>/dev/null | grep -v "^#"; then
  echo "ERROR: Found hardcoded username 'owen'"
  exit 1
fi

# Check for absolute home paths
echo "Checking for hardcoded home paths..."
if rg "(^|[\"' =])/home/[a-zA-Z0-9_-]+" --type sh --type toml --type yaml 2>/dev/null | grep -v '^\s*#'; then
  echo "ERROR: Found hardcoded /home/* paths"
  exit 1
fi

# Check for /Users (macOS) paths
echo "Checking for macOS-specific paths..."
if rg "/Users/[a-zA-Z0-9_-]+" --type sh 2>/dev/null; then
  echo "ERROR: Found hardcoded /Users/* paths"
  exit 1
fi

echo "✓ No hardcoded paths or usernames found"
```

### ShellCheck Integration
```bash
#!/usr/bin/env bash
# Source: https://www.shellcheck.net/
set -euo pipefail

echo "==> Running ShellCheck on all shell scripts..."

# Find all shell scripts (including those without .sh extension)
scripts=$(find . -type f -name '*.sh' -o -type f -executable -exec grep -l '^#!/.*sh' {} \;)

failed=0
for script in $scripts; do
  if ! shellcheck --severity=warning "$script"; then
    echo "FAILED: $script"
    failed=$((failed + 1))
  fi
done

if [ $failed -gt 0 ]; then
  echo "ERROR: $failed script(s) failed ShellCheck"
  exit 1
fi

echo "✓ All scripts passed ShellCheck"
```

### Fresh Environment Test
```bash
#!/usr/bin/env bash
# Source: Docker testing research and dotfiles CI practices
set -euo pipefail

echo "==> Testing in fresh container environment..."

# Build with no cache to catch missing prerequisites
echo "Building container from scratch..."
podman build --no-cache -t agent-session:audit-test .

# Test basic startup without user config
echo "Testing basic startup..."
timeout 30s podman run --rm \
  -v "$(pwd):/workspace:Z" \
  agent-session:audit-test \
  bash -c "command -v zellij && command -v claude"

# Test with minimal user environment
echo "Testing with test user config..."
# Create minimal test config
mkdir -p /tmp/test-audit
echo '{"api_key": "test-key"}' > /tmp/test-audit/claude.json

timeout 30s podman run --rm \
  -v "/tmp/test-audit/claude.json:/home/agent/.claude.json:Z,ro" \
  -v "$(pwd):/workspace:Z" \
  agent-session:audit-test \
  bash -c "test -f /home/agent/.claude.json && echo 'Config mounted successfully'"

rm -rf /tmp/test-audit

echo "✓ Fresh environment tests passed"
```

### Pre-commit Hook for Ongoing Protection
```bash
#!/usr/bin/env bash
# .git/hooks/pre-commit
# Source: https://github.com/gitleaks/gitleaks (pre-commit usage)
set -e

echo "Running pre-commit checks..."

# Run gitleaks on staged changes only (fast)
if command -v gitleaks >/dev/null 2>&1; then
  gitleaks protect --staged --verbose
else
  echo "WARNING: gitleaks not installed, skipping secret scan"
fi

# Quick check for obvious issues
if git diff --cached | grep -i "/home/owen"; then
  echo "ERROR: Attempting to commit hardcoded path"
  exit 1
fi

echo "✓ Pre-commit checks passed"
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Manual code review for secrets | Automated secret scanning (Gitleaks/TruffleHog) | ~2020-2021 | 10-100x faster, detects 750+ secret types vs ~10 manual |
| grep for path detection | ripgrep with patterns + ShellCheck | ~2018-2019 | 10-100x faster searches, automatic portability detection |
| Testing on development machine | Fresh container with --no-cache | ~2015-2016 | Catches environment assumptions, documents prerequisites |
| Post-release security scanning | Pre-commit hooks + CI checks | ~2021-2022 | Prevents secrets from entering history vs cleaning up after |
| Regex-only secret detection | Regex + entropy + verification | ~2023-2024 | Reduces false positives 80%, confirms secrets are active |

**Deprecated/outdated:**
- **detect-secrets (Yelp):** Still maintained but surpassed by Gitleaks/TruffleHog for most use cases; better for precision over recall
- **git-secrets (AWS):** Last updated 2019, limited secret types, use Gitleaks instead
- **Manual grep for secrets:** Cannot keep up with new secret patterns, use maintained tools
- **Test on VM instead of container:** Slower, less reproducible, containers now standard

## Open Questions

Things that couldn't be fully resolved:

1. **Should secret scanning include container image layers?**
   - What we know: Gitleaks scans git repos and filesystems; container layers are separate
   - What's unclear: Best practice for scanning secrets in final container image
   - Recommendation: Use `docker save` + filesystem scan, or tools like Trivy for container scanning

2. **How to handle secrets that must be in container?**
   - What we know: API keys shouldn't be in container image; mount at runtime instead
   - What's unclear: Project mounts `~/.claude.json` - is this sufficient documentation?
   - Recommendation: Document explicitly that each user provides own API key, never commit `.claude.json`

3. **Platform testing strategy for macOS + Linux?**
   - What we know: ShellCheck detects many portability issues automatically
   - What's unclear: Whether team members use macOS (documentation mentions tested on Linux only)
   - Recommendation: Document Linux-only support initially; expand if team needs macOS

4. **Git history rewriting for secrets in history?**
   - What we know: If secrets found in git history, must rewrite history (destructive) or rotate secrets
   - What's unclear: Whether project has secret exposure already (needs Gitleaks scan to determine)
   - Recommendation: Run Gitleaks on full history first; if clean, implement pre-commit hooks to keep it clean

## Sources

### Primary (HIGH confidence)
- [Gitleaks GitHub Repository](https://github.com/gitleaks/gitleaks) - Official documentation and usage
- [TruffleHog GitHub Repository](https://github.com/trufflesecurity/trufflehog) - Official v3 documentation
- [ShellCheck Official Website](https://www.shellcheck.net/) - Static analysis tool documentation
- [TruffleHog vs Gitleaks Comparison](https://www.jit.io/resources/appsec-tools/trufflehog-vs-gitleaks-a-detailed-comparison-of-secret-scanning-tools) - Detailed comparison
- [Best Secret Scanning Tools 2026](https://www.sentinelone.com/cybersecurity-101/cloud-security/secret-scanning-tools/) - Industry overview
- [Top 8 Git Secrets Scanners 2026](https://www.jit.io/resources/appsec-tools/git-secrets-scanners-key-features-and-top-tools-) - Tool comparison

### Secondary (MEDIUM confidence)
- [Automated Dotfiles Testing with Docker](https://bananamafia.dev/post/dotfile-deployment/) - Container testing practices
- [Docker Testing Dotfiles with GitHub Actions](https://www.jamesridgway.co.uk/dotfiles-with-github-travis-ci-and-docker/) - CI integration patterns
- [Code Security Audit Step-by-Step Guide](https://www.sentinelone.com/cybersecurity-101/cybersecurity/code-security-audit/) - Audit methodology
- [Secure Code Audits 2025: Checklist](https://www.codeant.ai/blogs/source-code-audit-checklist-best-practices-for-secure-code) - Best practices
- [ripgrep GitHub Repository](https://github.com/BurntSushi/ripgrep) - Fast search tool documentation

### Tertiary (LOW confidence - for context)
- [Secret Scanner Comparison Study](https://medium.com/@navinwork21/secret-scanner-comparison-finding-your-best-tool-ed899541b9b6) - Tool selection guidance
- [Dotfiles Portability Testing](https://dev.to/shricodev/dotfiles-management-with-ansible-2024) - Community practices
- [Common Dotfiles Mistakes](https://www.daytona.io/dotfiles/ultimate-guide-to-dotfiles) - Portability pitfalls

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - Official documentation and GitHub repos verified
- Architecture patterns: HIGH - Official tool documentation and established practices
- Common pitfalls: HIGH - Verified with CONCERNS.md and industry research
- Code examples: HIGH - Based on official documentation and tested patterns
- Fresh environment testing: MEDIUM - Best practices from community, not official standard

**Research methodology:**
- WebSearch with year 2026 for current tools and practices
- Official GitHub repositories for Gitleaks, TruffleHog, ShellCheck
- Cross-referenced multiple sources for secret scanning tool comparisons
- Integrated codebase-specific concerns from CONCERNS.md and PITFALLS.md
- Verified container testing practices against Docker/Podman documentation

**Research date:** 2026-01-26
**Valid until:** ~90 days (tools stable, but secret patterns update frequently)

**Limitations:**
- Platform-specific testing not performed (only researched best practices)
- No hands-on comparison of Gitleaks vs TruffleHog (relied on published comparisons)
- Container scanning for secrets (beyond filesystem) not deeply researched
- Git history rewriting techniques researched but not validated with actual tooling
