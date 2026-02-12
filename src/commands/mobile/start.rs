use anyhow::{bail, Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use std::thread;
use std::time::Duration;

use crate::config::{load_global_config, merge_configs};
use crate::container::{
    container_status, detect_runtime, ensure_network, ensure_volume, hapi_container_name,
    hapi_volume_name, start_container, ContainerStatus, KLOTHO_NETWORK,
};
use crate::project_config::load_project_config;
use crate::resources;

use super::display_connection_info;

pub fn run(
    runtime_override: Option<&str>,
    cli_no_relay: bool,
    cli_relay: Option<&str>,
    cli_bind: Option<&str>,
) -> Result<()> {
    let runtime = detect_runtime(runtime_override)?;
    let container_name = hapi_container_name();

    // Load configs and resolve effective settings
    let global_config = load_global_config()?;
    let project_config = load_project_config(&std::env::current_dir()?)?;
    let resolved = merge_configs(&global_config, &project_config);

    // Resolve effective settings: CLI > config
    let no_relay = cli_no_relay || resolved.mobile.no_relay;
    let relay_url = cli_relay
        .map(String::from)
        .or_else(|| resolved.mobile.relay.clone());
    let bind_ip = cli_bind
        .map(String::from)
        .or_else(|| resolved.mobile.bind.clone());

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

    // Check for HAPI_PUBLIC_URL env var (backward compatibility)
    let public_url = std::env::var("HAPI_PUBLIC_URL").ok();

    // Determine the effective mode and display URL
    let (effective_mode, display_url, lan_ip) = if public_url.is_some() {
        ("custom", public_url.clone(), None)
    } else if no_relay {
        // Local-only mode
        let lan_ip = if let Some(ip) = &bind_ip {
            ip.clone()
        } else {
            // Auto-detect LAN IP
            let detected = super::detect_lan_ip();
            let all_ips = super::get_all_lan_ips();

            if all_ips.len() > 1 && detected.is_none() {
                // Multiple interfaces, can't auto-detect
                println!("{}", "Multiple network interfaces detected:".yellow());
                for ip in &all_ips {
                    println!("  {} {}", "•".cyan(), ip);
                }
                println!();
                println!("Use {} to specify which IP to use.", "--bind <ip>".cyan());
                bail!("Cannot auto-detect LAN IP with multiple interfaces");
            }

            detected
                .or_else(|| all_ips.first().cloned())
                .context("No LAN IP address found. Use --bind to specify.")?
        };
        let url = format!("http://{}:3006", lan_ip);
        ("local", Some(url), Some(lan_ip))
    } else if let Some(url) = &relay_url {
        ("relay", Some(url.clone()), None)
    } else {
        ("relay", None, None) // Default relay, URL comes from hapi logs
    };

    // Create and start the hapi container
    println!("{}", "Creating hapi mobile hub...".bold());

    let mut cmd = runtime.command();
    cmd.args(["run", "-d"])
        .args(["--name", &container_name])
        .args(["--label=klotho=true"])
        .args(["--network", KLOTHO_NETWORK]);

    // Port binding based on mode
    // For local mode, bind to 0.0.0.0 (all interfaces)
    // For relay mode, bind to localhost only
    let port_binding = if effective_mode == "local" {
        "0.0.0.0:3006:3006".to_string()
    } else {
        "127.0.0.1:3006:3006".to_string()
    };
    cmd.args(["-p", &port_binding]);

    cmd.args(["-e", "HAPI_LISTEN_HOST=0.0.0.0"])
        .args(["-v", &format!("{}:/root/.hapi", hapi_volume_name())]);

    // Pass env vars to hapi based on mode
    if no_relay {
        cmd.args(["-e", "HAPI_NO_RELAY=true"]);
    }

    if let Some(url) = &relay_url {
        cmd.args(["-e", &format!("HAPI_RELAY_URL={}", url)]);
    }

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

    // For relay mode, check if connection succeeded
    if effective_mode == "relay" && display_url.is_none() {
        // Wait a bit more for relay to connect
        thread::sleep(Duration::from_secs(2));

        // Check logs for relay connection error
        let logs_output = runtime
            .command()
            .args(["logs", "--tail", "50", &container_name])
            .output();

        let relay_failed = if let Ok(output) = logs_output {
            let logs = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            let all_logs = format!("{}{}", logs, stderr);

            // Check for common relay failure indicators
            all_logs.contains("relay connection failed")
                || all_logs.contains("could not connect to relay")
                || all_logs.contains("relay unavailable")
                || all_logs.contains("ECONNREFUSED")
                || all_logs.contains("ETIMEDOUT")
                // Also check if no public URL appears after reasonable wait
                || (!all_logs.contains("https://") && !all_logs.contains("relay.hapi.run"))
        } else {
            false
        };

        if relay_failed {
            println!();
            println!("{} Relay unavailable.", "⚠".yellow());
            println!();

            use dialoguer::Confirm;

            let start_local = Confirm::new()
                .with_prompt("Start in local-only mode?")
                .default(false)
                .interact()?;

            if start_local {
                // Stop current container and restart in local mode
                println!();
                println!("{}", "Restarting in local-only mode...".bold());

                // Stop and remove current container
                runtime.command().args(["stop", &container_name]).output()?;
                runtime.command().args(["rm", &container_name]).output()?;

                // Recursive call with local mode
                // Note: This will re-create the container with local settings
                return run(runtime_override, true, None, cli_bind);
            } else {
                // User declined - show helpful message
                println!();
                println!("To start in local-only mode manually, run:");
                println!("  {}", "klotho mobile start --no-relay".cyan());
                println!();
                println!("To use a custom relay, run:");
                println!("  {}", "klotho mobile start --relay <url>".cyan());

                // Clean up the failed container
                runtime.command().args(["stop", &container_name]).output()?;
                runtime.command().args(["rm", &container_name]).output()?;

                bail!("Relay connection failed. See above for alternatives.");
            }
        }
    }

    // Display appropriate URL based on mode
    if let Some(url) = &display_url {
        if effective_mode == "local" {
            println!();
            println!("{} Local mode - accessible on LAN at:", "ℹ".blue());
            super::display_url_with_qr(url)?;
            if let Some(ip) = &lan_ip {
                println!("  {} Bound to {}", "Note:".dimmed(), ip.cyan());
            }
        } else {
            super::display_url_with_qr(url)?;
        }
    } else {
        // Wait for hapi to connect to relay and get URL from logs
        display_connection_info(runtime, &container_name)?;
    }

    Ok(())
}
