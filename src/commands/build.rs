use anyhow::{Context, Result};
use dialoguer::MultiSelect;
use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use std::io::{BufRead, BufReader};
use std::process::Stdio;

use crate::agent::{self, AgentConfig};
use crate::container::{self, Runtime};
use crate::resources;

/// Build command entry point
pub fn run(
    all: bool,
    agents: Vec<String>,
    no_cache: bool,
    runtime_override: Option<&str>,
) -> Result<()> {
    // Detect runtime
    let runtime = container::detect_runtime(runtime_override)?;

    // Determine which agents to build
    let agents_to_build = if all {
        get_all_agents()?
    } else if agents.is_empty() {
        select_agents_interactive()?
    } else {
        agents
    };

    if agents_to_build.is_empty() {
        eprintln!("{}", "No agents selected".yellow());
        return Ok(());
    }

    // Build each agent
    for agent in agents_to_build {
        run_build(runtime, &agent, no_cache)?;
    }

    Ok(())
}

/// Internal build function (also used by start command for auto-build)
pub fn run_build(runtime: Runtime, agent: &str, no_cache: bool) -> Result<()> {
    // Get build context (embedded or local)
    let build_context = if resources::should_use_embedded() {
        resources::extract_build_context()?
    } else {
        std::path::PathBuf::from(".")
    };

    // Load agent config to get build args
    let config_content = if resources::should_use_embedded() {
        resources::get_agent_config(agent)?
    } else {
        std::fs::read_to_string(format!("config/agents/{}/config.conf", agent))
            .context(format!("Agent config not found: {}", agent))?
    };

    let config_map = AgentConfig::from_keyvalue(&config_content)?;
    let agent_config = AgentConfig::from_map(&config_map)?;

    // Verify Containerfile has target stage
    let containerfile_path = build_context.join("Containerfile");
    let containerfile = std::fs::read_to_string(&containerfile_path)
        .context("Failed to read Containerfile")?;

    let stages = find_stages(&containerfile);
    if !stages.contains(&agent.to_string()) {
        anyhow::bail!(
            "Containerfile does not contain stage '{}'\nAvailable stages: {}",
            agent,
            stages.join(", ")
        );
    }

    // Prepare build command
    let image_name = format!("klotho-{}:latest", agent);
    let mut build_cmd = runtime.command();
    build_cmd
        .arg("build")
        .arg("-t")
        .arg(&image_name)
        .arg("--target")
        .arg(agent)
        .arg("--build-arg")
        .arg(format!("AGENT_NAME={}", agent_config.name))
        .arg("--build-arg")
        .arg(format!("AGENT_INSTALL_CMD={}", agent_config.install_cmd))
        .arg("--build-arg")
        .arg(format!("AGENT_SHELL={}", agent_config.shell))
        .arg("--build-arg")
        .arg(format!("AGENT_LAUNCH_CMD={}", agent_config.launch_cmd))
        .arg("-f")
        .arg(containerfile_path)
        .arg(&build_context)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    if no_cache {
        build_cmd.arg("--no-cache");
    }

    // Create spinner with steady tick for animation
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    spinner.set_message(format!("Building {} agent...", agent));
    spinner.enable_steady_tick(std::time::Duration::from_millis(100));

    // Run build and capture stderr for progress
    let mut child = build_cmd.spawn().context("Failed to start build command")?;

    // Read stderr for progress updates
    if let Some(stderr) = child.stderr.take() {
        let reader = BufReader::new(stderr);
        for line in reader.lines() {
            let line = line?;

            // Extract step info from build output
            if let Some(step) = extract_step_info(&line) {
                spinner.set_message(format!("Building {}: {}", agent, step));
            }

            // Spin the spinner to show progress
            spinner.tick();
        }
    }

    // Wait for completion
    let status = child.wait().context("Failed to wait for build")?;

    spinner.finish_and_clear();

    if status.success() {
        eprintln!(
            "{} Built {} → {}",
            "✓".green(),
            agent.bold(),
            image_name.cyan()
        );
        Ok(())
    } else {
        anyhow::bail!("Build failed for agent: {}", agent);
    }
}

