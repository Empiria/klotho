# Phase 7: Rust Rewrite - Research

**Researched:** 2026-01-27
**Domain:** Rust CLI application development
**Confidence:** HIGH

## Summary

Rust is the ideal choice for a CLI rewrite requiring single-binary distribution. The ecosystem has matured significantly, with established libraries for all required functionality: `clap` (argument parsing with derive macros), `anyhow` (error handling), `indicatif` (progress spinners), `owo-colors` (terminal color detection), `rust-embed` (embedding resources), and `std::process::Command` (subprocess management).

The standard approach is to use clap's derive API for ergonomic argument parsing, anyhow for application-level error propagation, and cross-compile with GitHub Actions using the `cross` tool or `rust-build` action. Static binaries with musl on Linux eliminate runtime dependencies, and rust-embed allows embedding Containerfiles and configs directly into the binary.

**Primary recommendation:** Use clap 4.x with derive macros for CLI structure, anyhow for errors, indicatif for spinners, shell out to podman/docker via std::process::Command, and cross-compile with GitHub Actions for multi-platform releases.

## Standard Stack

The established libraries/tools for Rust CLI applications:

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| clap | 4.5+ | Argument parsing, help generation, subcommands | Industry standard, merged structopt derive API, excellent ergonomics |
| anyhow | 1.0+ | Application error handling with context | Standard for CLI apps, simpler than custom error types |
| indicatif | 0.17+ | Progress bars and spinners | Gold standard for terminal progress indicators |
| owo-colors | 4.0+ | Terminal color output with auto-detection | Supports NO_COLOR, CLICOLOR standards, good performance |
| rust-embed | 8.11+ | Embed files into binary at compile time | Standard for single-binary distribution with assets |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| serde | 1.0+ | Serialization/deserialization | Config file parsing (with toml crate) |
| toml | 0.8+ | TOML parsing | Reading .conf files with KEY=value format |
| dialoguer | 0.11+ | Interactive prompts and menus | Agent selection menu, confirmations |
| inquire | 0.7+ | Alternative interactive prompts | Alternative to dialoguer, more features |
| cross | 0.2+ | Cross-compilation tool | Building for multiple targets in CI/CD |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| anyhow | thiserror | thiserror for libraries needing typed errors, anyhow simpler for apps |
| owo-colors | colored, termcolor | All support NO_COLOR; owo-colors has good API and performance |
| dialoguer | inquire | inquire has more features; dialoguer simpler and lighter |
| cross | cargo + rustup targets | cross uses Docker for true cross-compilation, rustup native only |

**Installation:**
```toml
[dependencies]
clap = { version = "4.5", features = ["derive"] }
anyhow = "1.0"
indicatif = "0.17"
owo-colors = "4.0"
rust-embed = "8.11"
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"
dialoguer = "0.11"
```

## Architecture Patterns

### Recommended Project Structure
```
src/
├── main.rs           # Entry point, CLI definition, dispatch
├── cli.rs            # Clap structs and subcommand definitions
├── commands/         # One module per command
│   ├── mod.rs        # Re-exports
│   ├── start.rs      # start command implementation
│   ├── stop.rs
│   ├── build.rs
│   └── ...
├── container.rs      # Podman/Docker abstraction
├── config.rs         # Config file loading and validation
├── agent.rs          # Agent discovery and selection
├── error.rs          # Error types (if needed beyond anyhow)
└── resources/        # Embedded files (via rust-embed)
    ├── Containerfile
    └── config/
```

### Pattern 1: Clap Derive API
**What:** Define CLI structure using derive macros on structs
**When to use:** All CLI argument parsing (preferred over builder API)
**Example:**
```rust
// Source: https://docs.rs/clap/latest/clap/
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "klotho")]
#[command(about = "Run AI agents in isolated containers")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new session or attach to existing one
    Start {
        #[arg(short, long, default_value = "claude")]
        agent: String,

        #[arg(short, long, default_value = "default")]
        name: String,

        /// Project paths to mount
        paths: Vec<String>,
    },
    // Other subcommands...
}
```

