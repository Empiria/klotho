use crate::agent::AgentConfig;
use crate::project_config::{
    AgentCredentials, KlothoConfig, McpConfig, Packages, ProjectMeta, VolumeSpec,
};
use crate::resources;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;

#[cfg(test)]
use crate::project_config::McpServerConfig;

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

/// Get config home with XDG fallback
///
/// Returns ~/.config/klotho (or XDG_CONFIG_HOME/klotho if set)
pub fn get_config_home() -> PathBuf {
    let config_base = get_xdg_config_home();
    config_base.join("klotho")
}

/// Load agent config with XDG-style layering
///
/// Priority (highest to lowest):
/// 1. User config in ~/.config/klotho/agents/<agent>/config.toml
/// 2. Embedded default config (compiled into binary)
///
/// Returns the merged agent config
pub fn load_agent_config(agent: &str) -> Result<AgentConfig> {
    // Load embedded default first (must exist)
    let embedded_content = resources::get_agent_config(agent).map_err(|_| {
        let available = resources::list_embedded_agents().join(", ");
        anyhow::anyhow!("unknown agent: {}\navailable agents: {}", agent, available)
    })?;

    let mut config: AgentConfig =
        toml::from_str(&embedded_content).context("failed to parse embedded config")?;

    // Check for user config override
    let config_home = get_config_home();
    let user_config_path = config_home.join("agents").join(agent).join("config.toml");

    if user_config_path.exists() {
        let user_content =
            fs::read_to_string(&user_config_path).context("failed to read user config file")?;
        let user_config: AgentConfig =
            toml::from_str(&user_content).context("failed to parse user config")?;

        // User config completely replaces embedded config
        config = user_config;
    }

    Ok(config)
}

/// Global user configuration from ~/.config/klotho/config.toml
#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct GlobalConfig {
    #[serde(default)]
    pub runtime: Option<String>,
    #[serde(default)]
    pub default_agent: Option<String>,
    #[serde(default)]
    pub volumes: Vec<VolumeSpec>,
    #[serde(default)]
    pub mcp: Option<McpConfig>,
    #[serde(default)]
    pub agents: HashMap<String, AgentCredentials>,
}

/// Resolved configuration after merging global and project configs
#[derive(Debug)]
pub struct ResolvedConfig {
    pub runtime: Option<String>,
    pub default_agent: Option<String>,
    pub volumes: Vec<VolumeSpec>,
    pub mcp: McpConfig,
    pub project: Option<ProjectMeta>,
    pub packages: Option<Packages>,
    pub agents: HashMap<String, AgentCredentials>,
    pub mount_host_config: bool,
}

/// Load global config from ~/.config/klotho/config.toml
pub fn load_global_config() -> Result<GlobalConfig> {
    let config_home = get_config_home();
    let config_path = config_home.join("config.toml");

    if !config_path.exists() {
        return Ok(GlobalConfig::default());
    }

    let contents = fs::read_to_string(&config_path)
        .context(format!("Failed to read {}", config_path.display()))?;

    toml::from_str(&contents).context(format!("Failed to parse {} as TOML", config_path.display()))
}

/// Merge global and project configurations
/// Project config takes precedence over global for scalar values
/// Volumes are additive (global + project, with deduplication)
/// MCP configs merge: project agent-specific replaces global agent-specific, shared servers are additive
/// Agents: merge, project credentials override global for same agent
pub fn merge_configs(global: &GlobalConfig, project: &KlothoConfig) -> ResolvedConfig {
    // Runtime: project doesn't have this field, use global
    let runtime = global.runtime.clone();

    // Default agent: project.project.agent overrides global.default_agent
    let default_agent = project
        .project
        .as_ref()
        .and_then(|p| p.agent.clone())
        .or_else(|| global.default_agent.clone());

    // Volumes: concatenate and deduplicate (project overrides global for same source)
    let mut volumes = global.volumes.clone();
    volumes.extend(project.volumes.clone());
    let volumes = deduplicate_volumes(volumes);

    // MCP: merge configs
    let mcp = merge_mcp(global.mcp.as_ref(), project.mcp.as_ref());

    // Agents: merge, project credentials override global for same agent
    let mut agents = global.agents.clone();
    for (name, creds) in &project.agents {
        agents.insert(name.clone(), creds.clone());
    }

    // mount_host_config: project overrides global (default false)
    let mount_host_config = project.mount_host_config;

    ResolvedConfig {
        runtime,
        default_agent,
        volumes,
        mcp,
        project: project.project.clone(),
        packages: project.packages.clone(),
        agents,
        mount_host_config,
    }
}

