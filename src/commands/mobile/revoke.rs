use anyhow::{bail, Result};
use owo_colors::OwoColorize;
use std::process::Stdio;

use crate::container::{container_status, detect_runtime, hapi_container_name, ContainerStatus};

pub fn run(runtime_override: Option<&str>) -> Result<()> {
    let runtime = detect_runtime(runtime_override)?;
    let container_name = hapi_container_name();

    // Check if hapi container is running
    let status = container_status(runtime, &container_name)?;

    match status {
        ContainerStatus::NotFound => {
            bail!(
                "Hapi mobile hub is not configured.\n\
                 Run {} to begin.",
                "klotho mobile start".cyan()
            );
        }
        ContainerStatus::Stopped => {
            bail!(
                "Hapi mobile hub is not running.\n\
                 Start it with {}",
                "klotho mobile start".cyan()
            );
        }
        ContainerStatus::Running => {
            // Continue to revoke
        }
    }

    // Execute hapi hub revoke command inside container
    println!("{}", "Revoking device access...".bold());
    println!();

    let mut cmd = runtime.command();
    cmd.args(["exec", "-it", &container_name, "hapi", "hub", "revoke"]);

    // This is interactive - hapi may prompt user to select device
    cmd.stdin(Stdio::inherit());
    cmd.stdout(Stdio::inherit());
    cmd.stderr(Stdio::inherit());

    let status = cmd.status()?;

    if !status.success() {
        bail!("Failed to revoke device access");
    }

    println!();
    println!("{} Device revoked successfully", "✓".green().bold());

    Ok(())
}
