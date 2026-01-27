use anyhow::{Context, Result};
use rust_embed::RustEmbed;
use std::path::{Path, PathBuf};

#[derive(RustEmbed)]
#[folder = "src/resources/"]
pub struct Resources;

/// Get embedded Containerfile content
pub fn get_containerfile() -> Result<String> {
    let file = Resources::get("Containerfile")
        .context("Containerfile not found in embedded resources")?;
    Ok(String::from_utf8_lossy(&file.data).into_owned())
}

/// Get embedded entrypoint.sh content
pub fn get_entrypoint() -> Result<String> {
    let file = Resources::get("entrypoint.sh")
        .context("entrypoint.sh not found in embedded resources")?;
    Ok(String::from_utf8_lossy(&file.data).into_owned())
}

/// Get embedded agent config content
pub fn get_agent_config(agent: &str) -> Result<String> {
    let path = format!("agents/{}/config.conf", agent);
    let file = Resources::get(&path)
        .context(format!("Agent config not found: {}", path))?;
    Ok(String::from_utf8_lossy(&file.data).into_owned())
}

/// List available embedded agents
pub fn list_embedded_agents() -> Vec<String> {
    let mut agents = Vec::new();

    for path in Resources::iter() {
        // Look for agents/*/config.conf patterns
        if let Some(rest) = path.strip_prefix("agents/") {
            if let Some(agent) = rest.split('/').next() {
                if !agents.contains(&agent.to_string()) {
                    agents.push(agent.to_string());
                }
            }
        }
    }

    agents.sort();
    agents
}

/// Extract embedded resources to a temporary directory for building
/// Returns the path to the temp directory
pub fn extract_build_context() -> Result<PathBuf> {
    let temp_dir = std::env::temp_dir().join("klotho-build");

    // Clean and recreate
    if temp_dir.exists() {
        std::fs::remove_dir_all(&temp_dir)
            .context("Failed to clean temp directory")?;
    }
    std::fs::create_dir_all(&temp_dir)
        .context("Failed to create temp directory")?;

    // Write Containerfile
    let containerfile = get_containerfile()?;
    std::fs::write(temp_dir.join("Containerfile"), containerfile)
        .context("Failed to write Containerfile")?;

    // Write entrypoint.sh
    let entrypoint = get_entrypoint()?;
    std::fs::write(temp_dir.join("entrypoint.sh"), entrypoint)
        .context("Failed to write entrypoint.sh")?;

    // Make entrypoint executable (Unix only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(temp_dir.join("entrypoint.sh"))?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(temp_dir.join("entrypoint.sh"), perms)?;
    }

    // Write agent configs
    let config_dir = temp_dir.join("config").join("agents");
    for agent in list_embedded_agents() {
        let agent_dir = config_dir.join(&agent);
        std::fs::create_dir_all(&agent_dir)
            .context(format!("Failed to create agent dir: {}", agent))?;

        let config = get_agent_config(&agent)?;
        std::fs::write(agent_dir.join("config.conf"), config)
            .context(format!("Failed to write config for: {}", agent))?;

        // Also extract opencode.json if it exists
        if let Some(file) = Resources::get(&format!("agents/{}/opencode.json", agent)) {
            std::fs::write(agent_dir.join("opencode.json"), &file.data)
                .context("Failed to write opencode.json")?;
        }
    }

    Ok(temp_dir)
}

/// Check if running from repo (has local config/) or needs embedded resources
pub fn should_use_embedded() -> bool {
    // If local config/ exists, prefer it (development mode)
    !Path::new("config").exists()
}
