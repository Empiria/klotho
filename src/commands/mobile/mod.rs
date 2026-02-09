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
    if let Ok(url) = std::env::var("HAPI_PUBLIC_URL") {
        display_url_with_qr(&url)?;
        return Ok(());
    }

    // Parse logs — hapi prints the app URL and relay URL during startup
    let logs_output = runtime
        .command()
        .args(["logs", container_name])
        .output();

    if let Ok(output) = logs_output {
        let logs = String::from_utf8_lossy(&output.stdout);

        // Strategy 1: Find the app.hapi.run URL (the intended mobile pairing URL)
        // Hapi prints: "  https://app.hapi.run/?hub=...&token=..."
        for line in logs.lines() {
            if let Some(start) = line.find("https://app.hapi.run/") {
                let url = line[start..].trim();
                display_url_with_qr(url)?;
                return Ok(());
            }
        }

        // Strategy 2: Find relay URL from "[Web] Public:" line and construct app URL
        // Hapi prints: "[Web] Public: https://XXXX.relay.hapi.run"
        let mut relay_url = None;
        for line in logs.lines() {
            if line.contains("[Web] Public:") {
                if let Some(start) = line.find("https://") {
                    relay_url = Some(line[start..].trim().to_string());
                    break;
                }
            }
        }

        if let Some(relay) = relay_url {
            // Try to get token from settings.json to construct full app URL
            let token = get_cli_token(runtime, container_name);
            if let Some(token) = token {
                let encoded_relay = relay.replace("://", "%3A%2F%2F").replace("/", "%2F");
                let app_url = format!(
                    "https://app.hapi.run/?hub={}&token={}",
                    encoded_relay, token
                );
                display_url_with_qr(&app_url)?;
            } else {
                display_url_with_qr(&relay)?;
            }
            return Ok(());
        }

        // Strategy 3: Find any non-localhost https URL in logs
        for line in logs.lines() {
            if let Some(start) = line.find("https://") {
                let url_part = &line[start..];
                let url = match url_part.find(char::is_whitespace) {
                    Some(end) => &url_part[..end],
                    None => url_part.trim(),
                };
                if !url.contains("localhost") {
                    display_url_with_qr(url)?;
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

fn get_cli_token(runtime: Runtime, container_name: &str) -> Option<String> {
    let output = runtime
        .command()
        .args(["exec", container_name, "cat", "/root/.hapi/settings.json"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let settings = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&settings).ok()?;
    json.get("cliApiToken")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
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
