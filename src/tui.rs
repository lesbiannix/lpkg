use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use std::{error::Error, io};
use tui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem},
};

pub fn tui_menu() -> Result<(), Box<dyn Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let menu_items = vec!["Start downloader", "Exit"];
    let mut selected = 0;

    loop {
        terminal.draw(|f| {
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(5)
                .constraints([Constraint::Length(menu_items.len() as u16)].as_ref())
                .split(size);

            let items: Vec<ListItem> = menu_items
                .iter()
                .enumerate()
                .map(|(i, m)| {
                    let style = if i == selected {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    };
                    ListItem::new(*m).style(style)
                })
                .collect();

            let list = List::new(items).block(
                Block::default()
                    .title("LFS Downloader")
                    .borders(Borders::ALL),
            );
            f.render_widget(list, chunks[0]);
        })?;

        // Handle input
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Up => {
                    if selected > 0 {
                        selected -= 1;
                    }
                }
                KeyCode::Down => {
                    if selected < menu_items.len() - 1 {
                        selected += 1;
                    }
                }
                KeyCode::Enter => {
                    match selected {
                        0 => {
                            // Start downloader
                            disable_raw_mode()?;
                            execute!(
                                terminal.backend_mut(),
                                LeaveAlternateScreen,
                                DisableMouseCapture
                            )?;
                            super::start_downloader()?; // call your downloader function
                            return Ok(());
                        }
                        1 => {
                            break; // Exit
                        }
                        _ => {}
                    }
                }
                KeyCode::Esc => break,
                _ => {}
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    Ok(())
}
