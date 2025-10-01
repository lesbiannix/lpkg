use std::io::Stdout;
use tui::{Terminal, backend::CrosstermBackend};

pub struct Settings {
    pub theme: Theme,
}

pub struct Theme;

impl Theme {
    pub fn primary_color(&self) -> tui::style::Color {
        tui::style::Color::Cyan
    }
    pub fn secondary_color(&self) -> tui::style::Color {
        tui::style::Color::White
    }
}

impl Settings {
    pub fn show_settings(
        _terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Render settings UI here
        Ok(())
    }
}
