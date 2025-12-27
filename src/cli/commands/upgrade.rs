//! velocity upgrade - Self-update Velocity

use clap::Args;

use crate::cli::output;
use crate::core::{VelocityResult, VelocityError};

const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const RELEASES_URL: &str = "https://api.github.com/repos/nicholaspalmer/velocity/releases/latest";

#[derive(Args)]
pub struct UpgradeArgs {
    /// Check for updates without installing
    #[arg(long)]
    pub check: bool,

    /// Force upgrade even if on latest version
    #[arg(short, long)]
    pub force: bool,
}

pub async fn execute(args: UpgradeArgs, json_output: bool) -> VelocityResult<()> {
    if !json_output {
        output::info(&format!("Current version: v{}", CURRENT_VERSION));
    }

    let progress = if !json_output {
        Some(output::spinner("Checking for updates..."))
    } else {
        None
    };

    // Check for latest version
    let latest_version = check_latest_version().await;

    if let Some(pb) = progress {
        pb.finish_and_clear();
    }

    match latest_version {
        Ok(latest) => {
            let is_newer = is_version_newer(&latest, CURRENT_VERSION);

            if json_output {
                output::json(&serde_json::json!({
                    "current_version": CURRENT_VERSION,
                    "latest_version": latest,
                    "update_available": is_newer,
                    "check_only": args.check
                }))?;
            } else if is_newer {
                output::info(&format!("New version available: v{}", latest));

                if args.check {
                    println!();
                    output::info("Run 'velocity upgrade' to update");
                } else {
                    println!();
                    perform_upgrade(&latest, json_output).await?;
                }
            } else {
                output::success("You're already on the latest version!");

                if args.force && !args.check {
                    println!();
                    output::info("Force reinstalling...");
                    perform_upgrade(&latest, json_output).await?;
                }
            }
        }
        Err(e) => {
            if json_output {
                output::json(&serde_json::json!({
                    "error": true,
                    "message": e.to_string(),
                    "current_version": CURRENT_VERSION
                }))?;
            } else {
                output::warning(&format!("Could not check for updates: {}", e));
                println!();
                output::info("You can manually download the latest version from:");
                println!("  https://github.com/nicholaspalmer/velocity/releases");
            }
        }
    }

    Ok(())
}

async fn check_latest_version() -> VelocityResult<String> {
    let client = reqwest::Client::new();

    let response = client
        .get(RELEASES_URL)
        .header("User-Agent", format!("velocity/{}", CURRENT_VERSION))
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| VelocityError::Network(e.to_string()))?;

    if !response.status().is_success() {
        return Err(VelocityError::Network(format!(
            "GitHub API returned status {}",
            response.status()
        )));
    }

    let release: serde_json::Value = response.json().await
        .map_err(|e| VelocityError::Network(e.to_string()))?;

    let tag_name = release
        .get("tag_name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| VelocityError::Network("Invalid release format".to_string()))?;

    // Remove 'v' prefix if present
    let version = tag_name.trim_start_matches('v').to_string();

    Ok(version)
}

fn is_version_newer(latest: &str, current: &str) -> bool {
    let parse_version = |v: &str| -> (u32, u32, u32) {
        let parts: Vec<&str> = v.split('.').collect();
        (
            parts.first().and_then(|s| s.parse().ok()).unwrap_or(0),
            parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0),
            parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0),
        )
    };

    let latest_v = parse_version(latest);
    let current_v = parse_version(current);

    latest_v > current_v
}

async fn perform_upgrade(version: &str, json_output: bool) -> VelocityResult<()> {
    let progress = if !json_output {
        Some(output::spinner("Downloading new version..."))
    } else {
        None
    };

    // Determine platform
    let (os, ext) = if cfg!(target_os = "windows") {
        ("windows", "zip")
    } else if cfg!(target_os = "macos") {
        ("macos", "tar.gz")
    } else {
        ("linux", "tar.gz")
    };

    let arch = if cfg!(target_arch = "x86_64") {
        "x86_64"
    } else if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else {
        "x86_64" // fallback
    };

    let download_url = format!(
        "https://github.com/nicholaspalmer/velocity/releases/download/v{}/velocity-{}-{}.{}",
        version, os, arch, ext
    );

    if let Some(ref pb) = progress {
        pb.set_message(format!("Downloading from {}", download_url));
    }

    // In a real implementation, we would:
    // 1. Download the binary
    // 2. Verify checksum
    // 3. Replace the current executable
    // 4. Handle permissions

    // For now, provide manual instructions
    if let Some(pb) = progress {
        pb.finish_and_clear();
    }

    if !json_output {
        output::info("Automatic updates are not yet implemented.");
        println!();
        output::info("To upgrade manually:");
        println!("  1. Download from: {}", download_url);
        println!("  2. Replace the current velocity binary");
        println!("  3. Run 'velocity --version' to verify");
    }

    Ok(())
}
