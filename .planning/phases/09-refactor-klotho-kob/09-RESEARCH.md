# Phase 9: Refactor KLOTHO_KOB - Research

**Researched:** 2026-01-27
**Domain:** Rust CLI argument parsing, container mount configuration, environment variable handling
**Confidence:** HIGH

## Summary

This phase refactors the `KLOTHO_KOB` environment variable feature which enables mounting external directories at the same host path inside containers so symlinks resolve correctly. The research focused on understanding the current implementation bug (wrong mount path), identifying the correct Rust patterns for implementing the replacement, and understanding container mount behavior with symlinks.

**Key findings:**
- Current bug confirmed: Rust implementation mounts at `/home/agent/.klotho` but bash script mounts at same path as host (e.g., `/home/user/projects/kob:/home/user/projects/kob:Z`)
- Clap 4.5 provides native support for repeatable flags via `Vec<String>` with derive macros
- Rust stdlib provides `env::split_paths()` for parsing colon-separated path variables with platform-specific separators
- SELinux `:Z` flag is critical for proper file access in rootless containers
- Mounting at the same path as host is the only way to make symlinks work inside containers

**Primary recommendation:** Use `Vec<String>` for `--linked-dir` CLI flags, parse `KLOTHO_LINKED_DIRS` with `split(':')`, mount each directory at its canonical host path with `:Z` suffix, and completely remove all legacy environment variable support.

## Standard Stack

The established libraries/tools for this domain:

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| clap | 4.5+ | CLI argument parsing | Industry standard for Rust CLI apps, excellent derive macro support |
| std::env | stdlib | Environment variable access | Built-in, no external deps needed |
| std::path | stdlib | Path manipulation | Built-in, handles canonicalization |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| anyhow | 1.0 | Error handling | Already in use, provides context |
| PathBuf | stdlib | Path storage | Canonical path representation |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| clap derive | clap builder API | Derive is more concise, builder offers more control |
| split(':') | env::split_paths() | split_paths() is platform-aware (Windows uses `;`), but we know target is Linux |

**Installation:**
```bash
# Already in Cargo.toml
clap = { version = "4.5", features = ["derive"] }
anyhow = "1.0"
```

## Architecture Patterns

### Recommended Code Structure
```
src/
├── cli.rs                    # Add --linked-dir flag to Start command
├── commands/start.rs         # Refactor mount building logic
└── container.rs              # No changes needed
```

### Pattern 1: Repeatable CLI Flag with Vec
**What:** Using `Vec<String>` type with clap derive creates repeatable flags automatically
**When to use:** When users need to specify the same flag multiple times

**Example:**
```rust
// Source: https://docs.rs/clap/latest/clap/_derive/_tutorial/index.html
#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Start {
        /// Directories to mount at same path (repeatable)
        #[arg(long = "linked-dir")]
        linked_dirs: Vec<String>,

        // ... other fields
    },
}
```

**Usage:**
```bash
klotho start --linked-dir /home/user/kob --linked-dir /home/user/shared ~/project
```

### Pattern 2: Colon-Separated Environment Variable Parsing
**What:** Parse PATH-style environment variables with colon as separator
**When to use:** When accepting multiple paths in a single environment variable

**Example:**
```rust
// Parse KLOTHO_LINKED_DIRS
if let Ok(linked_dirs_var) = env::var("KLOTHO_LINKED_DIRS") {
    for dir in linked_dirs_var.split(':') {
        let dir = dir.trim();
        if !dir.is_empty() {
            // Process each directory
        }
    }
}
```

**Note:** Could use `std::env::split_paths()` for platform-agnostic parsing, but klotho targets Linux containers exclusively, so simple `split(':')` is sufficient and clearer.

### Pattern 3: Canonical Path Mounting
**What:** Mount directories at their canonical host path inside container
**When to use:** When symlinks in workspace need to resolve correctly

**Example:**
```rust
use std::path::PathBuf;

// Canonicalize the path
let canonical = PathBuf::from(dir)
    .canonicalize()
    .context("failed to resolve path")?;

// Mount at same path with SELinux label
mount_args.push("-v".to_string());
mount_args.push(format!("{}:{}:Z", canonical.display(), canonical.display()));
```

