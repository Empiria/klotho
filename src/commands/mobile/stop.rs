use anyhow::Result;
use owo_colors::OwoColorize;

use crate::container::{
    container_status, detect_runtime, hapi_container_name, stop_container, ContainerStatus,
};

pub fn run(runtime_override: Option<&str>) -> Result<()> {
    let runtime = detect_runtime(runtime_override)?;
    let container_name = hapi_container_name();

    // Check status
    let status = container_status(runtime, &container_name)?;

    match status {
        ContainerStatus::Running => {
            stop_container(runtime, &container_name)?;
            println!("{} Stopped hapi mobile hub", "✓".green().bold());
        }
        ContainerStatus::Stopped => {
            println!("Hapi mobile hub is already stopped");
        }
        ContainerStatus::NotFound => {
            println!("Hapi mobile hub is not running");
            println!();
            println!("Run {} to start it.", "klotho mobile start".cyan());
        }
    }

    Ok(())
}
