use anyhow::{Context, Result};
use std::process::Command;

/// Container runtime (podman or docker)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Runtime {
    Podman,
    Docker,
}

impl Runtime {
    pub fn as_str(&self) -> &'static str {
        match self {
            Runtime::Podman => "podman",
            Runtime::Docker => "docker",
        }
    }

    pub fn command(&self) -> Command {
        Command::new(self.as_str())
    }
}

/// Container status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContainerStatus {
    Running,
    Stopped,
    NotFound,
}

/// Detect container runtime
///
/// Priority:
/// 1. Use runtime_override if provided (--runtime flag)
/// 2. Auto-detect: try podman first, then docker
/// 3. Error if neither is available
pub fn detect_runtime(runtime_override: Option<&str>) -> Result<Runtime> {
    // If override specified, validate and use it
    if let Some(runtime_str) = runtime_override {
        match runtime_str {
            "podman" => {
                ensure_runtime_available(Runtime::Podman)?;
                return Ok(Runtime::Podman);
            }
            "docker" => {
                ensure_runtime_available(Runtime::Docker)?;
                eprintln!(
                    "warning: using Docker (Podman is recommended for better rootless support)"
                );
                return Ok(Runtime::Docker);
            }
            "auto" => {
                // Fall through to auto-detection
            }
            other => {
                anyhow::bail!(
                    "invalid runtime '{}' - must be 'auto', 'podman', or 'docker'",
                    other
                );
            }
        }
    }

    // Auto-detect: prefer podman, fall back to docker
    if is_runtime_available(Runtime::Podman) {
        return Ok(Runtime::Podman);
    }

    if is_runtime_available(Runtime::Docker) {
        eprintln!("warning: using Docker (Podman not found)");
        eprintln!(
            "         for better rootless support, install Podman: \
             https://podman.io/getting-started/installation"
        );
        return Ok(Runtime::Docker);
    }

    anyhow::bail!(
        "no container runtime found\n\
         install Podman (recommended) or Docker to use klotho"
    )
}

/// Check if runtime is available
fn is_runtime_available(runtime: Runtime) -> bool {
    Command::new(runtime.as_str())
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Ensure runtime is available, error if not
fn ensure_runtime_available(runtime: Runtime) -> Result<()> {
    if !is_runtime_available(runtime) {
        anyhow::bail!(
            "{} not found - install it or use --runtime to specify a different runtime",
            runtime.as_str()
        );
    }
    Ok(())
}

/// Check container status
pub fn container_status(runtime: Runtime, container_name: &str) -> Result<ContainerStatus> {
    // Check if running
    let output = runtime
        .command()
        .args(["ps", "--format", "{{.Names}}"])
        .output()
        .context("failed to run container ps")?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.trim() == container_name {
                return Ok(ContainerStatus::Running);
            }
        }
    }

    // Check if exists but stopped
    let output = runtime
        .command()
        .args(["ps", "-a", "--format", "{{.Names}}"])
        .output()
        .context("failed to run container ps -a")?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.trim() == container_name {
                return Ok(ContainerStatus::Stopped);
            }
        }
    }

    Ok(ContainerStatus::NotFound)
}

/// Check if image exists (supports both new and legacy naming)
pub fn image_exists(runtime: Runtime, agent: &str) -> Result<bool> {
    // Check new naming first
    if check_image_exists(runtime, &format!("klotho-{}:latest", agent))? {
        return Ok(true);
    }

    // Check legacy naming
    if check_image_exists(runtime, &format!("agent-session-{}:latest", agent))? {
        return Ok(true);
    }

    Ok(false)
}

/// Check if specific image exists
fn check_image_exists(runtime: Runtime, image_name: &str) -> Result<bool> {
    let output = runtime
        .command()
        .args(["image", "exists", image_name])
        .output()
        .context("failed to check image existence")?;

    Ok(output.status.success())
}

/// Get image name (prefers new naming, falls back to legacy)
pub fn get_image_name(runtime: Runtime, agent: &str) -> Result<String> {
    let new_name = format!("klotho-{}:latest", agent);
    if check_image_exists(runtime, &new_name)? {
        return Ok(new_name);
    }

    let legacy_name = format!("agent-session-{}:latest", agent);
    if check_image_exists(runtime, &legacy_name)? {
        eprintln!(
            "note: using legacy image {}",
            legacy_name
        );
        eprintln!(
            "      rebuild to use new naming: klotho build {}",
            agent
        );
        return Ok(legacy_name);
    }

    // Default to new naming (for new builds)
    Ok(new_name)
}

/// List containers with klotho naming pattern
pub fn list_containers(runtime: Runtime) -> Result<Vec<(String, ContainerStatus)>> {
    let output = runtime
        .command()
        .args(["ps", "-a", "--format", "{{.Names}}|{{.Status}}"])
        .output()
        .context("failed to list containers")?;

    if !output.status.success() {
        anyhow::bail!("failed to list containers");
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut containers = Vec::new();

    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Parse name|status
        if let Some((name, status_str)) = line.split_once('|') {
            // Match both new naming (klotho-session-agent-name) and legacy (agent-name)
            if name.starts_with("klotho-session-") || name.contains("-") {
                let status = if status_str.to_lowercase().contains("up") {
                    ContainerStatus::Running
                } else {
                    ContainerStatus::Stopped
                };
                containers.push((name.to_string(), status));
            }
        }
    }

    Ok(containers)
}

/// Find container by session name
pub fn find_container(runtime: Runtime, session_name: &str) -> Result<Option<String>> {
    let containers = list_containers(runtime)?;

    // Try new naming first: klotho-session-agent-name
    let new_suffix = format!("-{}", session_name);
    for (name, _) in &containers {
        if name.ends_with(&new_suffix) {
            return Ok(Some(name.clone()));
        }
    }

    // Try legacy naming: agent-name
    for (name, _) in &containers {
        if name.ends_with(session_name) {
            return Ok(Some(name.clone()));
        }
    }

    Ok(None)
}

/// Stop container
pub fn stop_container(runtime: Runtime, container_name: &str) -> Result<()> {
    let output = runtime
        .command()
        .args(["stop", container_name])
        .output()
        .context("failed to stop container")?;

    if !output.status.success() {
        // Stopping already-stopped container is OK
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !stderr.to_lowercase().contains("no such container")
            && !stderr.to_lowercase().contains("not running")
        {
            anyhow::bail!("failed to stop container: {}", stderr);
        }
    }

    Ok(())
}

/// Start container
pub fn start_container(runtime: Runtime, container_name: &str) -> Result<()> {
    let output = runtime
        .command()
        .args(["start", container_name])
        .output()
        .context("failed to start container")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("failed to start container: {}", stderr);
    }

    Ok(())
}

/// Remove container
pub fn remove_container(runtime: Runtime, container_name: &str) -> Result<()> {
    let output = runtime
        .command()
        .args(["rm", container_name])
        .output()
        .context("failed to remove container")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("failed to remove container: {}", stderr);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_as_str() {
        assert_eq!(Runtime::Podman.as_str(), "podman");
        assert_eq!(Runtime::Docker.as_str(), "docker");
    }

    #[test]
    fn test_detect_runtime_invalid() {
        let result = detect_runtime(Some("invalid"));
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("invalid runtime"));
    }

    #[test]
    fn test_detect_runtime_auto_fallthrough() {
        // "auto" should not error out immediately
        let result = detect_runtime(Some("auto"));
        // Will succeed if either podman or docker is available
        // Will fail if neither is available
        // Just check it doesn't panic
        let _ = result;
    }
}
