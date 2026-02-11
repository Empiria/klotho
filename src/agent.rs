use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use crate::project_config::VolumeSpec;

/// Agent configuration loaded from TOML config files
#[derive(Debug, Clone, Deserialize, Serialize)]
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
    /// Environment variables (array of KEY=value strings)
    pub env_vars: Vec<String>,
    /// Optional command to launch the agent through hapi for mobile PTY bridging
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hapi_cmd: Option<String>,
    /// Agent-specific optional mounts (checked at runtime, skipped if missing)
    #[serde(default)]
    pub optional_mounts: Vec<VolumeSpec>,
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
    fn test_parse_toml_basic() {
        let content = r#"
# Comment
name = "claude"
description = "Anthropic Claude Code agent"
shell = "/usr/bin/fish"
install_cmd = "curl -fsSL https://claude.ai/install.sh | bash"
launch_cmd = "claude --dangerously-skip-permissions"
env_vars = ["PATH=/home/agent/.local/bin:$PATH", "SHELL=/usr/bin/fish"]
hapi_cmd = "hapi --dangerously-skip-permissions"
"#;
        let config: AgentConfig = toml::from_str(content).unwrap();
        assert_eq!(config.name, "claude");
        assert_eq!(config.description, "Anthropic Claude Code agent");
        assert_eq!(config.shell, "/usr/bin/fish");
        assert_eq!(config.env_vars, vec![
            "PATH=/home/agent/.local/bin:$PATH",
            "SHELL=/usr/bin/fish"
        ]);
        assert_eq!(config.hapi_cmd, Some("hapi --dangerously-skip-permissions".to_string()));
    }

    #[test]
    fn test_parse_toml_with_optional_mounts() {
        let content = r#"
name = "claude"
description = "Anthropic Claude Code agent"
shell = "/usr/bin/fish"
install_cmd = "curl -fsSL https://claude.ai/install.sh | bash"
launch_cmd = "claude --dangerously-skip-permissions"
env_vars = ["PATH=/home/agent/.local/bin:$PATH"]
hapi_cmd = "hapi --dangerously-skip-permissions"

[[optional_mounts]]
src = "~/.claude"
dest = "/home/agent/.claude"

[[optional_mounts]]
src = "~/.config/claude"
dest = "/home/agent/.config/claude"
readonly = true
"#;
        let config: AgentConfig = toml::from_str(content).unwrap();
        assert_eq!(config.name, "claude");
        assert_eq!(config.optional_mounts.len(), 2);

        // Check first mount (Detailed without readonly)
        match &config.optional_mounts[0] {
            VolumeSpec::Detailed { src, dest, readonly } => {
                assert_eq!(src, "~/.claude");
                assert_eq!(dest, "/home/agent/.claude");
                assert_eq!(*readonly, false);
            }
            _ => panic!("Expected Detailed variant"),
        }

        // Check second mount (Detailed with readonly)
        match &config.optional_mounts[1] {
            VolumeSpec::Detailed { src, dest, readonly } => {
                assert_eq!(src, "~/.config/claude");
                assert_eq!(dest, "/home/agent/.config/claude");
                assert_eq!(*readonly, true);
            }
            _ => panic!("Expected Detailed variant"),
        }
    }

    #[test]
    fn test_optional_mounts_embedded() {
        // Test that embedded agent configs with optional_mounts deserialize correctly
        let content = r#"
name = "test-agent"
description = "Test agent"
shell = "/bin/bash"
install_cmd = "echo install"
launch_cmd = "echo launch"
env_vars = []

[[optional_mounts]]
src = "~/.test"
dest = "/home/agent/.test"
"#;
        let config: AgentConfig = toml::from_str(content).unwrap();
        assert_eq!(config.optional_mounts.len(), 1);
    }

    #[test]
    fn test_optional_mounts_user_override() {
        // Test that user override configs can add optional_mounts
        let content = r#"
name = "test-agent"
description = "Test agent with user overrides"
shell = "/bin/bash"
install_cmd = "echo install"
launch_cmd = "echo launch"
env_vars = ["CUSTOM=value"]

[[optional_mounts]]
src = "~/custom-mount"
dest = "/home/agent/custom"
readonly = true
"#;
        let config: AgentConfig = toml::from_str(content).unwrap();
        assert_eq!(config.optional_mounts.len(), 1);
        match &config.optional_mounts[0] {
            VolumeSpec::Detailed { src, dest, readonly } => {
                assert_eq!(src, "~/custom-mount");
                assert_eq!(dest, "/home/agent/custom");
                assert_eq!(*readonly, true);
            }
            _ => panic!("Expected Detailed variant"),
        }
    }

    #[test]
    fn test_parse_toml_env_vars_array() {
        let content = r#"
name = "opencode"
description = "OpenCode agent"
shell = "/usr/bin/fish"
install_cmd = "curl install"
launch_cmd = "opencode"
env_vars = [
    "PATH=/home/agent/.local/bin:$PATH",
    "SHELL=/usr/bin/fish",
    "OPENCODE_CONFIG_CONTENT={\"permission\":{\"*\":\"allow\"}}"
]
"#;
        let config: AgentConfig = toml::from_str(content).unwrap();
        assert_eq!(config.env_vars.len(), 3);
        assert_eq!(config.env_vars[2], r#"OPENCODE_CONFIG_CONTENT={"permission":{"*":"allow"}}"#);
    }
}
