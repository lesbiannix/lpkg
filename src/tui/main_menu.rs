use crate::tui::disk_manager::DiskManager;
use crossterm::event::{self, Event, KeyCode};
use std::error::Error;
use std::io::Stdout;
use tui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
};

pub fn show_main_menu() -> Result<(), Box<dyn Error>> {
    let mut stdout = std::io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        terminal.draw(|f| {
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints([Constraint::Length(3), Constraint::Length(3)].as_ref())
                .split(size);

            let menu = Paragraph::new("1) Disk Manager\n0) Exit")
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(menu, chunks[0]);

            let status = Paragraph::new("Use number keys to select an option")
                .style(Style::default().fg(Color::Yellow))
                .block(Block::default().borders(Borders::ALL).title("Status"));
            f.render_widget(status, chunks[1]);
        })?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('1') => DiskManager::show_disk_manager(&mut terminal)?,
                    KeyCode::Char('0') => break,
                    _ => {}
                }
            }
        }
    }

    Ok(())
}
