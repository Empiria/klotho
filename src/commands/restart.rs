use anyhow::{bail, Context, Result};
use std::process::{Command, Stdio};
use crate::agent::AgentConfig;
use crate::config::load_agent_config;
use crate::container::{
    container_status, detect_runtime, find_container, start_container, ContainerStatus,
};

pub fn run(name: String, runtime_override: Option<&str>) -> Result<()> {
    let runtime = detect_runtime(runtime_override)?;

    // Find container by session name
    let container_name = match find_container(runtime, &name)? {
        Some(name) => name,
        None => bail!("Session '{}' not found", name),
    };

    // Check container status
    let status = container_status(runtime, &container_name)?;

    match status {
        ContainerStatus::Running => {
            println!("Session '{}' is already running. Attaching...", name);
        }
        ContainerStatus::Stopped => {
            println!("Starting '{}'...", name);
            start_container(runtime, &container_name)?;
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
        ContainerStatus::NotFound => {
            bail!("Session '{}' not found", name);
        }
    }

    // Extract agent type from container name
    // Container names: klotho-session-<agent>-<name> or <agent>-<name> (legacy)
    let agent_type = extract_agent_from_container(&container_name, &name)?;

    // Load agent config
    let (config, _is_legacy) = load_agent_config(&agent_type)?;

    // Attach to zellij
    attach_zellij(runtime, &container_name, &name, &config)
}

fn extract_agent_from_container(container_name: &str, session_name: &str) -> Result<String> {
    // Try new naming: klotho-session-<agent>-<name>
    if let Some(rest) = container_name.strip_prefix("klotho-session-") {
        if let Some(agent) = rest.strip_suffix(&format!("-{}", session_name)) {
            return Ok(agent.to_string());
        }
    }

    // Try legacy naming: <agent>-<name>
    if let Some(agent) = container_name.strip_suffix(&format!("-{}", session_name)) {
        return Ok(agent.to_string());
    }

    bail!("Cannot extract agent type from container name: {}", container_name);
}

fn attach_zellij(
    runtime: crate::container::Runtime,
    container_name: &str,
    session_name: &str,
    config: &AgentConfig,
) -> Result<()> {
    // Check if zellij session exists
    let check = Command::new(runtime.as_str())
        .args(["exec", container_name, "zellij", "list-sessions"])
        .output()
        .context("Failed to list zellij sessions")?;

    let stdout = String::from_utf8_lossy(&check.stdout);
    // Strip ANSI codes for comparison
    let clean_output: String = stdout
        .chars()
        .filter(|c| !c.is_control() || *c == '\n')
        .collect();
    let session_exists = clean_output
        .lines()
        .any(|line| line.trim().starts_with(session_name));

    // Build the attach/create command
    let zellij_cmd = if session_exists {
        // Attach to existing session
        format!(
            "zellij attach '{}'; zellij list-sessions 2>/dev/null | sed 's/\\x1b\\[[0-9;]*m//g' | grep -q '^{} ' || exec {}",
            session_name, session_name, config.shell
        )
    } else {
        // Create new session with agent wrapper
        format!(
            "zellij -s '{}'; zellij list-sessions 2>/dev/null | sed 's/\\x1b\\[[0-9;]*m//g' | grep -q '^{} ' || exec {}",
            session_name, session_name, config.shell
        )
    };

    // Run interactive exec
    let shell_env = format!("/home/agent/.local/bin/{}-session", config.name);
    let mut cmd = Command::new(runtime.as_str());
    cmd.args(["exec", "-it"]);
    cmd.args(["-e", &format!("SHELL={}", shell_env)]);
    cmd.args(["-e", &format!("AGENT_LAUNCH_CMD={}", config.launch_cmd)]);
    cmd.args([container_name, "bash", "-c", &zellij_cmd]);

    // This is interactive - we need to inherit stdio
    cmd.stdin(Stdio::inherit());
    cmd.stdout(Stdio::inherit());
    cmd.stderr(Stdio::inherit());

    let status = cmd.status().context("Failed to attach to container")?;

    if !status.success() {
        bail!("Failed to attach to session");
    }

    Ok(())
}