### Pattern 4: Merging CLI and Environment Sources
**What:** CLI flags override environment variables, but both sources are merged
**When to use:** When providing both environment and CLI configuration options

**Example:**
```rust
// Collect from both sources
let mut all_linked_dirs = Vec::new();

// First, parse environment variable
if let Ok(env_dirs) = env::var("KLOTHO_LINKED_DIRS") {
    for dir in env_dirs.split(':') {
        let dir = dir.trim();
        if !dir.is_empty() {
            all_linked_dirs.push(dir.to_string());
        }
    }
}

// Then, add CLI flags (can override or supplement)
all_linked_dirs.extend(cli_linked_dirs);

// Deduplicate if needed
all_linked_dirs.sort();
all_linked_dirs.dedup();
```

### Anti-Patterns to Avoid
- **Mounting at different path than host:** Breaks symlinks - the whole point of the feature
- **Using relative paths:** Always canonicalize before mounting
- **Forgetting SELinux labels:** Containers won't be able to read files without `:Z`
- **Not validating directory exists:** Podman will fail to start container if mount source doesn't exist

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Parse colon-separated paths | Custom split logic | `split(':')` or `env::split_paths()` | Handles empty values, trim whitespace, platform-aware if needed |
| Path resolution | String manipulation | `PathBuf::canonicalize()` | Resolves symlinks, handles `.` and `..`, absolute paths |
| Repeatable CLI flags | Manual vec building | `Vec<String>` with clap derive | Built-in ArgAction::Append, automatic help text |
| Deduplication | Manual loop | `sort()` then `dedup()` | Standard pattern, handles all edge cases |

**Key insight:** Path handling has many edge cases (symlinks, relative paths, missing directories, permissions). Use stdlib functions that have been battle-tested rather than string manipulation.

## Common Pitfalls

### Pitfall 1: Wrong Mount Path (Current Bug)
**What goes wrong:** Rust code mounts at `/home/agent/.klotho`, bash script mounts at same path as host. Symlinks don't resolve.
**Why it happens:** Copy-paste from other mount logic without understanding the feature's purpose
**How to avoid:** Always mount linked directories at their canonical host path: `$HOST_PATH:$HOST_PATH:Z`
**Warning signs:** Symlinks that work with bash script but break with Rust version

**Example of bug:**
```rust
// WRONG - current implementation
mount_args.push(format!("{}:/home/agent/.klotho:Z", kob));

// CORRECT - what bash script does
let canonical = PathBuf::from(kob).canonicalize()?;
mount_args.push(format!("{}:{}:Z", canonical.display(), canonical.display()));
```

### Pitfall 2: Missing SELinux Label
**What goes wrong:** Container starts but can't read files in mounted directories
**Why it happens:** Forgetting `:Z` suffix when running rootless containers on SELinux systems
**How to avoid:** Always append `:Z` to mount specifications for private unshared volumes
**Warning signs:** "Permission denied" errors when accessing mounted files

**From research:** The `:Z` option tells Podman to label the content with a private unshared label, setting the SELinux context to match the container process. Without it, SELinux blocks access even if Unix permissions would allow it.

### Pitfall 3: Non-Canonical Paths
**What goes wrong:** Paths with `..` or symlinks in them don't match what's expected inside container
**Why it happens:** Not canonicalizing before mounting
**How to avoid:** Always call `.canonicalize()` before building mount arguments
**Warning signs:** Mounts work but paths don't match, symlinks resolve to wrong locations

### Pitfall 4: Empty or Non-Existent Directories
**What goes wrong:** Container fails to start with cryptic error
**Why it happens:** Podman refuses to start if mount source doesn't exist
**How to avoid:** Check `PathBuf::exists()` before adding to mount args, provide clear error message
**Warning signs:** Container creation fails with "no such file or directory"

**Example:**
```rust
let path = PathBuf::from(dir);
if !path.exists() {
    eprintln!("warning: linked directory does not exist, skipping: {}", dir);
    continue;
}
let canonical = path.canonicalize()?;
```

### Pitfall 5: Legacy Variable Cleanup
**What goes wrong:** Forgetting to remove all references to old environment variables
**Why it happens:** Variables referenced in multiple places (code, docs, tests, planning docs)
**How to avoid:** Use grep to find ALL references before starting, create checklist
**Warning signs:** Deprecation warnings still appearing, tests failing

