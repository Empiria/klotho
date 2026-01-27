# Phase 6: Rename to Klotho - Research

**Researched:** 2026-01-27
**Domain:** Project and CLI tool renaming
**Confidence:** HIGH

## Summary

This research covers the technical domain of renaming a CLI tool project from "agent-session" to "klotho". The project is a bash-based CLI that manages containerized AI agent sessions using Podman. Renaming involves multiple layers: the command name users type, container image tags, documentation, repository name, and XDG config paths.

The standard approach combines systematic search-and-replace with backward compatibility strategies. Key challenges include maintaining existing sessions during transition, handling hardcoded references across multiple file types (bash, markdown, config), and ensuring container images rebuild correctly with new naming.

**Primary recommendation:** Use a phased approach with symlinks for backward compatibility, systematic grep-based search for all references, and careful attention to image naming patterns that include the project name as a prefix.

## Standard Stack

Renaming a bash CLI project requires primarily built-in Unix tools and careful validation.

### Core Tools

| Tool | Version | Purpose | Why Standard |
|------|---------|---------|--------------|
| grep/ripgrep | system | Find all references to old name | Universal text search, regex support |
| git mv | system | Rename files while preserving history | Maintains git history properly |
| ln -s | system | Create backward compatibility symlink | Standard Unix approach for transitions |
| sed | system | Automated text replacement | Stream editor for bulk changes |

### Supporting Tools

| Tool | Version | Purpose | When to Use |
|------|---------|---------|-------------|
| shellcheck | latest | Validate bash scripts after rename | Catch syntax errors from replacements |
| podman | 4.0+ | Rebuild and retag container images | Container image management |
| git remote | system | Update repository URL if renamed | After GitHub repository rename |

**Installation:**
No additional installation needed - uses system tools already present.

## Architecture Patterns

### Recommended Rename Sequence

The correct order prevents breaking changes and maintains backward compatibility:

```
1. Code references (bash, config)
2. Container image names (build system)
3. Documentation (README, help text, examples)
4. Repository name (GitHub)
5. XDG config paths (maintain compatibility)
```

### Pattern 1: Systematic Search and Replace

**What:** Use grep to find all references before making changes
**When to use:** Before any rename operation
**Example:**
```bash
# Find all references to old name
grep -r "agent-session" . --exclude-dir=.git --exclude-dir=.planning

# Count references by file type
grep -r "agent-session" . --exclude-dir=.git | cut -d: -f1 | sort | uniq -c

# Find case variations
grep -ri "agent.session\|agent_session\|agentsession" . --exclude-dir=.git
```

### Pattern 2: Backward Compatibility Symlink

**What:** Create symlink from old name to new name for transition period
**When to use:** When users may have muscle memory or scripts using old name
**Example:**
```bash
# After renaming main script file
ln -s klotho agent-session

# Verify symlink works
./agent-session --help  # Should work via symlink
```

### Pattern 3: Image Name Pattern Replacement

**What:** Container images use project name as prefix (project-agent:tag)
**When to use:** When renaming projects that build multiple related images
**Example:**
```bash
# Old pattern: agent-session-claude:latest, agent-session-opencode:latest
# New pattern: klotho-claude:latest, klotho-opencode:latest

# In build script
podman build -t "klotho-${AGENT_NAME}:latest" .

# Check for image existence
podman image exists "klotho-${agent}:latest"
```

### Pattern 4: XDG Config Path Compatibility

**What:** Support both old and new config paths during transition
**When to use:** User configuration directories that follow XDG conventions
**Example:**
```bash
# Check both old and new locations
old_config="$HOME/.config/agent-session/agents/$agent/config.conf"
new_config="$HOME/.config/klotho/agents/$agent/config.conf"

# Prefer new, fall back to old
if [[ -f "$new_config" ]]; then
    source "$new_config"
elif [[ -f "$old_config" ]]; then
    source "$old_config"
fi
```