/// Deduplicate volumes by source path, keeping last occurrence (project overrides global)
fn deduplicate_volumes(volumes: Vec<VolumeSpec>) -> Vec<VolumeSpec> {
    let mut seen = HashMap::new();
    for (i, vol) in volumes.iter().enumerate() {
        let src = match vol {
            VolumeSpec::Simple(p) => p.clone(),
            VolumeSpec::Detailed { src, .. } => src.clone(),
        };
        seen.insert(src, i);
    }
    let mut indices: Vec<usize> = seen.into_values().collect();
    indices.sort();
    indices.into_iter().map(|i| volumes[i].clone()).collect()
}

/// Merge MCP configurations
/// Project agent-specific sections completely replace global agent-specific sections
/// Shared servers are merged additively (project adds to global)
fn merge_mcp(global: Option<&McpConfig>, project: Option<&McpConfig>) -> McpConfig {
    let mut result = McpConfig::default();

    // Start with global shared servers
    if let Some(g) = global {
        result.servers.extend(g.servers.clone());
        result.claude.extend(g.claude.clone());
        result.opencode.extend(g.opencode.clone());
    }

    // Merge project config
    if let Some(p) = project {
        // Shared servers are additive (project can add more)
        result.servers.extend(p.servers.clone());

        // Agent-specific sections: if project defines any servers for an agent,
        // it completely replaces global for that agent
        if !p.claude.is_empty() {
            result.claude = p.claude.clone();
        }
        if !p.opencode.is_empty() {
            result.opencode = p.opencode.clone();
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_global_config_from_toml() {
        let toml_content = r#"
runtime = "podman"
default_agent = "claude"

[[volumes]]
src = "/global/data"
dest = "/data"

[mcp.servers.global-server]
command = "global-cmd"
args = ["--global"]
"#;
        let config: GlobalConfig = toml::from_str(toml_content).unwrap();
        assert_eq!(config.runtime, Some("podman".to_string()));
        assert_eq!(config.default_agent, Some("claude".to_string()));
        assert_eq!(config.volumes.len(), 1);
        assert!(config.mcp.is_some());
    }

    #[test]
    fn test_merge_configs_global_only() {
        let global = GlobalConfig {
            runtime: Some("podman".to_string()),
            default_agent: Some("claude".to_string()),
            volumes: vec![VolumeSpec::Simple("/global/data".to_string())],
            mcp: None,
            agents: HashMap::new(),
        };
        let project = KlothoConfig::default();

        let resolved = merge_configs(&global, &project);

        assert_eq!(resolved.runtime, Some("podman".to_string()));
        assert_eq!(resolved.default_agent, Some("claude".to_string()));
        assert_eq!(resolved.volumes.len(), 1);
    }

    #[test]
    fn test_merge_configs_project_overrides_agent() {
        let global = GlobalConfig {
            runtime: Some("docker".to_string()),
            default_agent: Some("opencode".to_string()),
            volumes: vec![],
            mcp: None,
            agents: HashMap::new(),
        };
        let mut project = KlothoConfig::default();
        project.project = Some(ProjectMeta {
            agent: Some("claude".to_string()),
            name: None,
            workdir: None,
        });

        let resolved = merge_configs(&global, &project);

        // Project agent overrides global default_agent
        assert_eq!(resolved.default_agent, Some("claude".to_string()));
        // Runtime comes from global (project doesn't have this field)
        assert_eq!(resolved.runtime, Some("docker".to_string()));
    }

    #[test]
    fn test_merge_configs_volume_deduplication() {
        let global = GlobalConfig {
            runtime: None,
            default_agent: None,
            volumes: vec![
                VolumeSpec::Simple("/data".to_string()),
                VolumeSpec::Detailed {
                    src: "/config".to_string(),
                    dest: "/etc/config".to_string(),
                    readonly: true,
                },
            ],
            mcp: None,
            agents: HashMap::new(),
        };
        let mut project = KlothoConfig::default();
        project.volumes = vec![
            VolumeSpec::Simple("/project/data".to_string()),
            VolumeSpec::Detailed {
                src: "/data".to_string(), // Same src as global - should replace
                dest: "/project/data".to_string(),
                readonly: false,
            },
        ];

        let resolved = merge_configs(&global, &project);

        // Should have 3 unique volumes (/data replaced, /config kept, /project/data added)
        assert_eq!(resolved.volumes.len(), 3);

        // Find the /data volume - should be the project version
        let data_vol = resolved.volumes.iter().find(|v| {
            matches!(v, VolumeSpec::Detailed { src, dest, .. } if src == "/data" && dest == "/project/data")
        });
        assert!(data_vol.is_some());
    }

    #[test]
    fn test_merge_mcp_shared_servers_additive() {
        let mut global_mcp = McpConfig::default();
        global_mcp.servers.insert(
            "global1".to_string(),
            McpServerConfig {
                command: "cmd1".to_string(),
                args: vec![],
                env: HashMap::new(),
            },
        );

        let global = GlobalConfig {
            runtime: None,
            default_agent: None,
            volumes: vec![],
            mcp: Some(global_mcp),
            agents: HashMap::new(),
        };

        let mut project_mcp = McpConfig::default();
        project_mcp.servers.insert(
            "project1".to_string(),
            McpServerConfig {
                command: "cmd2".to_string(),
                args: vec![],
                env: HashMap::new(),
            },
        );

        let mut project = KlothoConfig::default();
        project.mcp = Some(project_mcp);

        let resolved = merge_configs(&global, &project);

        // Both shared servers should be present
        assert_eq!(resolved.mcp.servers.len(), 2);
        assert!(resolved.mcp.servers.contains_key("global1"));
        assert!(resolved.mcp.servers.contains_key("project1"));
    }

    #[test]
    fn test_merge_mcp_agent_specific_replaces() {
        let mut global_mcp = McpConfig::default();
        global_mcp.claude.insert(
            "global-claude".to_string(),
            McpServerConfig {
                command: "global-cmd".to_string(),
                args: vec![],
                env: HashMap::new(),
            },
        );

        let global = GlobalConfig {
            runtime: None,
            default_agent: None,
            volumes: vec![],
            mcp: Some(global_mcp),
            agents: HashMap::new(),
        };

        let mut project_mcp = McpConfig::default();
        project_mcp.claude.insert(
            "project-claude".to_string(),
            McpServerConfig {
                command: "project-cmd".to_string(),
                args: vec![],
                env: HashMap::new(),
            },
        );

        let mut project = KlothoConfig::default();
        project.mcp = Some(project_mcp);

        let resolved = merge_configs(&global, &project);

        // Project claude config should completely replace global
        assert_eq!(resolved.mcp.claude.len(), 1);
        assert!(resolved.mcp.claude.contains_key("project-claude"));
        assert!(!resolved.mcp.claude.contains_key("global-claude"));
    }

    #[test]
    fn test_deduplicate_volumes_keeps_last() {
        let volumes = vec![
            VolumeSpec::Simple("/data".to_string()),
            VolumeSpec::Simple("/config".to_string()),
            VolumeSpec::Detailed {
                src: "/data".to_string(),
                dest: "/new/data".to_string(),
                readonly: true,
            },
        ];

        let deduped = deduplicate_volumes(volumes);

        // Should have 2 volumes: /config and /data (detailed version)
        assert_eq!(deduped.len(), 2);

        // /data should be the detailed version (last occurrence)
        let data_vol = deduped
            .iter()
            .find(|v| matches!(v, VolumeSpec::Detailed { src, .. } if src == "/data"));
        assert!(data_vol.is_some());
    }

    #[test]
    fn test_load_global_config_missing_file() {
        // This test would require setting up a temporary directory
        // For now, we test that default is returned correctly
        let default = GlobalConfig::default();
        assert!(default.runtime.is_none());
        assert!(default.default_agent.is_none());
        assert_eq!(default.volumes.len(), 0);
        assert!(default.mcp.is_none());
        assert!(default.agents.is_empty());
    }

    #[test]
    fn test_merge_agents_global_into_resolved() {
        use crate::project_config::AgentCredentials;

        let mut global_agents = HashMap::new();
        global_agents.insert(
            "claude".to_string(),
            AgentCredentials {
                api_key: Some("global-key".to_string()),
            },
        );

        let global = GlobalConfig {
            runtime: None,
            default_agent: None,
            volumes: vec![],
            mcp: None,
            agents: global_agents,
        };
        let project = KlothoConfig::default();

        let resolved = merge_configs(&global, &project);

        assert_eq!(resolved.agents.len(), 1);
        assert_eq!(
            resolved.agents.get("claude").unwrap().api_key,
            Some("global-key".to_string())
        );
    }

    #[test]
    fn test_merge_agents_project_overrides_global() {
        use crate::project_config::AgentCredentials;

        let mut global_agents = HashMap::new();
        global_agents.insert(
            "claude".to_string(),
            AgentCredentials {
                api_key: Some("global-key".to_string()),
            },
        );

        let global = GlobalConfig {
            runtime: None,
            default_agent: None,
            volumes: vec![],
            mcp: None,
            agents: global_agents,
        };

        let mut project = KlothoConfig::default();
        project.agents.insert(
            "claude".to_string(),
            AgentCredentials {
                api_key: Some("project-key".to_string()),
            },
        );

        let resolved = merge_configs(&global, &project);

        // Project key should override global
        assert_eq!(resolved.agents.len(), 1);
        assert_eq!(
            resolved.agents.get("claude").unwrap().api_key,
            Some("project-key".to_string())
        );
    }

    #[test]
    fn test_merge_agents_additive() {
        use crate::project_config::AgentCredentials;

        let mut global_agents = HashMap::new();
        global_agents.insert(
            "claude".to_string(),
            AgentCredentials {
                api_key: Some("claude-key".to_string()),
            },
        );

        let global = GlobalConfig {
            runtime: None,
            default_agent: None,
            volumes: vec![],
            mcp: None,
            agents: global_agents,
        };

        let mut project = KlothoConfig::default();
        project.agents.insert(
            "opencode".to_string(),
            AgentCredentials {
                api_key: Some("opencode-key".to_string()),
            },
        );

        let resolved = merge_configs(&global, &project);

        // Both agents should be present
        assert_eq!(resolved.agents.len(), 2);
        assert_eq!(
            resolved.agents.get("claude").unwrap().api_key,
            Some("claude-key".to_string())
        );
        assert_eq!(
            resolved.agents.get("opencode").unwrap().api_key,
            Some("opencode-key".to_string())
        );
    }

    #[test]
    fn test_mount_host_config_defaults_false() {
        let global = GlobalConfig {
            runtime: None,
            default_agent: None,
            volumes: vec![],
            mcp: None,
            agents: HashMap::new(),
        };
        let project = KlothoConfig::default();

        let resolved = merge_configs(&global, &project);

        assert!(!resolved.mount_host_config);
    }

    #[test]
    fn test_mount_host_config_from_project() {
        let global = GlobalConfig {
            runtime: None,
            default_agent: None,
            volumes: vec![],
            mcp: None,
            agents: HashMap::new(),
        };

        let mut project = KlothoConfig::default();
        project.mount_host_config = true;

        let resolved = merge_configs(&global, &project);

        assert!(resolved.mount_host_config);
    }
}
