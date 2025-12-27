//! Output formatting for CLI

use console::style;
use serde::Serialize;

/// Print a success message
pub fn success(message: &str) {
    println!("{} {}", style("✓").green().bold(), message);
}

/// Print an info message
pub fn info(message: &str) {
    println!("{} {}", style("ℹ").blue().bold(), message);
}

/// Print a warning message
pub fn warning(message: &str) {
    println!("{} {}", style("⚠").yellow().bold(), message);
}

/// Print an error message
pub fn error(message: &str) {
    eprintln!("{} {}", style("✗").red().bold(), message);
}

/// Print a step in a process
pub fn step(number: usize, total: usize, message: &str) {
    println!(
        "{} {}",
        style(format!("[{}/{}]", number, total)).dim(),
        message
    );
}

/// Print JSON output
pub fn json<T: Serialize>(data: &T) -> Result<(), serde_json::Error> {
    println!("{}", serde_json::to_string_pretty(data)?);
    Ok(())
}

/// Print a table header
pub fn table_header(columns: &[&str]) {
    let header: Vec<String> = columns.iter().map(|c| style(*c).bold().to_string()).collect();
    println!("{}", header.join("  "));
}

/// Print a divider line
pub fn divider() {
    println!("{}", style("─".repeat(60)).dim());
}

/// Format a package name with version
pub fn package_version(name: &str, version: &str) -> String {
    format!("{}@{}", style(name).cyan(), style(version).green())
}

/// Format a duration in human-readable form
pub fn format_duration(millis: u128) -> String {
    if millis < 1000 {
        format!("{}ms", millis)
    } else if millis < 60000 {
        format!("{:.2}s", millis as f64 / 1000.0)
    } else {
        let seconds = millis / 1000;
        let minutes = seconds / 60;
        let remaining_seconds = seconds % 60;
        format!("{}m {}s", minutes, remaining_seconds)
    }
}

/// Format a byte size in human-readable form
pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes < KB {
        format!("{} B", bytes)
    } else if bytes < MB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else if bytes < GB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    }
}

/// Create a progress spinner
pub fn spinner(message: &str) -> indicatif::ProgressBar {
    let spinner = indicatif::ProgressBar::new_spinner();
    spinner.set_style(
        indicatif::ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    spinner.set_message(message.to_string());
    spinner.enable_steady_tick(std::time::Duration::from_millis(100));
    spinner
}

/// Create a progress bar for downloads
pub fn download_progress(total: u64) -> indicatif::ProgressBar {
    let bar = indicatif::ProgressBar::new(total);
    bar.set_style(
        indicatif::ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} packages ({eta})")
            .unwrap()
            .progress_chars("█▓▒░"),
    );
    bar
}

/// Create a multi-progress for concurrent downloads
pub fn multi_progress() -> indicatif::MultiProgress {
    indicatif::MultiProgress::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(500), "500ms");
        assert_eq!(format_duration(1500), "1.50s");
        assert_eq!(format_duration(65000), "1m 5s");
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(500), "500 B");
        assert_eq!(format_bytes(1500), "1.5 KB");
        assert_eq!(format_bytes(1500000), "1.4 MB");
    }
}
