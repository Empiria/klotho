use anyhow::{bail, Result};
use crate::container::{detect_runtime, find_container, stop_container};

pub fn run(name: String, runtime_override: Option<&str>) -> Result<()> {
    let runtime = detect_runtime(runtime_override)?;

    // Find container by session name
    let container_name = match find_container(runtime, &name)? {
        Some(name) => name,
        None => bail!("Session '{}' not found", name),
    };

    // Stop container (idempotent)
    stop_container(runtime, &container_name)?;

    println!("Stopped: {}", name);
    Ok(())
}
