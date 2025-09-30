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
use std::collections::HashMap;
#[cfg(feature = "tui")]
use std::io::{self, stdout};
#[cfg(feature = "tui")]
use std::path::PathBuf;
#[cfg(feature = "tui")]
use std::time::{Duration, Instant};

#[cfg(feature = "tui")]
use crate::{downloader, mirrors, wget_list};

#[cfg(feature = "tui")]
fn init_environment() -> PathBuf {
    let tmp_path = format!("/tmp/lfs_{}", rand::random::<u32>() % 9000 + 1000);
    println!("ℹ️ Using temporary path {}", tmp_path);
    PathBuf::from(tmp_path).join("sources")
}

#[cfg(feature = "tui")]
fn select_mirrors_tui(mirrors: Vec<String>) -> Vec<String> {
    if mirrors.is_empty() {
        return vec![];
    }

    let mut selected: Vec<bool> = vec![false; mirrors.len()];
    let mut state = ListState::default();
    state.select(Some(0));

    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen).unwrap();
    enable_raw_mode().unwrap();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();

    loop {
        terminal
            .draw(|f| {
                let size = f.size();
                let block = Block::default()
                    .title("Select mirrors (space to toggle, Enter to confirm)")
                    .borders(Borders::ALL);
                f.render_widget(block, size);

                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(2)
                    .constraints(vec![Constraint::Length(3); mirrors.len()])
                    .split(size);

                for (i, mirror) in mirrors.iter().enumerate() {
                    let mut style = Style::default();
                    if Some(i) == state.selected() {
                        style = style.bg(Color::Red).fg(Color::White);
                    }
                    let prefix = if selected[i] { "[x] " } else { "[ ] " };
                    let list_item = ListItem::new(format!("{}{}", prefix, mirror)).style(style);
                    let list =
                        List::new(vec![list_item]).block(Block::default().borders(Borders::ALL));
                    f.render_widget(list, chunks[i]);
                }
            })
            .unwrap();

        if let Event::Key(key) = event::read().unwrap() {
            match key.code {
                KeyCode::Down => {
                    let i = state.selected().unwrap_or(0);
                    if i < mirrors.len() - 1 {
                        state.select(Some(i + 1));
                    }
                }
                KeyCode::Up => {
                    let i = state.selected().unwrap_or(0);
                    if i > 0 {
                        state.select(Some(i - 1));
                    }
                }
                KeyCode::Char(' ') => {
                    let i = state.selected().unwrap_or(0);
                    selected[i] = !selected[i];
                }
                KeyCode::Enter => {
                    disable_raw_mode().unwrap();
                    execute!(terminal.backend_mut(), LeaveAlternateScreen).unwrap();
                    return mirrors
                        .into_iter()
                        .enumerate()
                        .filter_map(|(i, m)| if selected[i] { Some(m) } else { None })
                        .collect();
                }
                KeyCode::Esc => {
                    disable_raw_mode().unwrap();
                    execute!(terminal.backend_mut(), LeaveAlternateScreen).unwrap();
                    return vec![];
                }
                _ => {}
            }
        }
    }
}

#[cfg(feature = "tui")]
fn download_packages(lfs_sources: &PathBuf) {
    let mirrors_list = mirrors::fetch_mirrors().unwrap_or_else(|_| vec![]);
    let selected_mirrors = select_mirrors_tui(mirrors_list);
    if selected_mirrors.is_empty() {
        println!("⚠️ No mirrors selected!");
        return;
    }

    let wget_list = wget_list::get_wget_list().unwrap_or_default();
    if wget_list.is_empty() {
        println!("⚠️ No packages to download!");
        return;
    }

    let mut md5_map = HashMap::new();
    if let Ok(md5_content) = crate::md5_utils::get_md5sums() {
        for line in md5_content.lines() {
            if let Some((hash, filename)) = line.split_once(' ') {
                md5_map.insert(filename.to_string(), hash.to_string());
            }
        }
    }

    for file in wget_list {
        let mut downloaded = false;
        for mirror in &selected_mirrors {
            print!("⬇️ Downloading {} from {} ... ", file, mirror);
            io::stdout().flush().unwrap();

            let start = Instant::now();
            if downloader::download_files(&file, lfs_sources, Some(mirror.clone()), Some(&md5_map))
                .is_ok()
            {
                println!("✅ done in {:?}", start.elapsed());
                downloaded = true;
                break;
            } else {
                println!("⚠️ failed, trying next mirror...");
            }
        }
        if !downloaded {
            println!("❌ Failed to download {}", file);
        }
    }

    println!("🎉 All downloads finished!");
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
        "🔍 Check status",
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
                    Some(2) => println!("🔍 Status selected! (TODO)"),
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