**Files to check:**
- `src/commands/start.rs` - Current implementation location
- `README.md` - User documentation
- `klotho` bash script - Legacy implementation (DELETE entire file)
- `.planning/` docs - Update for accuracy

## Code Examples

Verified patterns from official sources:

### Clap Vec Type for Repeatable Flags
```rust
// Source: https://docs.rs/clap/latest/clap/_derive/_tutorial/index.html
use clap::{Parser, Subcommand};

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Start {
        /// Directories to mount at same path for symlink resolution (repeatable)
        #[arg(long = "linked-dir")]
        linked_dirs: Vec<String>,

        #[arg(short, long)]
        agent: Option<String>,

        #[arg(short, long, default_value = "default")]
        name: String,

        paths: Vec<String>,
    },
}
```

### Environment Variable Parsing
```rust
// Parse colon-separated directories
use std::env;

fn parse_linked_dirs_env() -> Vec<String> {
    env::var("KLOTHO_LINKED_DIRS")
        .ok()
        .map(|s| {
            s.split(':')
                .filter_map(|dir| {
                    let trimmed = dir.trim();
                    if trimmed.is_empty() {
                        None
                    } else {
                        Some(trimmed.to_string())
                    }
                })
                .collect()
        })
        .unwrap_or_default()
}
```

### Path Canonicalization and Mount Building
```rust
use std::path::PathBuf;
use anyhow::{Context, Result};

fn build_linked_dir_mounts(dirs: &[String]) -> Result<Vec<String>> {
    let mut mount_args = Vec::new();

    for dir in dirs {
        let path = PathBuf::from(dir);

        // Check existence
        if !path.exists() {
            eprintln!("warning: linked directory does not exist, skipping: {}", dir);
            continue;
        }

        // Canonicalize to resolve symlinks and get absolute path
        let canonical = path
            .canonicalize()
            .context(format!("failed to resolve path: {}", dir))?;

        // Mount at same path with SELinux label
        mount_args.push("-v".to_string());
        mount_args.push(format!("{}:{}:Z", canonical.display(), canonical.display()));
    }

    Ok(mount_args)
}
```

