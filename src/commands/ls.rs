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
    // Try new naming: klotho-<agent>-<name>
    // Example: klotho-opencode-infinite-worlds -> agent="opencode-infinite", name="worlds"
    if let Some(rest) = container_name.strip_prefix("klotho-") {
        // Find the last hyphen to split agent and name
        if let Some(last_hyphen) = rest.rfind('-') {
            let agent = &rest[..last_hyphen];
            let name = &rest[last_hyphen + 1..];
            return (name.to_string(), agent.to_string());
        }
    }

    // Try legacy naming: <agent>-<name>
    // Example: agent-default -> agent="agent", name="default"
    if let Some(last_hyphen) = container_name.rfind('-') {
        let agent = &container_name[..last_hyphen];
        let name = &container_name[last_hyphen + 1..];
        return (name.to_string(), agent.to_string());
    }

    // Fallback: couldn't parse
    (container_name.to_string(), "unknown".to_string())
}