### Pattern 2: Anyhow for Error Propagation
**What:** Use anyhow::Result for all fallible operations, add context with .context()
**When to use:** Application-level error handling (not libraries)
**Example:**
```rust
// Source: https://docs.rs/anyhow/latest/anyhow/
use anyhow::{Context, Result};

fn load_config(path: &Path) -> Result<Config> {
    let contents = std::fs::read_to_string(path)
        .context("Failed to read config file")?;

    toml::from_str(&contents)
        .context("Failed to parse config file")
}

fn main() -> Result<()> {
    let config = load_config(&config_path)?;
    // ... rest of logic
    Ok(())
}
```

### Pattern 3: Subprocess Command Output Handling
**What:** Use std::process::Command with proper output capture and error handling
**When to use:** Shelling out to podman/docker
**Example:**
```rust
// Source: https://doc.rust-lang.org/std/process/struct.Command.html
use std::process::Command;
use anyhow::{bail, Result};

fn podman_ps() -> Result<Vec<String>> {
    let output = Command::new("podman")
        .args(&["ps", "-a", "--format", "{{.Names}}"])
        .output()
        .context("Failed to execute podman")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("podman ps failed: {}", stderr);
    }

    let stdout = String::from_utf8(output.stdout)?;
    Ok(stdout.lines().map(String::from).collect())
}
```

### Pattern 4: Embedded Resources
**What:** Embed Containerfile and configs into binary using rust-embed
**When to use:** Single-binary distribution with default resources
**Example:**
```rust
// Source: https://docs.rs/rust-embed/latest/rust_embed/
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "resources/"]
struct Resources;

fn get_containerfile() -> String {
    let file = Resources::get("Containerfile")
        .expect("Containerfile not found");
    String::from_utf8_lossy(&file.data).into_owned()
}
```

### Pattern 5: Interactive Selection
**What:** Use dialoguer for interactive menus with keyboard navigation
**When to use:** Agent selection when no --agent flag provided
**Example:**
```rust
// Source: https://docs.rs/dialoguer/latest/dialoguer/
use dialoguer::Select;

fn select_agent(agents: &[String]) -> Result<String> {
    let selection = Select::new()
        .with_prompt("Select agent")
        .items(agents)
        .default(0)
        .interact()?;

    Ok(agents[selection].clone())
}
```

### Pattern 6: Progress Spinner
**What:** Use indicatif ProgressBar in spinner mode with steady tick
**When to use:** Long-running operations like building containers
**Example:**
```rust
// Source: https://docs.rs/indicatif/latest/indicatif/
use indicatif::{ProgressBar, ProgressStyle};

fn build_with_progress(agent: &str) -> Result<()> {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
    );
    spinner.set_message(format!("Building {}...", agent));
    spinner.enable_steady_tick(std::time::Duration::from_millis(100));

    // Run build command
    let result = run_build(agent);

    spinner.finish_with_message("Done");
    result
}
```

### Pattern 7: Color Auto-Detection
**What:** Use owo-colors with auto-detection, respect NO_COLOR
**When to use:** All colored output
**Example:**
```rust
// Source: https://docs.rs/owo-colors/latest/owo_colors/
use owo_colors::OwoColorize;

fn print_status(name: &str, running: bool) {
    if running {
        println!("{} {}", name, "running".green());
    } else {
        println!("{} {}", name, "stopped".red());
    }
}
```

### Anti-Patterns to Avoid
- **Using .unwrap() or .expect() in production code**: Use ? operator or match for proper error handling. unwrap/expect crash the program with poor error messages.
- **Mixing domain logic with I/O**: Keep command implementations separate from business logic for testability and reusability.
- **Not using .context() on errors**: Bare errors from libraries don't tell users what operation failed. Always add context.
- **Parsing strings instead of structured output**: When shelling out, use structured formats (--format JSON) where possible instead of parsing human-readable output.

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Argument parsing | Manual args iteration, string matching | clap with derive API | Handles help text, validation, subcommands, shell completion |
| Progress indicators | Print dots or custom spinners | indicatif | Handles terminal detection, cleanup, multiple progress bars |
| Terminal colors | ANSI escape codes manually | owo-colors or termcolor | NO_COLOR support, terminal capability detection, cross-platform |
| Interactive menus | Print numbered list, read stdin | dialoguer or inquire | Keyboard navigation, validation, consistent UX |
| Config parsing | Custom parsers | serde + toml | Handles types, validation, nested structures |
| Cross-compilation | Manual toolchain setup | cross or GitHub Actions | Handles linking, libc differences, target-specific quirks |
| Error messages | String concatenation | anyhow with .context() | Stack traces, chaining, automatic formatting |
| Embedding files | include_str! macro | rust-embed | Directory support, compression, lazy loading |

