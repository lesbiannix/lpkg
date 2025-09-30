#[cfg(feature = "tui")]
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
#[cfg(feature = "tui")]
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, ListState},
};
#[cfg(feature = "tui")]
use spinners::{Spinner, Spinners};
#[cfg(feature = "tui")]
use std::io::{self, stdout};
#[cfg(feature = "tui")]
use std::path::PathBuf;
#[cfg(feature = "tui")]
use std::process::Command;

#[cfg(feature = "tui")]
use crate::{downloader, mirrors, wget_list};

#[cfg(feature = "tui")]
fn init_environment() -> PathBuf {
    let tmp_path = "/tmp/lfs_tmp"; // Simplified for demo
    PathBuf::from(tmp_path).join("sources")
}

#[cfg(feature = "tui")]
fn download_packages(lfs_sources: &PathBuf) {
    let spinner = Spinner::new(Spinners::Dots9, "Downloading packages...".into());
    let wget_list = wget_list::get_wget_list().unwrap_or_default();
    let package_mirror =
        mirrors::choose_package_mirror().unwrap_or_else(|| "ftp.fau.de".to_string());

    // Simplified download call
    let _ = downloader::download_files(&wget_list, lfs_sources, Some(package_mirror), None);

    spinner.stop();
}

#[cfg(feature = "tui")]
fn format_drive_tui() -> Result<(), Box<dyn std::error::Error>> {
    // Mocked drive list for demo
    let drives = vec!["/dev/sda".to_string(), "/dev/sdb".to_string()];
    let mut state = ListState::default();
    state.select(Some(0));

    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        terminal.draw(|f| {
            let size = f.size();
            let block = Block::default()
                .title("💾 Format Drive")
                .borders(Borders::ALL);
            f.render_widget(block, size);

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints(vec![Constraint::Length(3); drives.len()])
                .split(size);

            for (i, drive) in drives.iter().enumerate() {
                let mut style = Style::default();
                if Some(i) == state.selected() {
                    style = style.bg(Color::Red).fg(Color::White);
                }
                let list_item = ListItem::new(drive.clone()).style(style);
                let list = List::new(vec![list_item]).block(Block::default().borders(Borders::ALL));
                f.render_widget(list, chunks[i]);
            }
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Down => {
                    let i = state.selected().unwrap_or(0);
                    if i < drives.len() - 1 {
                        state.select(Some(i + 1));
                    }
                }
                KeyCode::Up => {
                    let i = state.selected().unwrap_or(0);
                    if i > 0 {
                        state.select(Some(i - 1));
                    }
                }
                KeyCode::Enter => {
                    if let Some(idx) = state.selected() {
                        let drive = &drives[idx];
                        disable_raw_mode()?;
                        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
                        println!("⚠️  Confirm formatting {}? (y/n)", drive);
                        let mut input = String::new();
                        io::stdin().read_line(&mut input)?;
                        if matches!(input.trim().to_lowercase().as_str(), "y" | "yes") {
                            println!("Formatting {}...", drive);
                            let _ = Command::new("mkfs.ext4").arg(drive).status();
                            println!("✅ Done!");
                        }
                        enable_raw_mode()?;
                        execute!(terminal.backend_mut(), EnterAlternateScreen)?;
                    }
                }
                KeyCode::Esc => break,
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

#[cfg(feature = "tui")]
pub fn tui_menu() -> Result<(), Box<dyn std::error::Error>> {
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    enable_raw_mode()?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let menu_items = vec![
        "🌱 Init environment",
        "📦 Download packages",
        "💾 Format drive",
        "❌ Exit",
    ];
    let mut state = ListState::default();
    state.select(Some(0));

    let mut lfs_sources: Option<PathBuf> = None;

    loop {
        terminal.draw(|f| {
            let size = f.size();
            let block = Block::default()
                .title("✨ lpkg TUI 🌈")
                .borders(Borders::ALL);
            f.render_widget(block, size);

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints(vec![Constraint::Length(3); menu_items.len()])
                .split(size);

            for (i, item) in menu_items.iter().enumerate() {
                let mut style = Style::default();
                if Some(i) == state.selected() {
                    style = style.bg(Color::Red).fg(Color::White);
                }
                let list_item = ListItem::new(*item).style(style);
                let list = List::new(vec![list_item]).block(Block::default().borders(Borders::ALL));
                f.render_widget(list, chunks[i]);
            }
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Down => {
                    let i = state.selected().unwrap_or(0);
                    if i < menu_items.len() - 1 {
                        state.select(Some(i + 1));
                    }
                }
                KeyCode::Up => {
                    let i = state.selected().unwrap_or(0);
                    if i > 0 {
                        state.select(Some(i - 1));
                    }
                }
                KeyCode::Enter => match state.selected() {
                    Some(0) => lfs_sources = Some(init_environment()),
                    Some(1) => {
                        if let Some(ref path) = lfs_sources {
                            download_packages(path);
                        } else {
                            println!("⚠️ Please initialize environment first!");
                        }
                    }
                    Some(2) => {
                        format_drive_tui()?;
                    }
                    Some(3) | _ => break,
                },
                KeyCode::Esc => break,
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
