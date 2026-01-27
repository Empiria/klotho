# Phase 5: Documentation - Research

**Researched:** 2026-01-27
**Domain:** CLI tool documentation and technical writing
**Confidence:** HIGH

## Summary

This research investigated best practices for CLI tool documentation in 2026, focusing on structure, content patterns, and common pitfalls. The modern standard for developer CLI tools emphasizes example-driven design, progressive disclosure through collapsible sections, and troubleshooting as a first-class concern. Industry-leading tools like GitHub CLI demonstrate that README documentation should provide quick orientation while directing to detailed references, with installation instructions tailored per platform and command examples that are immediately copy-pasteable.

Key findings reveal that the biggest documentation failures come from treating docs as afterthoughts, failing to update them with tool evolution, and assuming users understand implicit conventions. Successful CLI documentation follows a strict section order (overview -> prerequisites -> quick start -> command reference -> troubleshooting), uses markdown collapsible sections for comprehensive but scannable content, and includes verification commands with expected output so users can self-diagnose issues.

The established pattern for command-line tools is single-file README documentation for tools of this complexity, with troubleshooting given dedicated non-collapsed space because users in error states need immediate access to solutions. Copy-paste examples must avoid special characters like brackets, pipes, and ellipses that break execution, and all placeholders must be clearly explained in accompanying text.

**Primary recommendation:** Follow the established pattern: single README.md with overview, prerequisites with verification, quick start (5-minute path to first session), command reference in collapsible details blocks, and prominent troubleshooting section at end.

## Standard Stack

The established tools for this documentation domain:

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Markdown | GitHub-flavored | Documentation format | Universal support, readable in plain text, rich formatting in GitHub |
| HTML details/summary | HTML5 | Progressive disclosure | Native browser support, works in GitHub, no JavaScript required |
| Code fencing | Markdown ``` | Command examples | Syntax highlighting, copy buttons in GitHub UI |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| Verification commands | Bash | Self-diagnostic | Every prerequisite section to enable user troubleshooting |
| Expected output | Plain text | User validation | After key verification commands and critical examples |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Single README | docs/ folder with multiple files | Multiple files appropriate for complex tools with extensive API docs; single README better for CLI with <10 commands |
| Collapsible sections | Flat structure | Flat structure more scannable but requires scrolling; collapsible better for keeping quick start visible |
| Markdown | Dedicated docs site (Read the Docs, Docusaurus) | Docs site overkill for single CLI tool; appropriate when tool has plugin ecosystem or extensive API |

**Installation:**
No installation needed - standard Markdown with GitHub-flavored extensions.

## Architecture Patterns

### Recommended README Structure
```markdown
# Project Title

Overview paragraph: what it does, why it exists, who it's for

## Prerequisites

Platform requirements + installation links + verification commands

## Quick Start

5-minute path from zero to first successful command with copy-paste examples

## Commands

<details> blocks for each command with syntax, options, examples

## Concepts

Explain unfamiliar technologies (podman vs docker, zellij vs tmux)

## Troubleshooting

Common errors with symptoms, causes, and solutions (NOT collapsed)
```

### Pattern 1: Prerequisites with Verification
**What:** Each prerequisite includes version requirement, installation instructions per platform, and verification command showing expected output.

**When to use:** Every prerequisite section - this is non-negotiable for developer tools.

**Example:**
```markdown
### Podman

Container runtime for rootless containers.

**Required version:** 4.0+

**Verify:**
\`\`\`bash
podman --version
# Expected: podman version 4.x.x or higher
\`\`\`

**Install:**

**Linux (Debian/Ubuntu):**
\`\`\`bash
sudo apt install podman
\`\`\`

See [official Podman installation guide](https://podman.io/getting-started/installation) for other distributions.
```

