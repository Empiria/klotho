use anyhow::{bail, Result};
use std::io::{self, Write};
use crate::container::{
    container_status, detect_runtime, find_container, remove_container, ContainerStatus,
};

pub fn run(name: String, force: bool, runtime_override: Option<&str>) -> Result<()> {
    let runtime = detect_runtime(runtime_override)?;

    // Find container by session name
    let container_name = match find_container(runtime, &name)? {
        Some(name) => name,
        None => bail!("Session '{}' not found", name),
    };

    // Check if running
    let status = container_status(runtime, &container_name)?;
    if status == ContainerStatus::Running {
        bail!(
            "Cannot remove running session '{}'\nStop it first: klotho stop {}",
            name,
            name
        );
    }

    // Confirm unless --force
    if !force {
        print!("Remove session '{}'? [y/N] ", name);
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let answer = input.trim().to_lowercase();

        if answer != "y" && answer != "yes" {
            println!("Cancelled.");
            return Ok(());
        }
    }

    // Remove container
    remove_container(runtime, &container_name)?;

    println!("Removed: {}", name);
    Ok(())
}
