use anyhow::{bail, Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use qrcode::QrCode;
use qrcode::render::unicode;
use std::thread;
use std::time::Duration;

use crate::container::{
    container_status, detect_runtime, ensure_network, ensure_volume,
    hapi_container_name, hapi_volume_name, start_container, ContainerStatus, Runtime,
    KLOTHO_NETWORK,
};
use crate::resources;

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
        .args(["-v", &format!("{}:/root/.hapi", hapi_volume_name())]);

    // Add HAPI_PUBLIC_URL if set
    if let Some(url) = &public_url {
        cmd.args(["-e", &format!("HAPI_PUBLIC_URL={}", url)]);
    }

    cmd.arg(image_name);

    let create_output = cmd
        .output()
        .context("failed to create hapi container")?;

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

fn display_connection_info(runtime: Runtime, container_name: &str) -> Result<()> {
    // Check for override first
    if let Some(url) = std::env::var("HAPI_PUBLIC_URL").ok() {
        display_url_with_qr(&url)?;
        return Ok(());
    }

    // Strategy 1: Try to read settings.json from container
    let settings_output = runtime
        .command()
        .args(["exec", container_name, "cat", "/root/.hapi/settings.json"])
        .output();

    if let Ok(output) = settings_output {
        if output.status.success() {
            let settings = String::from_utf8_lossy(&output.stdout);
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&settings) {
                // Try various possible field names
                if let Some(url) = json.get("url")
                    .or_else(|| json.get("hubUrl"))
                    .or_else(|| json.get("relay_url"))
                    .or_else(|| json.get("relayUrl"))
                    .and_then(|v| v.as_str())
                {
                    display_url_with_qr(url)?;
                    return Ok(());
                }
            }
        }
    }

    // Strategy 2: Try hapi CLI to get URL
    let cli_output = runtime
        .command()
        .args(["exec", container_name, "hapi", "hub", "url"])
        .output();

    if let Ok(output) = cli_output {
        if output.status.success() {
            let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !url.is_empty() && (url.starts_with("http://") || url.starts_with("https://")) {
                display_url_with_qr(&url)?;
                return Ok(());
            }
        }
    }

    // Strategy 3: Parse container logs for URL
    let logs_output = runtime
        .command()
        .args(["logs", container_name])
        .output();

    if let Ok(output) = logs_output {
        let logs = String::from_utf8_lossy(&output.stdout);
        // Look for URL patterns in logs
        for line in logs.lines() {
            if let Some(url_start) = line.find("http://").or_else(|| line.find("https://")) {
                // Extract URL from the line
                let url_part = &line[url_start..];
                if let Some(url_end) = url_part.find(char::is_whitespace) {
                    let url = &url_part[..url_end];
                    display_url_with_qr(url)?;
                    return Ok(());
                } else {
                    // URL goes to end of line
                    display_url_with_qr(url_part.trim())?;
                    return Ok(());
                }
            }
        }
    }

    // Strategy 4: Fallback - warn user
    println!();
    println!("  {} Hapi is initializing...", "⚠".yellow());
    println!();
    println!("  The connection URL is not yet available.");
    println!("  Run {} in a few seconds to see it.", "klotho mobile status".cyan());
    println!();

    Ok(())
}

fn display_url_with_qr(url: &str) -> Result<()> {
    let code = QrCode::new(url)?;
    let image = code
        .render::<unicode::Dense1x2>()
        .dark_color(unicode::Dense1x2::Light)
        .light_color(unicode::Dense1x2::Dark)
        .build();

    println!();
    println!("{}", image);
    println!();
    println!("  {} {}", "URL:".bold(), url.cyan());
    println!();
    println!("  Scan the QR code or open the URL on your phone");
    println!("  to connect to your klotho sessions.");
    println!();

    Ok(())
}
