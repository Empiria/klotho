use anyhow::{Context, Result};
use std::path::Path;

const PROJECT_CONFIG_TEMPLATE: &str = r#"# .klotho.toml - klotho project configuration
# Run `klotho build` after editing to apply changes.

# [project]
# # Default agent for this project (skips interactive menu)
# agent = "claude"
# # Session name template (default: directory name)
# # name = "my-project"
# # Working directory inside container
# # workdir = "/workspace/subdir"

# # Project-specific volumes
# # Simple syntax: path mounts at same location in container
# # Detailed syntax: { src = "/host/path", dest = "/container/path", readonly = false }
# volumes = [
#     "/home/user/shared-libs",
#     { src = "/host/data", dest = "/workspace/data", readonly = true },
# ]

# [packages.apt]
# # System packages (Debian/Ubuntu)
# gcc = "*"
# build-essential = "*"

# [packages.pip]
# # Python packages (pip install)
# pytest = ">=7.0"

# [packages.npm]
# # Node.js packages (npm install -g)
# typescript = "^5.0"

# [packages.cargo]
# # Rust packages (cargo install)
# ripgrep = "*"
#
# # Known runtimes:
# # rustup = "*"   -> installs Rust toolchain via rustup
# # nvm = "*"      -> installs Node.js via nvm (in [packages.npm])

# # MCP servers for this project (shared across agents)
# [mcp.servers.my-server]
# command = "uvx"
# args = ["my-mcp-server"]
# # env = { API_KEY = "..." }
#
# # Agent-specific MCP servers (replaces shared for that agent)
# [mcp.claude.custom-tool]
# command = "npx"
# args = ["-y", "my-tool"]

# # Agent credentials (API keys for AI agents)
# # Use environment variable references to avoid storing secrets in config files
# [agents.claude]
# api_key = "${ANTHROPIC_API_KEY}"
#
# [agents.opencode]
# api_key = "${OPENAI_API_KEY}"  # or ANTHROPIC_API_KEY depending on provider
#
# # Set to false to disable mounting host config directories (~/.claude, ~/.config/opencode)
# # Default is false - credentials are configured above instead
# mount_host_config = false
"#;

const GLOBAL_CONFIG_TEMPLATE: &str = r#"# klotho global configuration
# Location: ~/.config/klotho/config.toml
# User-wide defaults for all projects.

# Container runtime preference (auto-detects if not set)
# Options: "podman", "docker", "auto"
# runtime = "auto"

# Default agent for new sessions (skips interactive menu)
# Options: "claude", "opencode", or any custom agent
# default_agent = "claude"

# Global volumes mounted in every session
# Simple syntax: path mounts at same location inside container
# Detailed syntax: { src = "/host/path", dest = "/container/path", readonly = false }
# volumes = [
#     "/home/user/shared-libs",
#     { src = "/etc/ssl/certs", dest = "/etc/ssl/certs", readonly = true },
# ]

# Global MCP servers available to all agents
# [mcp.servers.context7]
# command = "uvx"
# args = ["@upstash/context7-mcp"]

# # Agent credentials (default keys for all projects)
# # Project-level config can override these
# [agents.claude]
# api_key = "${ANTHROPIC_API_KEY}"
#
# # Set to true to mount host config directories (legacy behavior)
# # mount_host_config = true
"#;

/// Initialize a .klotho.toml file in the current directory or global config
pub fn run(global: bool) -> Result<()> {
    if global {
        scaffold_global_config()
    } else {
        scaffold_project_config()
    }
}

/// Scaffold project config (.klotho.toml)
fn scaffold_project_config() -> Result<()> {
    let config_path = Path::new(".klotho.toml");

    // Check if file already exists
    if config_path.exists() {
        anyhow::bail!(".klotho.toml already exists");
    }

    // Write template
    std::fs::write(config_path, PROJECT_CONFIG_TEMPLATE).context("Failed to write .klotho.toml")?;

    eprintln!("Created .klotho.toml - edit to add packages, then run `klotho build`");

    Ok(())
}

/// Scaffold global config (~/.config/klotho/config.toml)
fn scaffold_global_config() -> Result<()> {
    let config_home = crate::config::get_config_home();
    let config_path = config_home.join("config.toml");

    // Create directory if it doesn't exist
    std::fs::create_dir_all(&config_home)
        .context(format!("Failed to create {}", config_home.display()))?;

    // Check if file already exists
    if config_path.exists() {
        anyhow::bail!("{} already exists", config_path.display());
    }

    // Write template
    std::fs::write(&config_path, GLOBAL_CONFIG_TEMPLATE)
        .context(format!("Failed to write {}", config_path.display()))?;

    eprintln!(
        "Created {} - edit to set user-wide defaults",
        config_path.display()
    );

    Ok(())
}
