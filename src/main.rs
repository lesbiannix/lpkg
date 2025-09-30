mod tui;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tui::disk_manager::DiskManager::run_tui()?;
    Ok(())
}