### Complete Integration Pattern
```rust
// In commands/start.rs, in the run() function:

// Collect linked directories from both sources
let mut all_linked_dirs = Vec::new();

// Parse environment variable
if let Ok(env_dirs) = env::var("KLOTHO_LINKED_DIRS") {
    for dir in env_dirs.split(':') {
        let dir = dir.trim();
        if !dir.is_empty() {
            all_linked_dirs.push(dir.to_string());
        }
    }
}

// Add CLI flags
all_linked_dirs.extend(linked_dirs);

// Deduplicate
all_linked_dirs.sort();
all_linked_dirs.dedup();

// Build mount arguments
for dir in &all_linked_dirs {
    let path = PathBuf::from(dir);
    if !path.exists() {
        eprintln!("warning: linked directory does not exist, skipping: {}", dir);
        continue;
    }

    let canonical = path
        .canonicalize()
        .context(format!("failed to resolve linked directory: {}", dir))?;

    mount_args.push("-v".to_string());
    mount_args.push(format!("{}:{}:Z", canonical.display(), canonical.display()));
}

// Remove ALL legacy code:
// DELETE: Lines 92-102 (KLOTHO_KOB and AGENT_SESSION_KOB)
// DELETE: Lines 107-114 (AGENT_SESSION_EXTRA_MOUNTS)
// Keep KLOTHO_MOUNTS (lines 105-124) - different feature
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `KLOTHO_KOB` (single dir) | `KLOTHO_LINKED_DIRS` (colon-separated) | Phase 9 | Supports multiple external directories |
| Mount at `/home/agent/.klotho` | Mount at canonical host path | Phase 9 | Symlinks actually work |
| `AGENT_SESSION_KOB` | No fallback | Phase 9 | Clean break, simpler code |
| `AGENT_SESSION_EXTRA_MOUNTS` | Already replaced by `KLOTHO_MOUNTS` | Phase 6 | Different feature, leave unchanged |
| Bash script is canonical | Rust binary is canonical | Phase 7 | Bash script deleted in Phase 9 |

**Deprecated/outdated:**
- `KLOTHO_KOB`: Replaced by `KLOTHO_LINKED_DIRS` (no fallback support)
- `AGENT_SESSION_KOB`: Removed completely (no migration path)
- `AGENT_SESSION_EXTRA_MOUNTS`: Already migrated to `KLOTHO_MOUNTS` in Phase 6 (keep that migration)
- `klotho` bash script: Entire file deleted, Rust is now canonical

## Open Questions

1. **Should we validate that symlinks in workspace actually point to linked directories?**
   - What we know: Technically possible to check, but complex (need to walk workspace, resolve all symlinks)
   - What's unclear: Is the performance cost worth it? What if user has valid reason for other symlinks?
   - Recommendation: Don't validate - trust user to configure correctly, let them discover via errors if wrong

2. **What error message when linked directory doesn't exist?**
   - What we know: Bash script shows warning and skips, Podman would fail container start
   - What's unclear: Warning + skip vs hard error?
   - Recommendation: Warning + skip (matches bash behavior, more forgiving)

3. **Should CLI flags replace or append to environment variable?**
   - What we know: Both approaches have precedent (Docker does replace, some tools append)
   - What's unclear: User expectation
   - Recommendation: Append (merge both sources) - more flexible, users can override by not setting env var

## Sources

### Primary (HIGH confidence)
- [Clap derive tutorial](https://docs.rs/clap/latest/clap/_derive/_tutorial/index.html) - Vec types for repeatable flags
- [Podman run documentation](https://docs.podman.io/en/latest/markdown/podman-run.1.html) - Volume mount syntax, symlink behavior
- [Rust std::env documentation](https://doc.rust-lang.org/std/env/index.html) - Environment variable functions
- [Rust std::path documentation](https://doc.rust-lang.org/std/path/index.html) - Path canonicalization
- Current codebase (`src/commands/start.rs` lines 92-102) - Bug location confirmed
- Bash script (`klotho` lines 550-558) - Correct implementation reference

### Secondary (MEDIUM confidence)
- [Red Hat article on SELinux container labeling](https://developers.redhat.com/articles/2025/04/11/my-advice-selinux-container-labeling) - Explains :Z flag importance
- [Podman volume options documentation](https://docs.podman.io/en/v4.3/markdown/options/volume.html) - SELinux relabeling details
- [Rust forum on clap repeatable flags](https://users.rust-lang.org/t/how-to-let-clap-v4-allow-multiple-occurrences/137671) - Community patterns

### Tertiary (LOW confidence)
- [Docker bind mounts documentation](https://docs.docker.com/engine/storage/bind-mounts/) - General container mount concepts
- [Podman symlink issues](https://github.com/containers/podman/issues/6003) - Known challenges with symlinks (marked for validation)

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - clap 4.5 is current, stdlib functions are stable
- Architecture: HIGH - Patterns verified in official docs and current codebase
- Pitfalls: HIGH - Current bug confirmed in code, SELinux behavior well-documented
- Code examples: HIGH - Derived from official clap tutorial and Rust stdlib docs

**Research date:** 2026-01-27
**Valid until:** ~30 days (stable domain - Rust stdlib and clap 4.x are mature)

**Sources:**
- [Arg in clap - Rust](https://docs.rs/clap/latest/clap/struct.Arg.html)
- [Clap derive tutorial](https://docs.rs/clap/latest/clap/_derive/_tutorial/index.html)
- [Podman volume mount documentation](https://docs.podman.io/en/v4.3/markdown/options/volume.html)
- [Podman run documentation](https://docs.podman.io/en/latest/markdown/podman-run.1.html)
- [std::env::split_paths documentation](https://doc.rust-lang.org/std/env/fn.split_paths.html)
- [Red Hat SELinux container labeling guide](https://developers.redhat.com/articles/2025/04/11/my-advice-selinux-container-labeling)
- [Rust forum: clap multiple occurrences](https://users.rust-lang.org/t/how-to-let-clap-v4-allow-multiple-occurrences/137671)
- [Podman symlink handling discussion](https://github.com/containers/podman/issues/6003)
- [Docker bind mounts documentation](https://docs.docker.com/engine/storage/bind-mounts/)