**Source:** Combined pattern from [Command Line Interface Guidelines](https://clig.dev/) and [Google Developer Documentation Style Guide](https://developers.google.com/style/code-syntax)

### Pattern 2: Collapsible Command Reference
**What:** Use HTML `<details>` and `<summary>` tags to create collapsible sections for detailed command documentation, keeping the quick start visible when users first open the README.

**When to use:** For command reference sections where comprehensive documentation would create excessive scrolling. Do NOT use for troubleshooting - errors need immediate visibility.

**Example:**
```markdown
## Commands

### start

<details>
<summary>Create a new session or attach to existing one</summary>

**Syntax:**
\`\`\`
agent-session start [-a AGENT] [-n NAME] [project-paths...]
\`\`\`

**Options:**
- `-a, --agent AGENT` - Agent to use (default: "claude")
- `-n, --name NAME` - Session name (default: "default")

**Examples:**
\`\`\`bash
agent-session start ~/projects/webapp
agent-session start -n frontend ~/webapp
\`\`\`

</details>
```

**Source:** [GitHub Documentation - Organizing information with collapsed sections](https://docs.github.com/en/get-started/writing-on-github/working-with-advanced-formatting/organizing-information-with-collapsed-sections)

**Critical spacing requirement:** Must include blank line after `<summary>` tag before markdown content, and blank line after each `</details>` tag.

### Pattern 3: Copy-Paste Command Blocks
**What:** Command examples use clean code blocks without inline comments, special characters (brackets, pipes, ellipses), or optional parameters that could break execution if copied directly.

**When to use:** Every command example in quick start and command reference.

**Example:**
```markdown
**Good (copy-paste safe):**
\`\`\`bash
agent-session start -n frontend ~/webapp
\`\`\`

**Bad (breaks on copy):**
\`\`\`bash
agent-session start [-n NAME] [project-paths...]  # Create session
\`\`\`
```

**Source:** [Google Developer Documentation Style Guide - Document command-line syntax](https://developers.google.com/style/code-syntax)

**Why:** Brackets, pipes, braces, and ellipses are special shell characters. When users copy-paste examples containing these, commands fail cryptically. Document variations with separate examples or prose explanation.

### Pattern 4: Troubleshooting Entry Structure
**What:** Each troubleshooting entry follows the pattern: error symptom (what user sees) -> cause (why it happens) -> solution (specific commands to fix) -> verification (how to confirm fix).

**When to use:** Every troubleshooting entry.

**Example:**
```markdown
### "Cannot connect to Podman" (macOS)

**Symptom:** Error message "Error: unable to connect to Podman socket"

**Cause:** The podman machine is not running. On macOS, Podman runs inside a lightweight Linux VM that must be started before use.

**Solution:**
\`\`\`bash
podman machine start
\`\`\`

**Verify:**
\`\`\`bash
podman machine list
# Should show a running machine
\`\`\`
```

**Source:** [Archbee - What Is a Troubleshooting Guide](https://www.archbee.com/blog/troubleshooting-guide) and [Infrasity - CLI Documentation Checklist](https://www.infrasity.com/blog/cli-docs-checklist)

### Pattern 5: Platform-Specific Installation
**What:** Installation instructions organized by platform (Linux distros, macOS, Windows if supported) with clear headings and distro-specific package manager commands.

**When to use:** Prerequisites section for any tool requiring platform-specific installation.

**Example:**
```markdown
**Install:**

**Linux:**

Debian/Ubuntu:
\`\`\`bash
sudo apt install podman
\`\`\`

Fedora:
\`\`\`bash
sudo dnf install podman
\`\`\`

**macOS:**
\`\`\`bash
brew install podman
\`\`\`
```

**Source:** Pattern observed in [GitHub CLI README](https://github.com/cli/cli) and documented in [CLI Documentation Checklist](https://www.infrasity.com/blog/cli-docs-checklist)

### Anti-Patterns to Avoid
- **Inline comments in commands:** Clutters copy-paste blocks, can break shell parsing
- **Optional parameters in examples:** Use `[OPTIONAL]` notation only in syntax reference, never in runnable examples
- **Collapsed troubleshooting:** Users in error states need immediate access, not extra clicks
- **Missing expected output:** Without output examples, users can't tell if verification succeeded
- **Outdated examples:** Examples that don't match current command structure destroy trust immediately

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| API documentation | Custom HTML generator | Maintain single Markdown README | Tool has <10 commands; dedicated API docs overkill; Markdown sufficient |
| Version-specific docs | Multiple README versions | Single README with version notes inline | Tool versioning simple; separate docs needed only for major breaking changes |
| Interactive tutorials | Custom onboarding system | Well-structured quick start section | 5-minute goal achievable with clear quick start; interactivity adds complexity without value |
| Command completion docs | Separate shell completion guide | Single troubleshooting entry if completion exists | Shell completion is prerequisite verification, not feature documentation |

**Key insight:** CLI documentation suffers from over-engineering more often than under-engineering. The pattern that works for GitHub CLI (50+ commands) is overkill for 5-command tools. Markdown README with smart use of collapsible sections hits the sweet spot for tools with <10 commands and clear workflows.

## Common Pitfalls

### Pitfall 1: Documentation as Afterthought
**What goes wrong:** Writing documentation after tool is "done" results in missing examples, outdated commands, and descriptions that don't match actual behavior. Documentation gets updated slowly or not at all as tool evolves.

**Why it happens:** Documentation seen as separate from development rather than integral to user experience. No process to update docs alongside code changes.

**How to avoid:** Treat documentation as first-class deliverable. When commands change, update docs in same commit. Test examples by actually running them, not assuming they work.

**Warning signs:**
- Examples that reference flags or options that no longer exist
- Help text in CLI doesn't match README
- No one has actually followed the quick start on a fresh machine

**Source:** [Infrasity - CLI Documentation Checklist](https://www.infrasity.com/blog/cli-docs-checklist)

### Pitfall 2: Assuming Implicit Knowledge
**What goes wrong:** Documentation uses terms like "the podman machine" or "your Zellij session" without explaining what these are. Users familiar with Docker but not Podman or tmux but not Zellij get confused and frustrated.

**Why it happens:** Author curse of knowledge - developers writing docs know the domain deeply and forget users might not. Especially common when tool combines multiple technologies.

**How to avoid:** Include dedicated "Concepts" section explaining key technologies. When first introducing terms, provide brief inline explanation or link to concepts section.

**Warning signs:**
- User questions like "What's a podman machine?" or "I thought this used Docker?"
- Prerequisites section assumes users know what Podman/Zellij/etc. are
- No links to external documentation for unfamiliar technologies

**Source:** [Command Line Interface Guidelines](https://clig.dev/) - "Don't assume what the user meant" principle

### Pitfall 3: Non-Copy-Pasteable Examples
**What goes wrong:** Command examples include brackets `[OPTIONAL]`, pipes `{A|B}`, ellipses `...`, or inline comments that break commands when copied directly. Users get cryptic errors and blame the tool.

**Why it happens:** Trying to show command syntax and examples in single block. Notation that makes sense in documentation (brackets for optional) is shell-meaningful.

**How to avoid:** Syntax reference uses notation (brackets, pipes). Examples use actual runnable commands with real values or generic placeholders like `~/webapp`. Explain variations in prose, show variations in separate example blocks.

**Warning signs:**
- Commands with brackets, pipes, braces, or ellipses in quick start
- Inline comments in command blocks (# Create new session)
- Single example trying to show multiple options simultaneously

**Source:** [Google Developer Documentation Style Guide - Command-line syntax](https://developers.google.com/style/code-syntax)

### Pitfall 4: Missing Verification Commands
**What goes wrong:** Prerequisites list required software without showing how to verify installation or expected output. Users who installed incorrectly don't discover issues until later failure, making debugging harder.

**Why it happens:** Assumption that "install X" is sufficient instruction. Missing feedback loop for users to self-diagnose.

**How to avoid:** Every prerequisite includes three components: version requirement, installation instructions, and verification command with expected output. Format expected output as comment or prose: "# Expected: podman version 4.x.x"

**Warning signs:**
- Prerequisites section ends at installation, no verification step
- Verification commands without expected output
- Troubleshooting gets questions that verification would have caught

**Source:** [Command Line Interface Guidelines](https://clig.dev/) and [CLI Documentation Checklist](https://www.infrasity.com/blog/cli-docs-checklist)

### Pitfall 5: Troubleshooting Buried or Collapsed
**What goes wrong:** Troubleshooting hidden in collapsed section or placed after extensive documentation makes it hard to find when users encounter errors. Users in error state need solutions immediately, not extra navigation.

**Why it happens:** Misguided attempt to keep README "clean" or assumption that troubleshooting is rare. Designer optimization (visual cleanliness) conflicting with user optimization (error recovery).

**How to avoid:** Troubleshooting gets dedicated section at end of README, NOT collapsed. Use descriptive error headings so users can scan for their specific error. Link to troubleshooting from places where errors commonly occur.

**Warning signs:**
- Troubleshooting in `<details>` block requiring click to expand
- Troubleshooting scattered throughout doc rather than centralized
- Generic "if you have problems" text instead of specific error patterns

**Source:** [Archbee - Troubleshooting Guide Structure](https://www.archbee.com/blog/troubleshooting-guide) and [CLI Best Practices](https://hackmd.io/@arturtamborski/cli-best-practices)

### Pitfall 6: Cross-Platform Inconsistencies Ignored
**What goes wrong:** Documentation shows only Linux commands, making macOS users figure out differences themselves. Platform-specific quirks (like "podman machine" on macOS) discovered through trial and error rather than documented upfront.

**Why it happens:** Developer writes docs from their own platform, doesn't test on others. Platform differences seem "obvious" to author.

**How to avoid:** Explicitly call out platform-specific behavior with headers ("macOS-specific notes:"). Test documentation on all supported platforms. Include platform indicators in commands when behavior differs.

**Warning signs:**
- Installation instructions for only one OS
- No mention of platform-specific quirks or requirements
- Users reporting platform-specific failures that aren't in troubleshooting

**Source:** [Infrasity - CLI Documentation Checklist](https://www.infrasity.com/blog/cli-docs-checklist)

## Code Examples

Verified patterns from official sources:

### Example 1: Prerequisite with Verification
```markdown
### Bash

Shell interpreter required to run the agent-session script.

**Required version:** 4.0+

**Verify:**
\`\`\`bash
bash --version
# Expected: GNU bash, version 4.x.x or higher
\`\`\`

**Install:**

**Linux:**
Pre-installed on most distributions.

**macOS:**
\`\`\`bash
brew install bash
\`\`\`
```

**Source:** Pattern from [Command Line Interface Guidelines](https://clig.dev/) - "Display comprehensive help when users pass -h or --help"

### Example 2: Quick Start Copy-Paste Flow
```markdown
## Quick Start

Start your first agent session in under 5 minutes:

1. Verify prerequisites:
\`\`\`bash
podman --version
bash --version
\`\`\`

2. Create session with current directory:
\`\`\`bash
agent-session start
\`\`\`

3. Inside the container, Claude Code starts automatically. Try:
\`\`\`
/gsd:help
\`\`\`

4. Detach anytime with `Ctrl+C` - session keeps running.

5. Reattach later:
\`\`\`bash
agent-session start
\`\`\`
```

**Source:** Pattern from [Archbee - README Creating Tips](https://www.archbee.com/blog/readme-creating-tips) - "Lead with examples"

### Example 3: Collapsible Command Reference
```markdown
### stop

<details>
<summary>Stop a running session</summary>

**Syntax:**
\`\`\`
agent-session stop [SESSION_NAME]
\`\`\`

**Arguments:**
- `SESSION_NAME` - Name of session to stop (default: "default")

**Examples:**
\`\`\`bash
agent-session stop
agent-session stop frontend
\`\`\`

**Notes:**
- Stopping an already-stopped session succeeds silently
- Use `agent-session ls` to see session status

</details>
```

**Source:** [GitHub Documentation - Collapsed sections](https://docs.github.com/en/get-started/writing-on-github/working-with-advanced-formatting/organizing-information-with-collapsed-sections)

### Example 4: Troubleshooting Entry
```markdown
### "podman: command not found"

**Cause:** Podman is not installed or not in PATH.

**Solution:**

**Linux (Debian/Ubuntu):**
\`\`\`bash
sudo apt update
sudo apt install podman
\`\`\`

**macOS:**
\`\`\`bash
brew install podman
\`\`\`

**Verify installation:**
\`\`\`bash
podman --version
# Should show: podman version 4.x.x or higher
\`\`\`
```

**Source:** [Archbee - Troubleshooting Guide](https://www.archbee.com/blog/troubleshooting-guide)

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Separate man pages only | README + man pages + web docs | ~2015 | GitHub normalized README as primary docs; man pages still valued but supplementary |
| Plain help text dump | Contextual help with examples | ~2018 | Tools like `jq` showed value of example-first help; now standard |
| Separate docs/ folder | Single README for simple tools | ~2020 | Recognition that folder structure adds friction for <10 commands |
| No collapsible sections | Progressive disclosure with `<details>` | ~2021 | GitHub's markdown support made progressive disclosure viable |
| Generic "see website" links | Direct anchor links in help text | ~2022 | CLI Guidelines codified linking to specific doc sections from error messages |

**Deprecated/outdated:**
- **Man pages as primary documentation:** Still valuable, but README is now expected first stop. Man pages supplement, don't replace.
- **Wiki-based documentation:** Wikis hard to version-control and test. Markdown in repo ensures docs stay in sync with code.
- **Animated GIFs in README:** Common 2018-2020, now seen as noisy. Static screenshots with clear output preferred.

## Open Questions

### 1. Concepts Section Placement
**What we know:** User context identified need for explaining podman vs docker, zellij vs tmux for mixed-knowledge audience.

**What's unclear:** Optimal placement. Before quick start (foundational knowledge) or after (just-in-time learning)?

**Recommendation:** After quick start, before command reference. Users anxious to see tool work - quick start satisfies that urgency. After first success, they're receptive to deeper understanding. Reference: [Command Line Interface Guidelines](https://clig.dev/) principle of "easy to get started, easy to get help later."

### 2. OpenCode Agent Documentation Depth
**What we know:** Tool supports both Claude and OpenCode agents. User context specifies showing agent selection in quick start.

**What's unclear:** How much OpenCode-specific setup/config to document. Is agent selection transparent or does OpenCode need special instructions?

**Recommendation:** Test both agents end-to-end. If OpenCode "just works" with `--agent opencode`, mention in examples. If OpenCode requires additional setup (API keys, config files), add dedicated subsection in prerequisites. Only document what's actually different from Claude.

### 3. Environment Variables Documentation Completeness
**What we know:** Code shows `AGENT_SESSION_MOUNTS` and `AGENT_SESSION_KOB` for advanced use cases.

**What's unclear:** Whether these belong in quick start (adds cognitive load) or advanced usage section (might be needed sooner).

**Recommendation:** Keep out of quick start. Add "Advanced Usage" section after command reference, before troubleshooting. Include these env vars there with use cases. Quick start must hit 5-minute goal - environment variables are power features.

## Sources

### Primary (HIGH confidence)
- [Command Line Interface Guidelines](https://clig.dev/) - Comprehensive CLI design principles, 2023
- [Google Developer Documentation Style Guide - Command-line syntax](https://developers.google.com/style/code-syntax) - Official Google standards
- [GitHub Documentation - Organizing information with collapsed sections](https://docs.github.com/en/get-started/writing-on-github/working-with-advanced-formatting/organizing-information-with-collapsed-sections) - Official GitHub Markdown reference
- [GitHub CLI README](https://github.com/cli/cli/blob/trunk/README.md) - Industry example, multi-platform CLI with ~50 commands
- [Archbee - What Is a Troubleshooting Guide](https://www.archbee.com/blog/troubleshooting-guide) - Troubleshooting structure best practices

### Secondary (MEDIUM confidence)
- [Infrasity - CLI Documentation Checklist](https://www.infrasity.com/blog/cli-docs-checklist) - Comprehensive checklist, verified against official sources
- [DEV Community - 14 Tips for Amazing CLI Applications](https://dev.to/wesen/14-great-tips-to-make-amazing-cli-applications-3gp3) - Community-sourced patterns
- [MarkdownTools - Collapsible Sections Guide](https://blog.markdowntools.com/posts/markdown-collapsible-sections-guide) - Technical implementation details

### Secondary sources cross-verified with primary
- [README Best Practices - Tilburg Science Hub](https://tilburgsciencehub.com/building-blocks/store-and-document-your-data/document-data/readme-best-practices/) - Academic source, verified structure matches GitHub CLI pattern
- [FreeCodeCamp - How to Write a Good README](https://www.freecodecamp.org/news/how-to-write-a-good-readme-file/) - Educational source, principles match Command Line Interface Guidelines

### Tertiary (LOW confidence - flagged for validation)
- WebSearch results on "CLI documentation trends 2025" - General trends, not specific technical guidance
- WebSearch results on "modern CLI tools 2026" - Tool discovery, not documentation patterns

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - Markdown with GitHub-flavored extensions is universal, patterns verified in official docs
- Architecture: HIGH - Section structure and patterns verified across multiple authoritative sources (Google, GitHub, CLI Guidelines)
- Pitfalls: HIGH - Common mistakes documented in multiple sources with concrete examples and prevention strategies

**Research date:** 2026-01-27

**Valid until:** ~60 days (March 2026) - Documentation patterns change slowly; Markdown and GitHub features stable. Reassess if GitHub changes `<details>` rendering or if tool grows beyond simple CLI scope.
