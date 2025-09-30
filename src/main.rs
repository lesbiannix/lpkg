// src/main.rs - Initialize logging
use tracing::{error, info};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

mod tui;
mod wget_list;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    init_logging()?;

    info!("Starting lpkg package manager");
    info!("Version: 0.1.0");

    // Run the TUI
    if let Err(e) = tui::tui_menu() {
        error!("TUI error: {}", e);
        eprintln!("Error: {}", e);
        return Err(e);
    }

    info!("lpkg exiting normally");
    Ok(())
}

fn init_logging() -> Result<(), Box<dyn std::error::Error>> {
    // Create log directory if it doesn't exist
    std::fs::create_dir_all("logs")?;

    // File appender - rotates daily
    let file_appender = RollingFileAppender::new(Rotation::DAILY, "logs", "lpkg.log");

    // Console layer - only shows info and above
    let console_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(false)
        .with_file(true)
        .with_line_number(true)
        .with_filter(EnvFilter::new("info"));

    // File layer - shows debug and above
    let file_layer = fmt::layer()
        .with_writer(file_appender)
        .with_ansi(false)
        .with_target(true)
        .with_file(true)
        .with_line_number(true)
        .with_filter(EnvFilter::new("debug"));

    // Build the subscriber
    tracing_subscriber::registry()
        .with(console_layer)
        .with(file_layer)
        .init();

    Ok(())
}
