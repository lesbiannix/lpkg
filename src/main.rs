mod downloader;
mod html;
mod md5_utils;
mod mirrors;
mod version_check;
mod wget_list;

#[cfg(feature = "tui")]
mod tui;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "tui")]
    {
        tui::tui_menu()?;
        return Ok(());
    }

    #[cfg(not(feature = "tui"))]
    {
        println!("TUI feature not enabled. Compile with `--features tui` to run TUI.");
        Ok(())
    }
}
