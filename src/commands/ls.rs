use anyhow::Result;
use owo_colors::OwoColorize;
use crate::container::{detect_runtime, list_containers, ContainerStatus};

pub fn run(runtime_override: Option<&str>) -> Result<()> {
    let runtime = detect_runtime(runtime_override)?;

    // List containers
    let containers = list_containers(runtime)?;

    if containers.is_empty() {
        println!("No sessions found.");
        return Ok(());
    }

    // Print table header
    println!("{:<30} {:<20} {:<10}", "NAME", "AGENT", "STATUS");
    println!("{}", "-".repeat(60));

    for (container_name, status) in containers {
        // Extract session name and agent from container name
        let (session_name, agent) = extract_session_info(&container_name);

        // Colorize status
        let status_str = match status {
            ContainerStatus::Running => "running".green().to_string(),
            ContainerStatus::Stopped => "stopped".red().to_string(),
            ContainerStatus::NotFound => "unknown".yellow().to_string(),
        };

        println!("{:<30} {:<20} {:<10}", session_name, agent, status_str);
    }

    Ok(())
}

fn extract_session_info(container_name: &str) -> (String, String) {
    // New naming: klotho-session-{agent}-{name}
    if let Some(rest) = container_name.strip_prefix("klotho-session-") {
        if let Some(pos) = rest.find('-') {
            let agent = &rest[..pos];
            let name = &rest[pos + 1..];
            return (name.to_string(), agent.to_string());
        }
        return (rest.to_string(), rest.to_string());
    }

    // Non-session klotho containers (e.g. klotho-hapi sidecar)
    if let Some(rest) = container_name.strip_prefix("klotho-") {
        return (rest.to_string(), rest.to_string());
    }

    // Unknown format
    (container_name.to_string(), "unknown".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_naming_simple() {
        let (name, agent) = extract_session_info("klotho-session-claude-default");
        assert_eq!(agent, "claude");
        assert_eq!(name, "default");
    }

    #[test]
    fn test_new_naming_hyphenated_session_name() {
        let (name, agent) = extract_session_info("klotho-session-opencode-my-project");
        assert_eq!(agent, "opencode");
        assert_eq!(name, "my-project");
    }

    #[test]
    fn test_new_naming_no_session_name() {
        let (name, agent) = extract_session_info("klotho-session-claude");
        assert_eq!(agent, "claude");
        assert_eq!(name, "claude");
    }

    #[test]
    fn test_hapi_sidecar() {
        let (name, agent) = extract_session_info("klotho-hapi");
        assert_eq!(agent, "hapi");
        assert_eq!(name, "hapi");
    }

    #[test]
    fn test_unknown_format() {
        let (name, agent) = extract_session_info("some-unknown-container");
        assert_eq!(name, "some-unknown-container");
        assert_eq!(agent, "unknown");
    }
}
