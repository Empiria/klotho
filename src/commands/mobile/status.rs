use anyhow::Result;
use owo_colors::OwoColorize;

use crate::container::{
    detect_runtime, container_status, hapi_container_name,
    ContainerStatus, Runtime, KLOTHO_NETWORK,
};

use super::display_connection_info;

pub fn run(runtime_override: Option<&str>) -> Result<()> {
    let runtime = detect_runtime(runtime_override)?;
    let container_name = hapi_container_name();

    // Check container status
    let status = container_status(runtime, &container_name)?;

    match status {
        ContainerStatus::NotFound => {
            println!("Hapi mobile hub is not configured.");
            println!();
            println!("Run {} to begin.", "klotho mobile start".cyan());
            return Ok(());
        }
        ContainerStatus::Stopped => {
            println!("{}", format!("Hapi mobile hub: {}", "stopped").yellow());
            println!();
            println!("Run {} to start.", "klotho mobile start".cyan());
            return Ok(());
        }
        ContainerStatus::Running => {
            // Continue to show full status
        }
    }

    // Display running status
    println!("{}", format!("Hapi mobile hub: {}", "running").green());
    println!();

    // Display connection info (QR code and URL)
    display_connection_info(runtime, &container_name)?;

    // List connected sessions
    println!("{}", "Connected sessions:".bold());
    println!();

    let connected_sessions = list_connected_sessions(runtime)?;

    if connected_sessions.is_empty() {
        println!("  {}", "No sessions connected yet".dimmed());
        println!();
        println!("  Start a session with {} - it will", "klotho start".cyan());
        println!("  automatically connect to the mobile hub.");
    } else {
        for (session_name, session_status) in connected_sessions {
            let status_text = match session_status {
                ContainerStatus::Running => format!("{}", "running".green()),
                ContainerStatus::Stopped => format!("{}", "stopped".yellow()),
                ContainerStatus::NotFound => format!("{}", "unknown".dimmed()),
            };
            println!("  {} {} ({})", "•".cyan(), session_name, status_text);
        }
    }

    println!();

    Ok(())
}

/// List containers connected to the klotho network
fn list_connected_sessions(runtime: Runtime) -> Result<Vec<(String, ContainerStatus)>> {
    // Inspect the klotho network to get list of connected containers
    let output = runtime
        .command()
        .args(["network", "inspect", KLOTHO_NETWORK, "--format", "{{range .Containers}}{{.Name}} {{end}}"])
        .output();

    let mut sessions = Vec::new();

    if let Ok(output) = output {
        if output.status.success() {
            let containers = String::from_utf8_lossy(&output.stdout);

            // Parse container names
            for container_name in containers.split_whitespace() {
                // Filter to only show session containers (not hapi itself)
                if container_name.starts_with("klotho-session-") {
                    // Extract session name from container name
                    // Format: klotho-session-{agent}-{name}
                    if let Some(session_part) = container_name.strip_prefix("klotho-session-") {
                        // Find the last hyphen to split agent-name
                        if let Some(last_hyphen) = session_part.rfind('-') {
                            let session_name = &session_part[last_hyphen + 1..];

                            // Get the status of this container
                            let status = container_status(runtime, container_name).unwrap_or(ContainerStatus::NotFound);

                            sessions.push((session_name.to_string(), status));
                        }
                    }
                }
            }
        }
    }

    Ok(sessions)
}