**Key insight:** The Rust CLI ecosystem is mature. Almost every common CLI task has a well-maintained, battle-tested library. Using these libraries reduces bugs, improves UX consistency, and saves development time.

## Common Pitfalls

### Pitfall 1: Overusing unwrap/expect in Production
**What goes wrong:** Application crashes with unhelpful panic messages instead of graceful error handling
**Why it happens:** Examples use .unwrap() for brevity; developers copy this pattern into production code
**How to avoid:**
- Use ? operator for error propagation in functions returning Result
- Return Result from main() for top-level error handling
- Use .unwrap_or() / .unwrap_or_else() for Options with sensible defaults
- Only use .expect() with detailed messages explaining impossible conditions
**Warning signs:** "thread 'main' panicked at 'called `Option::unwrap()` on a `None` value'" in logs

### Pitfall 2: Incomplete Script Download Attack (curl | sh)
**What goes wrong:** Installer script executes partially if download interrupted, leaving system in broken state
**Why it happens:** bash starts executing before script fully downloaded
**How to avoid:** Wrap entire installer script in a function, call function at end of script
**Warning signs:** User reports installation failure mid-way with system changes already applied
**Example:**
```bash
# Bad: executes line by line
echo "Downloading binary..."
curl -L $URL -o /usr/local/bin/klotho

# Good: wrapped in function
main() {
    echo "Downloading binary..."
    curl -L $URL -o /usr/local/bin/klotho
    chmod +x /usr/local/bin/klotho
}
main "$@"  # Only executes if fully downloaded
```

### Pitfall 3: Static Binary Not Actually Static
**What goes wrong:** "Static" Linux binary fails with glibc version errors on older systems
**Why it happens:** Default Linux target (x86_64-unknown-linux-gnu) links against glibc dynamically
**How to avoid:** Use x86_64-unknown-linux-musl target for true static binaries
**Warning signs:** Binary works on build machine but fails on deployment with "GLIBC_2.XX not found"
**Solution:**
```bash
# Add musl target
rustup target add x86_64-unknown-linux-musl

# Build static binary
cargo build --release --target x86_64-unknown-linux-musl
```

### Pitfall 4: Mixing Sync and Async Runtime
**What goes wrong:** Dead-locks, performance issues, or runtime panics
**Why it happens:** Using tokio::process with std::thread, or blocking calls in async context
**How to avoid:** For CLI tools shelling out to external commands, use std::process::Command (sync). Don't add tokio unless truly needed.
**Warning signs:** "Cannot start a runtime from within a runtime" error, or mysterious hangs

### Pitfall 5: Not Testing on All Target Platforms
**What goes wrong:** Binary works on x86_64 Linux but crashes on aarch64 or macOS
**Why it happens:** Endianness issues, platform-specific APIs, linker differences
**How to avoid:**
- Use GitHub Actions matrix to build and test on all targets
- Test installers on each platform (Linux, macOS, Windows)
- Use cross for local testing of different architectures
**Warning signs:** Users report crashes only on specific platforms

### Pitfall 6: Subprocess Output Not Captured
**What goes wrong:** Error messages from podman/docker lost, making debugging impossible
**Why it happens:** Using .status() instead of .output(), not capturing stderr
**How to avoid:**
- Use .output() to capture stdout and stderr
- Log stderr on failure with context about what command ran
- For interactive commands (attach), use .status() but document why
**Warning signs:** "Command failed" errors with no details about what went wrong