### Anti-Patterns to Avoid

- **Manual find-and-replace without verification:** Easy to miss references in help text, examples, comments
- **Renaming without backward compatibility:** Breaks existing user scripts and muscle memory
- **Inconsistent naming across layers:** Container images using old name while CLI uses new name
- **Forgetting case variations:** Missing AGENT_SESSION, agent_session, AgentSession patterns
- **Breaking git history:** Using delete+create instead of git mv for file renames

## Don't Hand-Roll

Problems that have existing solutions for project renaming:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Finding all text references | Custom script | `grep -r "pattern" . --exclude-dir=.git` | Handles binary files, respects gitignore patterns |
| Renaming files | rm + touch | `git mv oldname newname` | Preserves git history and handles staging |
| Bulk text replacement | Manual editing | `sed -i 's/old/new/g' file` or ripgrep --replace | Less error-prone, repeatable, auditable |
| Container rename detection | Custom tracking | `podman ps -a --format "{{.Names}}" \| grep pattern` | Already handles all containers, filtering |
| GitHub repo redirects | Manual URL updates | GitHub's automatic redirects | Built-in after rename, updates automatically |

**Key insight:** Text search and git operations are well-solved problems. Focus effort on planning what to rename and in what order, not building custom tooling.

## Common Pitfalls

### Pitfall 1: Incomplete Reference Search

**What goes wrong:** Missing references in non-obvious locations (help text, examples in docs, error messages)
**Why it happens:** Searching only code files, not documentation or embedded strings
**How to avoid:**
- Search all file types: `grep -r "agent-session" . --exclude-dir=.git --exclude-dir=.planning`
- Check case variations: `grep -ri "agent.session\|agent_session" .`
- Review help text functions and error messages separately
- Test all commands after rename to catch help text issues

**Warning signs:**
- User-facing text still shows old name
- Examples in help output don't work when copy-pasted
- Error messages reference wrong command name

### Pitfall 2: Container Image Orphaning

**What goes wrong:** New code looks for klotho-claude:latest but only agent-session-claude:latest exists
**Why it happens:** Renaming code references without rebuilding images or updating image tags
**How to avoid:**
- Update image name in build scripts FIRST
- Rebuild all images immediately: `./scripts/build.sh claude && ./scripts/build.sh opencode`
- Verify new images exist: `podman images | grep klotho`
- Consider keeping old images temporarily for rollback

**Warning signs:**
- "Image not found" errors when starting sessions
- Build script succeeds but creates wrong image name
- Old image tags still present in `podman images`

### Pitfall 3: Running Container Name Conflicts

**What goes wrong:** Existing sessions have containers named "claude-default" but code now expects "klotho-claude-default"
**Why it happens:** Container naming pattern changed but running containers use old pattern
**How to avoid:**
- Document that existing sessions must be stopped before upgrade
- Add detection for old container name pattern in find_container()
- Provide migration helper or clear error message
- Success criteria explicitly states "existing sessions continue to work"

**Warning signs:**
- `agent-session ls` shows no sessions but `podman ps` shows running containers
- Cannot attach to existing sessions after upgrade
- "Session not found" errors for known-running sessions

### Pitfall 4: XDG Config Path Breaking Changes

**What goes wrong:** User configs in ~/.config/agent-session/ are ignored after rename
**Why it happens:** Code only checks new path ~/.config/klotho/
**How to avoid:**
- Support BOTH old and new config paths during transition
- Check new path first, fall back to old path
- Log deprecation warning when using old path
- Document migration path in release notes

**Warning signs:**
- User-customized agent configs suddenly ignored
- Behavior reverts to defaults unexpectedly
- Support requests about "configs not working"

### Pitfall 5: GitHub Repository Redirect Assumptions

