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
    widgets::{Block, Borders, List, ListItem, ListState},
};
#[cfg(feature = "tui")]
use spinners::{Spinner, Spinners};
#[cfg(feature = "tui")]
use std::{
    collections::HashMap,
    env,
    io::stdout,
    path::PathBuf,
    sync::{Arc, Mutex, mpsc::channel},
    thread,
    time::Duration,
};

#[cfg(feature = "tui")]
use crate::{downloader, md5_utils, mirrors, wget_list};

#[cfg(feature = "tui")]
fn init_environment() -> (PathBuf, String) {
    match env::var("LFS") {
        Ok(lfs) => (
            PathBuf::from(lfs).join("sources"),
            "Using LFS environment path.".into(),
        ),
        Err(_) => {
            let mut rng = rand::rng();
            let random_number: u32 = rng.random_range(1000..=9999);
            let tmp_path = format!("/tmp/lfs_{}", random_number);
            (
                PathBuf::from(&tmp_path).join("sources"),
                format!("Using temporary path {}", tmp_path),
            )
        }
    }
}

#[cfg(feature = "tui")]
fn prepare_wget_list() -> Vec<String> {
    wget_list::get_wget_list()
        .unwrap_or_default()
        .lines()
        .map(|s| s.to_string())
        .collect()
}

#[cfg(feature = "tui")]
fn prepare_md5_map() -> HashMap<String, String> {
    let mut map = HashMap::new();
    if let Ok(md5_content) = md5_utils::get_md5sums() {
        for line in md5_content.lines() {
            let mut parts = line.split_whitespace();
            if let (Some(hash), Some(filename)) = (parts.next(), parts.next()) {
                map.insert(filename.to_string(), hash.to_string());
            }
        }
    }
    map
}

