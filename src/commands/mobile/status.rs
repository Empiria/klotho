use anyhow::Result;
use owo_colors::OwoColorize;

use crate::container::{
    container_status, detect_runtime, hapi_container_name, ContainerStatus, Runtime, KLOTHO_NETWORK,
};

use super::display_connection_info;

/// Detect current mode from container environment variables
fn detect_container_mode(runtime: Runtime, container_name: &str) -> String {
    // Check container env vars to determine mode
    let output = runtime
        .command()
        .args([
            "inspect",
            "--format",
            "{{range .Config.Env}}{{println .}}{{end}}",
            container_name,
        ])
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            let envs = String::from_utf8_lossy(&output.stdout);

            // Check for HAPI_PUBLIC_URL (custom mode)
            if envs.contains("HAPI_PUBLIC_URL=") {
                return "custom".to_string();
            }

            // Check for HAPI_NO_RELAY (local mode)
            if envs.contains("HAPI_NO_RELAY=true") {
                // Try to get the bound IP from port mapping or detect LAN IP
                if let Some(ip) = get_container_lan_ip(runtime, container_name) {
                    return format!("local ({})", ip);
                }
                return "local".to_string();
            }

            // Check for custom relay
            for line in envs.lines() {
                if line.starts_with("HAPI_RELAY_URL=") {
                    let url = line.trim_start_matches("HAPI_RELAY_URL=");
                    return format!("relay ({})", url);
                }
            }
        }
    }

    "relay".to_string() // Default
}

/// Get LAN IP for container in local mode
fn get_container_lan_ip(runtime: Runtime, container_name: &str) -> Option<String> {
    // Check port binding to see if bound to specific IP
    let output = runtime
        .command()
        .args([
            "inspect",
            "--format",
            "{{range $p, $conf := .NetworkSettings.Ports}}{{range $conf}}{{.HostIp}}{{end}}{{end}}",
            container_name,
        ])
        .output()
        .ok()?;

    if output.status.success() {
        let ip = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !ip.is_empty() && ip != "127.0.0.1" && ip != "0.0.0.0" {
            return Some(ip);
        }
    }

    // Fallback: detect LAN IP
    super::detect_lan_ip()
}

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

    // Detect and display mode
    let mode = detect_container_mode(runtime, &container_name);

    // Display running status with mode
    println!("{}", format!("Hapi mobile hub: {}", "running").green());
    println!("{}", format!("Mode: {}", mode).dimmed());
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
    // Use `ps --filter network=` to find containers on the klotho network.
    // podman network inspect's Go template format differs from Docker and
    // doesn't expose a .Containers field, so we query running containers instead.
    let output = runtime
        .command()
        .args([
            "ps",
            "-a",
            "--filter",
            &format!("network={}", KLOTHO_NETWORK),
            "--format",
            "{{.Names}}",
        ])
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
                            let status = container_status(runtime, container_name)
                                .unwrap_or(ContainerStatus::NotFound);

                            sessions.push((session_name.to_string(), status));
                        }
                    }
                }
            }
        }
    }

    Ok(sessions)
}
