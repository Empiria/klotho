use crate::agent::AgentConfig;
use anyhow::{Context, Result};
use std::env;
use std::fs;
use std::path::PathBuf;

/// Get XDG config home directory
///
/// Checks XDG_CONFIG_HOME environment variable, falls back to ~/.config
fn get_xdg_config_home() -> PathBuf {
    if let Ok(xdg) = env::var("XDG_CONFIG_HOME") {
        PathBuf::from(xdg)
    } else if let Ok(home) = env::var("HOME") {
        PathBuf::from(home).join(".config")
    } else {
        PathBuf::from(".config")
    }
}

/// Get config home with XDG fallback and klotho/agent-session layering
///
/// Priority:
/// 1. ~/.config/klotho (preferred)
/// 2. ~/.config/agent-session (legacy, shows deprecation warning)
/// 3. Default to ~/.config/klotho
pub fn get_config_home() -> (PathBuf, bool) {
    let config_base = get_xdg_config_home();
    let klotho_config = config_base.join("klotho");
    let legacy_config = config_base.join("agent-session");

    if klotho_config.exists() {
        return (klotho_config, false);
    }

    if legacy_config.exists() {
        return (legacy_config, true);
    }

    // Default to klotho for new installations
    (klotho_config, false)
}

/// Get repository config directory
///
/// Resolves script location by following symlinks and returns config/agents path
pub fn get_repo_config_dir() -> Result<PathBuf> {
    // Try to find config relative to current executable
    let exe = env::current_exe().context("failed to get current executable path")?;
    let exe_dir = exe
        .parent()
        .context("failed to get parent directory of executable")?;

    // In development, executable is in target/debug or target/release
    // In production, it should be in the repo root or a bin directory
    let mut search_paths = vec![
        exe_dir.to_path_buf(),                 // Same directory as executable
        exe_dir.join(".."),                    // Parent of executable
        exe_dir.join("../.."),                 // Two levels up (for target/debug)
        PathBuf::from("."),                    // Current working directory
        env::current_dir().unwrap_or_default(), // Explicit current directory
    ];

    // Also check if we're in a git repository
    if let Ok(output) = std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
    {
        if output.status.success() {
            if let Ok(git_root) = String::from_utf8(output.stdout) {
                search_paths.push(PathBuf::from(git_root.trim()));
            }
        }
    }

    for base in search_paths {
        let config_dir = base.join("config/agents");
        if config_dir.exists() {
            return Ok(config_dir);
        }
    }

    anyhow::bail!(
        "could not find config/agents directory\n\
         searched relative to executable and current directory"
    )
}

/// Load agent config with XDG-style layering
///
/// Loads repo default config first, then overlays user config if present.
/// Repo config is required, user config is optional.
///
/// Returns (config, used_legacy_path)
pub fn load_agent_config(agent: &str) -> Result<(AgentConfig, bool)> {
    let repo_config_dir = get_repo_config_dir()?;
    let repo_config_path = repo_config_dir.join(agent).join("config.conf");

    // Load repo default first (must exist)
    if !repo_config_path.exists() {
        anyhow::bail!(
            "no default config found for agent: {}\nexpected: {}",
            agent,
            repo_config_path.display()
        );
    }

    let repo_content = fs::read_to_string(&repo_config_path)
        .context("failed to read repo config file")?;
    let mut config = AgentConfig::from_keyvalue(&repo_content)
        .context("failed to parse repo config")?;

    // Check for user config override
    let (config_home, is_legacy) = get_config_home();
    let user_config_path = config_home.join("agents").join(agent).join("config.conf");

    if user_config_path.exists() {
        let user_content = fs::read_to_string(&user_config_path)
            .context("failed to read user config file")?;
        let user_config = AgentConfig::from_keyvalue(&user_content)
            .context("failed to parse user config")?;

        // Merge user config on top of repo config
        for (key, value) in user_config {
            config.insert(key, value);
        }
    }

    // Convert to AgentConfig struct
    let agent_config = AgentConfig::from_map(&config)?;
    Ok((agent_config, is_legacy))
}
