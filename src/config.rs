use crate::agent::AgentConfig;
use crate::resources;
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

/// Load agent config with XDG-style layering
///
/// Priority (highest to lowest):
/// 1. User config in ~/.config/klotho/agents/<agent>/config.conf
/// 2. User config in ~/.config/agent-session/agents/<agent>/config.conf (legacy)
/// 3. Embedded default config (compiled into binary)
///
/// Returns (config, used_legacy_path)
pub fn load_agent_config(agent: &str) -> Result<(AgentConfig, bool)> {
    // Load embedded default first (must exist)
    let embedded_content = resources::get_agent_config(agent).map_err(|_| {
        let available = resources::list_embedded_agents().join(", ");
        anyhow::anyhow!("unknown agent: {}\navailable agents: {}", agent, available)
    })?;

    let mut config =
        AgentConfig::from_keyvalue(&embedded_content).context("failed to parse embedded config")?;

    // Check for user config override
    let (config_home, is_legacy) = get_config_home();
    let user_config_path = config_home.join("agents").join(agent).join("config.conf");

    if user_config_path.exists() {
        let user_content =
            fs::read_to_string(&user_config_path).context("failed to read user config file")?;
        let user_config =
            AgentConfig::from_keyvalue(&user_content).context("failed to parse user config")?;

        // Merge user config on top of embedded config
        for (key, value) in user_config {
            config.insert(key, value);
        }
    }

    // Convert to AgentConfig struct
    let agent_config = AgentConfig::from_map(&config)?;
    Ok((agent_config, is_legacy))
}