/// Get all available agents
fn get_all_agents() -> Result<Vec<String>> {
    if resources::should_use_embedded() {
        Ok(resources::list_embedded_agents())
    } else {
        agent::discover_agents(&std::path::PathBuf::from("."))
    }
}

/// Interactive multi-select for choosing agents to build
fn select_agents_interactive() -> Result<Vec<String>> {
    let available_agents = get_all_agents()?;

    if available_agents.is_empty() {
        anyhow::bail!("No agents found");
    }

    let selections = MultiSelect::new()
        .with_prompt("Select agents to build (space to select, enter to confirm)")
        .items(&available_agents)
        .interact()?;

    let selected: Vec<String> = selections
        .iter()
        .map(|&i| available_agents[i].clone())
        .collect();

    Ok(selected)
}

/// Find all stages defined in Containerfile
fn find_stages(containerfile: &str) -> Vec<String> {
    let mut stages = Vec::new();

    // Use regex-lite to find "FROM ... AS stage_name" patterns
    for line in containerfile.lines() {
        let line = line.trim();
        if line.to_lowercase().starts_with("from") && line.to_lowercase().contains(" as ") {
            // Parse "FROM base AS claude" -> "claude"
            if let Some(as_pos) = line.to_lowercase().find(" as ") {
                let stage_name = line[as_pos + 4..].trim();
                // Handle comments after stage name
                let stage_name = stage_name.split_whitespace().next().unwrap_or("");
                if !stage_name.is_empty() {
                    stages.push(stage_name.to_string());
                }
            }
        }
    }

    stages
}

/// Extract step information from build progress line
fn extract_step_info(line: &str) -> Option<String> {
    let line = line.trim();

    // Podman/Docker build output patterns:
    // "STEP 1/5: FROM debian:bookworm-slim"
    // "#5 [stage 2/3] RUN apt-get update"
    // "[2/3] RUN curl ..."

    // Pattern: "STEP N/M: CMD args"
    if line.starts_with("STEP ") {
        if let Some(colon_pos) = line.find(':') {
            let rest = line[colon_pos + 1..].trim();
            // Take first 50 chars to avoid super long lines
            if rest.len() > 60 {
                return Some(format!("{}...", &rest[..60]));
            }
            return Some(rest.to_string());
        }
    }

    // Pattern: "#N [stage] CMD"
    if line.starts_with('#') && line.contains('[') {
        if let Some(bracket_pos) = line.find(']') {
            let rest = line[bracket_pos + 1..].trim();
            if rest.len() > 60 {
                return Some(format!("{}...", &rest[..60]));
            }
            return Some(rest.to_string());
        }
    }

    // Pattern: "[N/M] CMD"
    if line.starts_with('[') && line.contains(']') {
        if let Some(bracket_pos) = line.find(']') {
            let rest = line[bracket_pos + 1..].trim();
            if rest.len() > 60 {
                return Some(format!("{}...", &rest[..60]));
            }
            return Some(rest.to_string());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_stages() {
        let containerfile = r#"
FROM debian:bookworm-slim AS base
RUN apt-get update

FROM base AS claude
RUN install-claude

FROM base AS opencode  # with comment
RUN install-opencode
"#;
        let stages = find_stages(containerfile);
        assert_eq!(stages, vec!["base", "claude", "opencode"]);
    }

    #[test]
    fn test_extract_step_info_podman() {
        assert_eq!(
            extract_step_info("STEP 1/5: FROM debian:bookworm-slim"),
            Some("FROM debian:bookworm-slim".to_string())
        );
        assert_eq!(
            extract_step_info("STEP 3/5: RUN apt-get update && apt-get install -y git"),
            Some("RUN apt-get update && apt-get install -y git".to_string())
        );
    }

    #[test]
    fn test_extract_step_info_docker() {
        assert_eq!(
            extract_step_info("#5 [stage 2/3] RUN apt-get update"),
            Some("RUN apt-get update".to_string())
        );
    }

    #[test]
    fn test_extract_step_info_none() {
        assert_eq!(extract_step_info("Some random output"), None);
        assert_eq!(extract_step_info("Successfully built image"), None);
    }
}