### Pitfall 7: Path Handling with Strings
**What goes wrong:** Paths break on Windows (\ vs /), spaces cause splits, encoding issues
**Why it happens:** Using String and str for file paths instead of Path/PathBuf
**How to avoid:**
- Use std::path::Path and PathBuf for all file system paths
- Use .join() instead of string concatenation
- Convert to String only at boundaries (display to user, pass to Command)
**Warning signs:** "File not found" on Windows but works on Linux, issues with spaces in paths

### Pitfall 8: Cargo Install vs GitHub Releases Mismatch
**What goes wrong:** cargo install version different from GitHub release, incompatible behaviors
**Why it happens:** Forgetting to publish to crates.io, or publishing before testing release builds
**How to avoid:**
- Automate releases: tag triggers both GitHub release build AND crates.io publish
- Test release binaries from CI before publishing to crates.io
- Document in README which installation method is primary
**Warning signs:** Bug reports mentioning different versions despite same semver

## Code Examples

Verified patterns from official sources:

### Main Entry Point with Result
```rust
// Source: https://rust-cli.github.io/book/tutorial/errors.html
use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Start { agent, name, paths } => {
            commands::start::run(agent, name, paths)?;
        }
        Commands::Stop { name } => {
            commands::stop::run(name)?;
        }
        // ... other commands
    }

    Ok(())
}
```

### Container Runtime Detection
```rust
// Pattern: Detect podman, fallback to docker, error if neither
use std::process::Command;
use anyhow::{bail, Result};

fn detect_runtime() -> Result<String> {
    if Command::new("podman").arg("--version").output().is_ok() {
        return Ok("podman".to_string());
    }

    if Command::new("docker").arg("--version").output().is_ok() {
        eprintln!("Warning: podman not found, using docker");
        return Ok("docker".to_string());
    }

    bail!("Neither podman nor docker found. Install podman: https://podman.io/")
}
```

### Config File Parsing
```rust
// Source: https://docs.rs/toml/latest/toml/
use serde::Deserialize;
use std::path::Path;
use anyhow::{Context, Result};

#[derive(Deserialize)]
struct AgentConfig {
    agent_name: String,
    agent_description: String,
    agent_install_cmd: String,
    agent_launch_cmd: String,
    agent_shell: String,
}

fn load_agent_config(agent: &str, config_dir: &Path) -> Result<AgentConfig> {
    let config_path = config_dir.join("agents").join(agent).join("config.conf");

    let contents = std::fs::read_to_string(&config_path)
        .context(format!("Failed to read config: {:?}", config_path))?;

    toml::from_str(&contents)
        .context("Failed to parse agent config")
}
```

