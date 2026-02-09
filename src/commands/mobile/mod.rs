pub mod start;
pub mod stop;
pub mod status;
pub mod revoke;

use anyhow::Result;
use owo_colors::OwoColorize;
use qrcode::QrCode;
use qrcode::render::unicode;

use crate::container::Runtime;

/// Display connection info with QR code
pub fn display_connection_info(runtime: Runtime, container_name: &str) -> Result<()> {
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
