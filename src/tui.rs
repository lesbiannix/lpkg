use crate::{downloader, wget_list};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use std::{error::Error, io, time::Duration};
use tui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph},
};

pub fn tui_menu() -> Result<(), Box<dyn Error>> {
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
                .margin(2)
                .constraints(
                    [
                        Constraint::Length(menu_items.len() as u16),
                        Constraint::Min(1),
                    ]
                    .as_ref(),
                )
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

            let status = Paragraph::new("Use ↑↓ to navigate, Enter to select.")
                .block(Block::default().borders(Borders::ALL).title("Status"));
            f.render_widget(status, chunks[1]);
        })?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Up => {
                        if selected > 0 {
                            selected -= 1
                        }
                    }
                    KeyCode::Down => {
                        if selected < menu_items.len() - 1 {
                            selected += 1
                        }
                    }
                    KeyCode::Enter => match selected {
                        0 => {
                            disable_raw_mode()?;
                            execute!(
                                terminal.backend_mut(),
                                LeaveAlternateScreen,
                                DisableMouseCapture
                            )?;
                            run_downloader_ui(&mut terminal)?; // Added &mut here
                            return Ok(());
                        }
                        1 => break,
                        _ => {}
                    },
                    KeyCode::Esc => break,
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    Ok(())
}

fn run_downloader_ui(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
) -> Result<(), Box<dyn Error>> {
    // Dynamically get the download list
    let wget_string = wget_list::get_wget_list()?;
    let files: Vec<String> = wget_string.lines().map(|s| s.to_string()).collect();
    let mut progress: Vec<f64> = vec![0.0; files.len()];

    // Example: simulate progress
    for (i, _file) in files.iter().enumerate() {
        for p in 0..=100 {
            progress[i] = p as f64;

            terminal.draw(|f| {
                let size = f.size();
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(2)
                    .constraints(
                        files
                            .iter()
                            .map(|_| Constraint::Length(3))
                            .collect::<Vec<_>>(),
                    )
                    .split(size);

                for (idx, &prog) in progress.iter().enumerate() {
                    let gauge = Gauge::default()
                        .block(
                            Block::default()
                                .title(files[idx].as_str())
                                .borders(Borders::ALL),
                        ) // Changed to .as_str()
                        .gauge_style(Style::default().fg(Color::Green).bg(Color::Black))
                        .percent(prog as u16);
                    f.render_widget(gauge, chunks[idx]);
                }
            })?;

            std::thread::sleep(Duration::from_millis(20)); // simulate download
        }
    }

    terminal.draw(|f| {
        let size = f.size();
        let paragraph = Paragraph::new("Download completed! Press any key to return.")
            .block(Block::default().borders(Borders::ALL).title("Status"));
        f.render_widget(paragraph, size);
    })?;

    loop {
        if let Event::Key(_) = event::read()? {
            break;
        }
    }

    Ok(())
}
