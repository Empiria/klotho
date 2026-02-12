use anyhow::{bail, Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use std::thread;
use std::time::Duration;

use crate::container::{
    container_status, detect_runtime, ensure_network, ensure_volume, hapi_container_name,
    hapi_volume_name, start_container, ContainerStatus, KLOTHO_NETWORK,
};
use crate::resources;

use super::display_connection_info;

pub fn run(runtime_override: Option<&str>) -> Result<()> {
    let runtime = detect_runtime(runtime_override)?;
    let container_name = hapi_container_name();

    // Check container status
    let status = container_status(runtime, &container_name)?;

    match status {
        ContainerStatus::Running => {
            println!("{} Hapi mobile hub already running", "✓".green().bold());
            display_connection_info(runtime, &container_name)?;
            return Ok(());
        }
        ContainerStatus::Stopped => {
            println!("{}", "Starting hapi mobile hub...".bold());
            start_container(runtime, &container_name)?;
            thread::sleep(Duration::from_secs(2));
            display_connection_info(runtime, &container_name)?;
            return Ok(());
        }
        ContainerStatus::NotFound => {
            // Continue to create new container
        }
    }

    // Build hapi image if needed
    let image_name = "klotho-hapi:latest";
    let image_check = runtime
        .command()
        .args(["image", "exists", image_name])
        .output()
        .context("failed to check if hapi image exists")?;

    if !image_check.status.success() {
        println!("{}", "Building hapi image...".bold());

        let build_dir = resources::extract_hapi_build_context()?;
        let containerfile_path = build_dir.join("Containerfile.hapi");

        let spinner = ProgressBar::new_spinner();
        spinner.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap(),
        );
        spinner.set_message("Building klotho-hapi image...");
        spinner.enable_steady_tick(Duration::from_millis(100));

        let build_output = runtime
            .command()
            .arg("build")
            .arg("-t")
            .arg(image_name)
            .arg("-f")
            .arg(&containerfile_path)
            .arg(&build_dir)
            .output()
            .context("failed to build hapi image")?;

        spinner.finish_and_clear();

        if !build_output.status.success() {
            let stderr = String::from_utf8_lossy(&build_output.stderr);
            bail!("failed to build hapi image:\n{}", stderr);
        }

        println!("{} Built hapi image", "✓".green().bold());
    }

    // Ensure network exists
    ensure_network(runtime, KLOTHO_NETWORK)?;

    // Ensure volume exists
    ensure_volume(runtime, &hapi_volume_name())?;

    // Check for HAPI_PUBLIC_URL env var
    let public_url = std::env::var("HAPI_PUBLIC_URL").ok();

    // Create and start the hapi container
    println!("{}", "Creating hapi mobile hub...".bold());

    let mut cmd = runtime.command();
    cmd.args(["run", "-d"])
        .args(["--name", &container_name])
        .args(["--label=klotho=true"])
        .args(["--network", KLOTHO_NETWORK])
        .args(["-p", "127.0.0.1:3006:3006"])
        .args(["-e", "HAPI_LISTEN_HOST=0.0.0.0"])
        .args(["-v", &format!("{}:/root/.hapi", hapi_volume_name())]);

    // Add HAPI_PUBLIC_URL if set
    if let Some(url) = &public_url {
        cmd.args(["-e", &format!("HAPI_PUBLIC_URL={}", url)]);
    }

    cmd.arg(image_name);

    let create_output = cmd.output().context("failed to create hapi container")?;

    if !create_output.status.success() {
        let stderr = String::from_utf8_lossy(&create_output.stderr);
        bail!("failed to create hapi container:\n{}", stderr);
    }

    println!("{} Hapi mobile hub started", "✓".green().bold());

    // Wait for hapi to initialize
    thread::sleep(Duration::from_secs(3));

    // Display connection info
    display_connection_info(runtime, &container_name)?;

    Ok(())
}