**What goes wrong:** Assuming redirects are permanent and work everywhere
**Why it happens:** GitHub redirects work for git operations but not all contexts
**How to avoid:**
- Update all local clones: `git remote set-url origin <new-url>`
- Don't reuse old repository name for different project
- Document URL change in release notes
- Update any CI/CD configs that reference old URL

**Warning signs:**
- GitHub warns "This is a redirect" when accessing repo
- Webhook deliveries fail with 404
- Submodules or dependencies break

### Pitfall 6: Incomplete Documentation Updates

**What goes wrong:** README has new name but examples still show old command
**Why it happens:** Bulk find-replace misses contextual examples or code blocks
**How to avoid:**
- Review every documentation file manually after bulk replace
- Test all documented examples by copy-paste execution
- Check both inline examples and separate example files
- Verify quick start guide end-to-end

**Warning signs:**
- Copy-pasting documented commands fails
- Users report "command not found" following quick start
- Mix of old and new names in same document

## Code Examples

### Find All References Pattern

```bash
# Source: Standard Unix text processing
# Find all references to project name
grep -r "agent-session" . \
  --exclude-dir=.git \
  --exclude-dir=.planning \
  --color=always

# Find with context to understand usage
grep -rn -C 2 "agent-session" agent-session scripts/build.sh

# Find in specific file types only
grep -r "agent-session" . --include="*.sh" --include="*.md"
```

### Safe Bulk Replacement Pattern

```bash
# Source: Standard sed usage pattern
# Always create backup before bulk replace
cp agent-session agent-session.backup

# Replace in specific file (creates .bak)
sed -i.bak 's/agent-session/klotho/g' agent-session

# Verify changes with diff
diff agent-session.backup agent-session

# If good, remove backup; if bad, restore
rm agent-session.bak  # OR: mv agent-session.backup agent-session
```

### Container Name Pattern Migration

```bash
# Source: Project's existing container finding logic
# Support both old and new container name patterns
find_container() {
    local session_name="$1"

    # Try new pattern first: klotho-<agent>-<session>
    local container
    container=$(podman ps -a --format "{{.Names}}" | grep -E "^klotho-.*-${session_name}$" | head -1)

    # Fall back to old pattern: <agent>-<session>
    if [[ -z "$container" ]]; then
        container=$(podman ps -a --format "{{.Names}}" | grep -E "^(claude|opencode)-${session_name}$" | head -1)
    fi

    if [[ -z "$container" ]]; then
        echo "error: session '$session_name' not found" >&2
        return 1
    fi

    echo "$container"
}
```

### Image Building with New Names

```bash
# Source: Project's scripts/build.sh pattern
# Build script must use new project name prefix
IMAGE_NAME="klotho-${AGENT_NAME}:latest"

podman build \
    --target="$AGENT_NAME" \
    --build-arg AGENT_NAME="$AGENT_NAME" \
    -t "$IMAGE_NAME" \
    .

echo "Build complete: $IMAGE_NAME"
```

### XDG Config Compatibility Layer