### Multi-Select Interactive Menu
```rust
// Source: https://docs.rs/dialoguer/latest/dialoguer/
use dialoguer::MultiSelect;
use anyhow::Result;

fn select_agents(agents: &[String]) -> Result<Vec<String>> {
    let selections = MultiSelect::new()
        .with_prompt("Select agents to build (space to select, enter to confirm)")
        .items(agents)
        .interact()?;

    Ok(selections.iter().map(|&i| agents[i].clone()).collect())
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| structopt crate | clap 4.x with derive feature | clap 3.0 (2021) | structopt merged into clap; use clap derive instead |
| colored crate | owo-colors, supports-color | 2022-2023 | Better NO_COLOR support, smaller compile times |
| error-chain | anyhow / thiserror | 2019-2020 | Simpler API, better error messages, lighter weight |
| rust-musl-builder Docker | cross tool | 2020-2021 | Easier setup, supports more targets, maintained |
| include_str! for files | rust-embed | 2019+ | Can embed directories, compression, better ergonomics |
| curl-rust crate | std::process::Command + curl binary | Always valid | Simpler, fewer deps, works with system curl |

**Deprecated/outdated:**
- **structopt**: Merged into clap 3.0+. Use `clap = { version = "4", features = ["derive"] }` instead.
- **rust-musl-builder**: Still works but cross tool is now preferred for cross-compilation.
- **error-chain**: No longer maintained. Use anyhow (apps) or thiserror (libraries).
- **questionnaire crate**: Unmaintained. Use dialoguer or inquire instead.

## Open Questions

Things that couldn't be fully resolved:

1. **Zellij attachment from Rust subprocess**
   - What we know: Current bash script uses `podman exec -it` to attach to Zellij sessions
   - What's unclear: Whether Rust std::process::Command handles interactive TTY correctly for `podman exec -it`
   - Recommendation: Shell out to `podman exec -it` with .status() (not .output()) to preserve TTY. Test thoroughly on Linux and macOS.

2. **Windows compatibility of container runtime**
   - What we know: Decision doc mentions Windows as target platform
   - What's unclear: Does Podman Desktop on Windows work with same CLI? Docker Desktop behavior?
   - Recommendation: Test on Windows early. May need runtime-specific paths or invocation differences. Consider documenting Windows as requiring Docker Desktop or Podman Desktop.

3. **Optimal embedded resource strategy**
   - What we know: rust-embed can embed Containerfile and configs
   - What's unclear: Should user overrides layer on top, or replace entirely? How to show user what's embedded?
   - Recommendation: Embed defaults, check ~/.config/klotho/ for overrides first. Add `klotho show-config` command to dump embedded defaults.

## Sources

### Primary (HIGH confidence)
- [Command Line Applications in Rust (official book)](https://rust-cli.github.io/book/index.html)
- [clap 4.5.54 documentation](https://docs.rs/clap/latest/clap/) - Current version with derive API
- [anyhow documentation](https://docs.rs/anyhow/latest/anyhow/)
- [indicatif documentation](https://docs.rs/indicatif/latest/indicatif/)
- [rust-embed documentation](https://docs.rs/rust-embed/latest/rust_embed/)
- [std::process::Command documentation](https://doc.rust-lang.org/std/process/struct.Command.html)
- [Rust Cookbook - External Commands](https://rust-lang-nursery.github.io/rust-cookbook/os/external.html)

### Secondary (MEDIUM confidence)
- [Rain's Rust CLI recommendations](https://rust-cli-recommendations.sunshowers.io/) - Community best practices
- [Picking an argument parser](https://rust-cli-recommendations.sunshowers.io/cli-parser.html)
- [Managing colors in Rust](https://rust-cli-recommendations.sunshowers.io/managing-colors-in-rust.html)
- [Cross Compiling Rust Projects in GitHub Actions](https://blog.urth.org/2023/03/05/cross-compiling-rust-projects-in-github-actions/)
- [How to Deploy Rust Binaries with GitHub Actions](https://dzfrias.dev/blog/deploy-rust-cross-platform-github-actions/)
- [Packaging and distributing a Rust tool](https://rust-cli.github.io/book/tutorial/packaging.html)
- [Fully Automated Releases for Rust Projects](https://blog.orhun.dev/automated-rust-releases/)
- [Comparison of Rust CLI Prompts](https://fadeevab.com/comparison-of-rust-cli-prompts/)

### Secondary (MEDIUM confidence - WebSearch verified)
- [rust-musl-cross GitHub](https://github.com/rust-cross/rust-musl-cross) - Docker images for musl compilation
- [houseabsolute/actions-rust-cross](https://github.com/houseabsolute/actions-rust-cross) - GitHub Action for cross-compilation
- [BurntSushi/termcolor](https://github.com/BurntSushi/termcolor) - Cross-platform terminal colors
- [NO_COLOR specification](https://no-color.org/) - Standard environment variable
- [supports-color crate](https://docs.rs/supports-color) - Terminal color capability detection

### Tertiary (LOW confidence - needs validation)
- WebSearch results on curl|sh security - opinions vary widely, needs case-by-case assessment
- WebSearch results on Windows Podman support - needs direct testing to confirm behavior
- Community forum discussions on error handling - good context but not authoritative

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - All libraries verified via docs.rs, active maintenance confirmed, version numbers current
- Architecture: HIGH - Patterns verified from official Rust book, docs.rs examples, community standard practices
- Pitfalls: MEDIUM-HIGH - Based on documented community issues, official warnings, and Rust book guidance
- Cross-compilation: MEDIUM - GitHub Actions approach well-documented but platform-specific issues need testing
- Windows support: LOW - Requires hands-on testing to validate podman/docker behavior

**Research date:** 2026-01-27
**Valid until:** Approximately 60 days (March 2026) - Rust ecosystem stable, but library versions update frequently. Clap and other core libraries backward-compatible within major versions.
