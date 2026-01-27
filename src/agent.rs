use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Agent configuration loaded from KEY=value config files
#[derive(Debug, Clone)]
pub struct AgentConfig {
    /// Agent identifier - must match directory name
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// Command to install the agent in the container
    pub install_cmd: String,
    /// Command to launch the agent
    pub launch_cmd: String,
    /// Default shell for the agent (full path)
    pub shell: String,
    /// Environment variables (space-separated KEY=value pairs)
    pub env_vars: String,
}

impl AgentConfig {
    /// Parse agent config from KEY=value format
    ///
    /// Format: shell-sourceable KEY=value pairs with quoted values
    /// Security: Rejects command substitution ($() or backticks)
    pub fn from_keyvalue(content: &str) -> Result<HashMap<String, String>> {
        let mut config = HashMap::new();

        for line in content.lines() {
            let line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Security check: reject command substitution
            if line.contains("$(") || line.contains('`') {
                anyhow::bail!(
                    "config contains command substitution ($() or backticks)\n\
                     config files may only contain KEY=value pairs and variable expansion ($VAR)"
                );
            }

            // Parse KEY=value (value may be quoted)
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let mut value = value.trim();

                // Remove surrounding quotes if present
                if (value.starts_with('"') && value.ends_with('"'))
                    || (value.starts_with('\'') && value.ends_with('\''))
                {
                    value = &value[1..value.len() - 1];
                }

                config.insert(key.to_string(), value.to_string());
            }
        }

        Ok(config)
    }

    /// Load agent config from parsed key-value map
    pub fn from_map(map: &HashMap<String, String>) -> Result<Self> {
        Ok(AgentConfig {
            name: map
                .get("AGENT_NAME")
                .context("missing AGENT_NAME in config")?
                .clone(),
            description: map
                .get("AGENT_DESCRIPTION")
                .context("missing AGENT_DESCRIPTION in config")?
                .clone(),
            install_cmd: map
                .get("AGENT_INSTALL_CMD")
                .context("missing AGENT_INSTALL_CMD in config")?
                .clone(),
            launch_cmd: map
                .get("AGENT_LAUNCH_CMD")
                .context("missing AGENT_LAUNCH_CMD in config")?
                .clone(),
            shell: map
                .get("AGENT_SHELL")
                .context("missing AGENT_SHELL in config")?
                .clone(),
            env_vars: map.get("AGENT_ENV_VARS").cloned().unwrap_or_default(),
        })
    }
}

/// Discover available agents from config directory
pub fn discover_agents(repo_dir: &PathBuf) -> Result<Vec<String>> {
    let agents_dir = repo_dir.join("config/agents");

    if !agents_dir.exists() {
        anyhow::bail!("no agents found in config/agents/");
    }

    let mut agents = Vec::new();

    for entry in fs::read_dir(&agents_dir)
        .context("failed to read config/agents directory")?
    {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                agents.push(name.to_string());
            }
        }
    }

    if agents.is_empty() {
        anyhow::bail!("no agents found in config/agents/");
    }

    agents.sort();
    Ok(agents)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_keyvalue_basic() {
        let content = r#"
# Comment
AGENT_NAME="claude"
AGENT_DESCRIPTION="Anthropic Claude Code agent"
AGENT_SHELL="/usr/bin/fish"
"#;
        let config = AgentConfig::from_keyvalue(content).unwrap();
        assert_eq!(config.get("AGENT_NAME"), Some(&"claude".to_string()));
        assert_eq!(
            config.get("AGENT_DESCRIPTION"),
            Some(&"Anthropic Claude Code agent".to_string())
        );
        assert_eq!(config.get("AGENT_SHELL"), Some(&"/usr/bin/fish".to_string()));
    }

    #[test]
    fn test_parse_keyvalue_rejects_command_substitution() {
        let content = r#"
AGENT_NAME="test"
AGENT_SHELL="$(whoami)"
"#;
        let result = AgentConfig::from_keyvalue(content);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("command substitution"));
    }

    #[test]
    fn test_parse_keyvalue_rejects_backticks() {
        let content = r#"
AGENT_NAME="test"
AGENT_SHELL="`whoami`"
"#;
        let result = AgentConfig::from_keyvalue(content);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("command substitution"));
    }

    #[test]
    fn test_parse_keyvalue_variable_expansion_allowed() {
        let content = r#"
AGENT_ENV_VARS="PATH=/home/agent/.local/bin:$PATH SHELL=/usr/bin/fish"
"#;
        let config = AgentConfig::from_keyvalue(content).unwrap();
        assert_eq!(
            config.get("AGENT_ENV_VARS"),
            Some(&"PATH=/home/agent/.local/bin:$PATH SHELL=/usr/bin/fish".to_string())
        );
    }
}
