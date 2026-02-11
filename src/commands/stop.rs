use anyhow::{bail, Result};
use owo_colors::OwoColorize;

use crate::container::{
    container_status, detect_runtime, find_container, hapi_container_name, stop_container,
    ContainerStatus,
};
use crate::commands::mobile;

pub fn run(name: String, runtime_override: Option<&str>) -> Result<()> {
    let runtime = detect_runtime(runtime_override)?;

    // Find container by session name
    let container_name = match find_container(runtime, &name)? {
        Some(name) => name,
        None => bail!("Session '{}' not found", name),
    };

    // Deregister from hapi before stopping (best-effort)
    let hapi_name = hapi_container_name();
    if let Ok(ContainerStatus::Running) = container_status(runtime, &hapi_name) {
        if let Err(e) = mobile::deregister_session_from_hapi(runtime, &container_name) {
            eprintln!("  {} Failed to deregister from mobile hub: {}", "⚠".yellow(), e);
        }
    }

    // Stop container (idempotent)
    stop_container(runtime, &container_name)?;

    println!("Stopped: {}", name);
    Ok(())
}
