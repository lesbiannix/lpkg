#[cfg(feature = "tui")]
use crate::html::fetch_and_parse_html_to_json;
#[cfg(feature = "tui")]
use crate::{downloader, md5_utils, mirrors, wget_list};
#[cfg(feature = "tui")]
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
#[cfg(feature = "tui")]
use rand::Rng;
#[cfg(feature = "tui")]
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
#[cfg(feature = "tui")]
use std::{
    collections::HashMap,
    fs,
    io::stdout,
    path::PathBuf,
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

#[cfg(feature = "tui")]
use reqwest::blocking::get;
#[cfg(feature = "tui")]
use scraper::{Html, Selector};
#[cfg(feature = "tui")]
use serde_json::json;

// ----------------- HTML fetch function -----------------
#[cfg(feature = "tui")]
fn fetch_html_to_json(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let body = get(url)?.text()?;
    let document = Html::parse_document(&body);

    let selector = Selector::parse("body").unwrap();
    let mut contents = vec![];
    for element in document.select(&selector) {
        contents.push(element.text().collect::<Vec<_>>().join(" "));
    }

    Ok(serde_json::to_string_pretty(
        &json!({ "body_text": contents }),
    )?)
}

// ----------------- TUI functions -----------------
#[cfg(feature = "tui")]
fn init_environment() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let mut rng = rand::thread_rng();
    let random_number: u32 = rng.gen_range(1000..=9999);
    let tmp_base_path = PathBuf::from(format!("/tmp/lfs_{}", random_number));
    let lfs_sources_path = tmp_base_path.join("sources");

    fs::create_dir_all(&lfs_sources_path)?;
    Ok(lfs_sources_path)
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
                let items: Vec<ListItem> = mirrors
                    .iter()
                    .enumerate()
                    .map(|(i, mirror)| {
                        let prefix = if selected[i] { "[x] " } else { "[ ] " };
                        ListItem::new(format!("{}{}", prefix, mirror))
                    })
                    .collect();
                let list = List::new(items)
                    .block(
                        Block::default()
                            .title("Select mirrors")
                            .borders(Borders::ALL),
                    )
                    .highlight_symbol(">> ");
                f.render_stateful_widget(list, size, &mut state);
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
fn download_packages_tui(lfs_sources: &PathBuf) {
    let mirrors_list = mirrors::fetch_mirrors().unwrap_or_default();
    let selected_mirrors = select_mirrors_tui(mirrors_list);
    if selected_mirrors.is_empty() {
        return;
    }

    let wget_list_content = wget_list::get_wget_list().unwrap_or_default();
    let wget_list: Vec<String> = wget_list_content.lines().map(|s| s.to_string()).collect();
    if wget_list.is_empty() {
        return;
    }

    let mut md5_map = HashMap::new();
    if let Ok(md5_content) = md5_utils::get_md5sums() {
        for line in md5_content.lines() {
            if let Some((hash, filename)) = line.split_once(' ') {
                md5_map.insert(filename.to_string(), hash.to_string());
            }
        }
    }

    let download_state: Arc<Mutex<Vec<(String, String)>>> = Arc::new(Mutex::new(
        wget_list
            .iter()
            .map(|f| (f.clone(), "Pending".into()))
            .collect(),
    ));

    let download_state_clone = Arc::clone(&download_state);
    let mirrors_clone = selected_mirrors.clone();
    let lfs_sources = lfs_sources.clone();

    thread::spawn(move || {
        for file in &wget_list {
            let mut status = "Failed".to_string();
            for mirror in &mirrors_clone {
                if downloader::download_files(
                    file,
                    &lfs_sources,
                    Some(mirror.clone()),
                    Some(&md5_map),
                )
                .is_ok()
                {
                    status = format!("Downloaded from {}", mirror);
                    break;
                }
            }
            let mut state = download_state_clone.lock().unwrap();
            if let Some(entry) = state.iter_mut().find(|(f, _)| f == file) {
                entry.1 = status.clone();
            }
        }
    });

    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen).unwrap();
    enable_raw_mode().unwrap();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();

    loop {
        terminal
            .draw(|f| {
                let size = f.size();
                let items: Vec<ListItem> = {
                    let state = download_state.lock().unwrap();
                    state
                        .iter()
                        .map(|(f, s)| ListItem::new(format!("{}: {}", f, s)))
                        .collect()
                };
                let list = List::new(items).block(
                    Block::default()
                        .title("Downloading Packages")
                        .borders(Borders::ALL),
                );
                f.render_widget(list, size);
            })
            .unwrap();

        let state = download_state.lock().unwrap();
        if state.iter().all(|(_, s)| s != "Pending") {
            break;
        }
        thread::sleep(Duration::from_millis(100));
    }

    disable_raw_mode().unwrap();
    execute!(terminal.backend_mut(), LeaveAlternateScreen).unwrap();
}