#[cfg(feature = "tui")]
pub fn tui_menu() -> Result<(), Box<dyn std::error::Error>> {
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    enable_raw_mode()?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        let menu_items = [
            "🌱 Init environment",
            "🌐 Select mirror",
            "📦 Download packages",
            "🔍 Check status",
            "❌ Exit",
        ];
        let mut state = ListState::default();
        state.select(Some(0));

        let mut lfs_sources: Option<PathBuf> = None;
        let mut mirrors_list: Vec<String> = Vec::new();
        let mut selected_mirror: Option<String> = None;
        let log_messages: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let progress_state: Arc<Mutex<HashMap<String, Option<Spinner>>>> =
            Arc::new(Mutex::new(HashMap::new()));

        let (tx, rx) = channel::<String>();

        loop {
            while let Ok(msg) = rx.try_recv() {
                log_messages.lock().unwrap().push(msg);
            }

            terminal.draw(|f| {
                let size = f.area();
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(1)
                    .constraints(
                        vec![Constraint::Length(3); menu_items.len()]
                            .into_iter()
                            .chain(vec![Constraint::Min(5)])
                            .collect::<Vec<_>>(),
                    )
                    .split(size);

                for (i, item) in menu_items.iter().enumerate() {
                    let style = if Some(i) == state.selected() {
                        Style::default().bg(Color::Blue).fg(Color::White)
                    } else {
                        Style::default()
                    };
                    let list_item = ListItem::new(*item).style(style);
                    f.render_widget(
                        List::new(vec![list_item]).block(Block::default().borders(Borders::ALL)),
                        chunks[i],
                    );
                }

                let logs = log_messages.lock().unwrap();
                let mut combined_logs: Vec<ListItem> = logs
                    .iter()
                    .rev()
                    .take(chunks.last().unwrap().height as usize - 2)
                    .map(|l| ListItem::new(l.clone()))
                    .collect();

                let progress = progress_state.lock().unwrap();
                for (file, spinner_opt) in progress.iter() {
                    let display_status = if let Some(spinner) = spinner_opt {
                        spinner.to_string()
                    } else {
                        "✅ Done".to_string()
                    };
                    combined_logs.push(ListItem::new(format!("{}: {}", file, display_status)));
                }

                f.render_widget(
                    List::new(combined_logs)
                        .block(Block::default().title("Logs").borders(Borders::ALL)),
                    *chunks.last().unwrap(),
                );
            })?;

            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Down => state
                            .select(state.selected().map(|i| (i + 1).min(menu_items.len() - 1))),
                        KeyCode::Up => state.select(state.selected().map(|i| i.saturating_sub(1))),
                        KeyCode::Enter => match state.selected() {
                            Some(0) => {
                                let (path, msg) = init_environment();
                                lfs_sources = Some(path);
                                log_messages.lock().unwrap().push(msg);
                            }
                            Some(1) => {
                                if mirrors_list.is_empty() {
                                    mirrors_list = mirrors::fetch_mirrors().unwrap_or_else(|_| {
                                        vec![
                                            "ftp.fau.de".to_string(),
                                            "mirror.kernel.org".to_string(),
                                            "mirror.example.org".to_string(),
                                        ]
                                    });
                                }

                                let mut mirror_state = ListState::default();
                                mirror_state.select(Some(0));

                                loop {
                                    terminal.draw(|f| {
                                        let size = f.area();
                                        let mirror_items: Vec<ListItem> = mirrors_list
                                            .iter()
                                            .map(|m| ListItem::new(m.clone()))
                                            .collect();
                                        f.render_widget(
                                            List::new(mirror_items)
                                                .block(
                                                    Block::default()
                                                        .title("Select Mirror")
                                                        .borders(Borders::ALL),
                                                )
                                                .highlight_style(
                                                    Style::default()
                                                        .bg(Color::Blue)
                                                        .fg(Color::White),
                                                ),
                                            size,
                                        );
                                    })?;

                                    if let Event::Key(k) = event::read()? {
                                        match k.code {
                                            KeyCode::Down => mirror_state.select(
                                                mirror_state
                                                    .selected()
                                                    .map(|i| (i + 1).min(mirrors_list.len() - 1)),
                                            ),
                                            KeyCode::Up => mirror_state.select(
                                                mirror_state
                                                    .selected()
                                                    .map(|i| i.saturating_sub(1)),
                                            ),
                                            KeyCode::Enter => {
                                                if let Some(idx) = mirror_state.selected() {
                                                    selected_mirror =
                                                        Some(mirrors_list[idx].clone());
                                                    log_messages.lock().unwrap().push(format!(
                                                        "Selected mirror: {}",
                                                        mirrors_list[idx]
                                                    ));
                                                }
                                                break;
                                            }
                                            KeyCode::Esc => break,
                                            _ => {}
                                        }
                                    }
                                }
                            }
                            Some(2) => {
                                if let Some(ref path) = lfs_sources {
                                    let mirror = selected_mirror
                                        .clone()
                                        .unwrap_or_else(|| "ftp.fau.de".to_string());
                                    let wget_list = prepare_wget_list();
                                    let md5_map = prepare_md5_map();

                                    if wget_list.is_empty() {
                                        log_messages
                                            .lock()
                                            .unwrap()
                                            .push("⚠️ No packages to download!".into());
                                        continue;
                                    }

                                    let progress_clone = Arc::clone(&progress_state);
                                    let tx_clone = tx.clone();
                                    let path_clone = path.clone();

                                    thread::spawn(move || {
                                        for file in wget_list {
                                            let spinner = Spinner::new(
                                                Spinners::Dots9,
                                                format!("Downloading {}", file),
                                            );
                                            progress_clone
                                                .lock()
                                                .unwrap()
                                                .insert(file.clone(), Some(spinner));

                                            let result = downloader::download_files(
                                                &file,
                                                &path_clone,
                                                Some(mirror.clone()),
                                                Some(&md5_map),
                                            );
                                            progress_clone
                                                .lock()
                                                .unwrap()
                                                .insert(file.clone(), None);

                                            let status_msg = match result {
                                                Ok(_) => format!("✅ {}", file),
                                                Err(_) => format!("❌ {}", file),
                                            };
                                            let _ = tx_clone.send(status_msg);
                                        }
                                        let _ = tx_clone.send("🎉 All downloads complete!".into());
                                    });

                                    log_messages
                                        .lock()
                                        .unwrap()
                                        .push("⬇️ Download started...".into());
                                } else {
                                    log_messages
                                        .lock()
                                        .unwrap()
                                        .push("⚠️ Initialize environment first!".into());
                                }
                            }
                            Some(3) => log_messages
                                .lock()
                                .unwrap()
                                .push("🔍 Status check (TODO)".into()),
                            Some(4) => break,
                            Some(_) | None => break,
                        },
                        KeyCode::Esc => break,
                        _ => {}
                    }
                }
            }
        }

        Ok(())
    })();

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    result
}