```bash
# Source: XDG Base Directory Specification compatibility pattern
load_agent_config() {
    local agent="$1"
    local config_home="${XDG_CONFIG_HOME:-$HOME/.config}"

    # Check new location first
    local new_config="$config_home/klotho/agents/$agent/config.conf"
    local old_config="$config_home/agent-session/agents/$agent/config.conf"
    local repo_config="./config/agents/$agent/config.conf"

    # Load repo default first (must exist)
    if [[ ! -f "$repo_config" ]]; then
        echo "error: no default config found for agent: $agent" >&2
        return 1
    fi
    source "$repo_config"

    # Override with user config (prefer new path)
    if [[ -f "$new_config" ]]; then
        source "$new_config"
    elif [[ -f "$old_config" ]]; then
        echo "warning: using deprecated config path: $old_config" >&2
        echo "please move config to: $new_config" >&2
        source "$old_config"
    fi
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Manual find-replace in editor | Grep-based systematic search + sed | Ancient practice | Reduces missed references |
| Delete and recreate files | git mv for renames | Git 1.5+ (2007) | Preserves history |
| Break backward compatibility | Symlinks/aliases during transition | Standard Unix practice | Smoother user experience |
| Single-pass rename | Phased approach with testing | Software engineering best practice | Catches issues early |
| Assume GitHub redirects permanent | Update all references explicitly | GitHub docs clarification | Prevents future breakage |

**Current best practices (2026):**
- Use ripgrep (rg) instead of grep for faster searching in large repos
- Leverage container tag aliases for zero-downtime transitions
- Automated testing of all rename scenarios before merge
- Comprehensive pre-commit hooks to prevent old name reintroduction

**Deprecated/outdated:**
- Manual search-replace without grep verification (too error-prone)
- Assuming old name disappears immediately (breaks users)
- Renaming without image rebuild plan (causes runtime failures)

## Open Questions

### Question 1: Container Naming Pattern

**What we know:** Current containers named `<agent>-<session>` (e.g., "claude-default")
**What's unclear:** Should new pattern be `klotho-<agent>-<session>` or just `<agent>-<session>`?
**Recommendation:** Use `klotho-<agent>-<session>` for consistency with image naming (`klotho-claude:latest`). Makes ownership clear when running multiple container projects. Update find_container() to support both patterns during transition.

### Question 2: Migration Strategy for Existing Sessions

**What we know:** Success criteria states "existing sessions continue to work during transition"
**What's unclear:** Define "during transition" - is this upgrade session or permanent?
**Recommendation:** Support old container name pattern permanently in find_container() for backward compatibility. Document that new sessions use new naming but old sessions remain accessible. No forced migration required.

### Question 3: Documentation Commit Timing

**What we know:** Phase 5 (Documentation) creates README before Phase 6 (Rename)
**What's unclear:** Should README in Phase 5 use agent-session or klotho?
**Recommendation:** Phase 5 should use "agent-session" as that's the current name. Phase 6 includes updating all documentation created in Phase 5. This matches real-world: document current state, then rename everything together.

## Sources

### Primary (HIGH confidence)

- [Command Line Interface Guidelines](https://clig.dev/) - CLI naming best practices
- [GitHub Docs: Renaming a repository](https://docs.github.com/en/repositories/creating-and-managing-repositories/renaming-a-repository) - Repository rename behavior
- [Podman rename documentation](https://docs.podman.io/en/stable/markdown/podman-rename.1.html) - Container rename capabilities
- Project source code analysis - agent-session script, Containerfile, build.sh, config patterns

### Secondary (MEDIUM confidence)

- [The Poetics of CLI Command Names](https://smallstep.com/blog/the-poetics-of-cli-command-names/) - Naming conventions and ergonomics
- [Technical Writing: Consistency rules and tools](https://medium.com/softserve-technical-communication/how-to-write-documentation-consistency-rules-and-tools-a7bf5162c9db) - Documentation consistency checks
- [Make SQL Server database changes backward compatible](https://www.mssqltips.com/sqlservertip/2071/make-your-sql-server-database-changes-backward-compatible-when-renaming-an-entity/) - Backward compatibility patterns
- [Rename refactoring pitfalls](https://dev.to/jamesdev4123/when-a-refactor-tool-renames-only-half-the-project-inconsistent-variable-names-across-files-501i) - Common mistakes in refactoring

### Tertiary (LOW confidence)

- WebSearch results for "CLI tool project renaming" - General best practices compilation
- WebSearch results for "symlink backward compatibility" - Unix transition patterns

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - Built-in Unix tools, verified usage patterns
- Architecture: HIGH - Patterns derived from project structure analysis and industry practices
- Pitfalls: HIGH - Based on documented issues in similar projects and anti-patterns
- Container specifics: HIGH - Verified against Podman documentation and project code

**Research date:** 2026-01-27
**Valid until:** 2026-04-27 (90 days - stable domain, tooling rarely changes)