// ----------------- Cleanup function -----------------
#[cfg(feature = "tui")]
fn cleanup_temp_directories() -> Result<usize, Box<dyn std::error::Error>> {
    let mut cleaned_count = 0;
    for entry in fs::read_dir("/tmp")? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                if name.starts_with("lfs_") {
                    fs::remove_dir_all(&path)?;
                    cleaned_count += 1;
                }
            }
        }
    }
    Ok(cleaned_count)
}

// ----------------- Main TUI menu -----------------
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
        "🧹 Clean up temp directories",
        "📄 Test JSON fetch", // NEW BUTTON
        "❌ Exit",
    ];
    let mut state = ListState::default();
    state.select(Some(0));

    let mut lfs_sources: Option<PathBuf> = None;
    let mut status_message: Option<String> = None;
    let mut status_message_timer: Option<Instant> = None;

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

            if let Some(msg) = &status_message {
                let paragraph = Paragraph::new(msg.as_str()).block(
                    Block::default()
                        .borders(Borders::NONE)
                        .style(Style::default().fg(Color::Yellow)),
                );
                let msg_area = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Percentage(10),
                        Constraint::Percentage(80),
                        Constraint::Percentage(10),
                    ])
                    .split(size)[1];
                let msg_area = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Min(0), Constraint::Length(1)])
                    .split(msg_area)[1];
                f.render_widget(paragraph, msg_area);
            }
        })?;

        if let Some(timer) = status_message_timer {
            if timer.elapsed() > Duration::from_secs(3) {
                status_message = None;
                status_message_timer = None;
            }
        }

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
                    Some(0) => {
                        // Init environment
                        match init_environment() {
                            Ok(path) => {
                                lfs_sources = Some(path.clone());
                                status_message =
                                    Some(format!("✅ Environment initialized: {}", path.display()));
                            }
                            Err(e) => {
                                status_message =
                                    Some(format!("❌ Failed to init environment: {}", e));
                            }
                        }
                        status_message_timer = Some(Instant::now());
                    }
                    Some(1) => {
                        // Download packages
                        if let Some(path) = &lfs_sources {
                            download_packages_tui(path);
                        }
                    }
                    Some(2) => {
                        // Status
                        status_message = Some("🔍 Status selected! (TODO)".to_string());
                        status_message_timer = Some(Instant::now());
                    }
                    Some(3) => {
                        // Cleanup
                        match cleanup_temp_directories() {
                            Ok(count) => {
                                status_message =
                                    Some(format!("✅ Cleaned {} temporary dirs", count));
                            }
                            Err(e) => {
                                status_message = Some(format!("❌ Failed cleanup: {}", e));
                            }
                        }
                        status_message_timer = Some(Instant::now());
                    }
                    Some(4) => {
                        // Test JSON fetch
                        // Leave TUI first
                        disable_raw_mode()?;
                        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
                        terminal.show_cursor()?;

                        match fetch_and_parse_html_to_json(
                            "https://www.linuxfromscratch.org/~thomas/multilib-m32/chapter02/hostreqs.html",
                        ) {
                            Ok(json) => println!("✅ JSON output:\n{}", json),
                            Err(e) => eprintln!("❌ Error: {}", e),
                        }

                        return Ok(());
                    }
                    Some(5) | _ => break,
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
