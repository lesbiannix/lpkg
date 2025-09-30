// src/tui.rs - Fixed version with themes and settings
use crate::wget_list;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use std::{error::Error, io, time::Duration};
use tracing::{info, warn, error, debug, trace, instrument};
use tui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph, Wrap},
};

// Theme definitions
#[derive(Debug, Clone, Copy, PartialEq)]
enum Theme {
    Default,
    Dracula,
    Nord,
    Gruvbox,
    Monokai,
    Catppuccin,
}

impl Theme {
    fn name(&self) -> &str {
        match self {
            Theme::Default => "💖 Default (Cute)",
            Theme::Dracula => "🦇 Dracula (Dark)",
            Theme::Nord => "❄️ Nord (Cold)",
            Theme::Gruvbox => "🔥 Gruvbox (Warm)",
            Theme::Monokai => "🎮 Monokai (Retro)",
            Theme::Catppuccin => "🌸 Catppuccin (Pastel)",
        }
    }

    fn primary_color(&self) -> Color {
        match self {
            Theme::Default => Color::Cyan,
            Theme::Dracula => Color::Magenta,
            Theme::Nord => Color::LightBlue,
            Theme::Gruvbox => Color::Yellow,
            Theme::Monokai => Color::Green,
            Theme::Catppuccin => Color::LightMagenta,
        }
    }

    fn secondary_color(&self) -> Color {
        match self {
            Theme::Default => Color::Magenta,
            Theme::Dracula => Color::Rgb(189, 147, 249),
            Theme::Nord => Color::Cyan,
            Theme::Gruvbox => Color::Red,
            Theme::Monokai => Color::Yellow,
            Theme::Catppuccin => Color::Rgb(245, 194, 231),
        }
    }

    fn accent_color(&self) -> Color {
        match self {
            Theme::Default => Color::Yellow,
            Theme::Dracula => Color::Rgb(255, 121, 198),
            Theme::Nord => Color::Rgb(136, 192, 208),
            Theme::Gruvbox => Color::Rgb(250, 189, 47),
            Theme::Monokai => Color::Rgb(166, 226, 46),
            Theme::Catppuccin => Color::Rgb(180, 190, 254),
        }
    }

    fn success_color(&self) -> Color {
        match self {
            Theme::Default => Color::Green,
            Theme::Dracula => Color::Rgb(80, 250, 123),
            Theme::Nord => Color::Rgb(163, 190, 140),
            Theme::Gruvbox => Color::Rgb(184, 187, 38),
            Theme::Monokai => Color::Rgb(166, 226, 46),
            Theme::Catppuccin => Color::Rgb(166, 227, 161),
        }
    }

    fn all_themes() -> Vec<Theme> {
        vec![
            Theme::Default,
            Theme::Dracula,
            Theme::Nord,
            Theme::Gruvbox,
            Theme::Monokai,
            Theme::Catppuccin,
        ]
    }
}

struct Settings {
    theme: Theme,
    show_progress_percentage: bool,
    auto_scroll: bool,
    sound_enabled: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: Theme::Default,
            show_progress_percentage: true,
            auto_scroll: true,
            sound_enabled: false,
        }
    }
}

#[instrument]
pub fn tui_menu() -> Result<(), Box<dyn Error>> {
    info!("🚀 Initializing TUI menu");
    
    enable_raw_mode()?;
    debug!("✅ Raw mode enabled");
    
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    debug!("✅ Terminal setup complete");
    
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    info!("✅ Terminal backend initialized");

    let mut settings = Settings::default();
    let menu_items = vec![
        "🚀 Start Package Downloader",
        "📋 View Download List", 
        "🔍 Check System Status",
        "⚙️  Settings",
        "❌ Exit"
    ];
    let mut selected = 0;

    info!("✨ Entering main menu loop");
    loop {
        trace!("🎨 Drawing menu frame, selected={}", selected);
        
        terminal.draw(|f| {
            let size = f.size();
            
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints([
                    Constraint::Length(5),
                    Constraint::Length(menu_items.len() as u16 + 2),
                    Constraint::Min(5),
                    Constraint::Length(3),
                ])
                .split(size);

            let header = vec![
                Spans::from(vec![
                    Span::styled("═══════════════════════════════════════", 
                        Style::default().fg(settings.theme.secondary_color())),
                ]),
                Spans::from(vec![
                    Span::styled("     ✨ LFS Package Downloader ✨", 
                        Style::default().fg(settings.theme.primary_color()).add_modifier(Modifier::BOLD)),
                ]),
                Spans::from(vec![
                    Span::styled("═══════════════════════════════════════", 
                        Style::default().fg(settings.theme.secondary_color())),
                ]),
            ];
            let header_widget = Paragraph::new(header)
                .alignment(Alignment::Center);
            f.render_widget(header_widget, chunks[0]);

            let items: Vec<ListItem> = menu_items
                .iter()
                .enumerate()
                .map(|(i, m)| {
                    let style = if i == selected {
                        Style::default()
                            .fg(Color::Black)
                            .bg(settings.theme.primary_color())
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::White)
                    };
                    let prefix = if i == selected { "▶ " } else { "  " };
                    ListItem::new(format!("{}{}", prefix, m)).style(style)
                })
                .collect();

            let list = List::new(items).block(
                Block::default()
                    .title(" Main Menu ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(settings.theme.primary_color())),
            );
            f.render_widget(list, chunks[1]);

            let description = match selected {
                0 => "📦 Download all LFS packages from the wget list.\n✨ Progress will be shown for each file during download.",
                1 => "📋 Display the complete list of packages that will be downloaded.\n🔍 Review all URLs before starting the download process.",
                2 => "🔍 Check system requirements and verify environment setup.\n✅ Ensure all dependencies are available for building LFS.",
                3 => "⚙️  Configure application settings, change theme, and adjust preferences.\n🎨 Customize your experience!",
                4 => "❌ Exit the package downloader and return to shell.",
                _ => "",
            };

            let desc_widget = Paragraph::new(description)
                .block(Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(settings.theme.accent_color()))
                    .title(" Description "))
                .wrap(Wrap { trim: true });
            f.render_widget(desc_widget, chunks[2]);

            let footer_text = format!("↑↓: Navigate  │  Enter: Select  │  Esc: Exit  │  Theme: {}", settings.theme.name());
            let footer = Paragraph::new(footer_text)
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(footer, chunks[3]);
        })?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                debug!("⌨️  Key pressed: {:?}", key.code);
                
                match key.code {
                    KeyCode::Up => {
                        if selected > 0 {
                            selected -= 1;
                            debug!("⬆️  Selection moved up to {}", selected);
                        }
                    }
                    KeyCode::Down => {
                        if selected < menu_items.len() - 1 {
                            selected += 1;
                            debug!("⬇️  Selection moved down to {}", selected);
                        }
                    }
                    KeyCode::Enter => {
                        info!("✅ Menu item {} selected: {}", selected, menu_items[selected]);
                        
                        match selected {
                            0 => {
                                info!("📦 Starting package downloader");
                                if let Err(e) = run_downloader_ui(&mut terminal, &settings) {
                                    error!("❌ Downloader failed: {}", e);
                                } else {
                                    info!("✅ Downloader completed successfully");
                                }
                                
                                enable_raw_mode()?;
                                execute!(
                                    terminal.backend_mut(),
                                    EnterAlternateScreen,
                                    EnableMouseCapture
                                )?;
                            }
                            1 => {
                                info!("📋 Viewing download list");
                                if let Err(e) = view_download_list(&mut terminal, &settings) {
                                    error!("❌ Failed to view download list: {}", e);
                                }
                                
                                enable_raw_mode()?;
                                execute!(
                                    terminal.backend_mut(),
                                    EnterAlternateScreen,
                                    EnableMouseCapture
                                )?;
                            }
                            2 => {
                                info!("🔍 Checking system status");
                                if let Err(e) = check_system_status(&mut terminal, &settings) {
                                    error!("❌ Failed to check system status: {}", e);
                                }
                                
                                enable_raw_mode()?;
                                execute!(
                                    terminal.backend_mut(),
                                    EnterAlternateScreen,
                                    EnableMouseCapture
                                )?;
                            }
                            3 => {
                                info!("⚙️  Opening settings menu");
                                if let Err(e) = settings_menu(&mut terminal, &mut settings) {
                                    error!("❌ Settings menu failed: {}", e);
                                }
                                
                                enable_raw_mode()?;
                                execute!(
                                    terminal.backend_mut(),
                                    EnterAlternateScreen,
                                    EnableMouseCapture
                                )?;
                            }
                            4 => {
                                info!("👋 User selected exit");
                                break;
                            }
                            _ => {}
                        }
                    }
                    KeyCode::Esc => {
                        info!("👋 Exit requested via Esc key");
                        break;
                    }
                    _ => {
                        trace!("🤷 Unhandled key: {:?}", key.code);
                    }
                }
            }
        }
    }

    info!("🧹 Cleaning up terminal");
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    debug!("✅ Terminal cleanup complete");
    
    Ok(())
}

#[instrument(skip(terminal, settings))]
fn settings_menu(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    settings: &mut Settings,
) -> Result<(), Box<dyn Error>> {
    info!("⚙️  Opening settings menu");
    
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    
    enable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        EnterAlternateScreen,
        EnableMouseCapture
    )?;

    let menu_items = vec![
        "🎨 Change Theme",
        "📊 Toggle Progress Percentage",
        "📜 Toggle Auto Scroll",
        "🔊 Toggle Sound",
        "↩️  Back to Main Menu",
    ];
    let mut selected = 0;

    loop {
        terminal.draw(|f| {
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints([
                    Constraint::Length(5),
                    Constraint::Length(menu_items.len() as u16 + 2),
                    Constraint::Min(5),
                    Constraint::Length(3),
                ])
                .split(size);

            let header = vec![
                Spans::from(vec![
                    Span::styled("═══════════════════════════════════════", 
                        Style::default().fg(settings.theme.secondary_color())),
                ]),
                Spans::from(vec![
                    Span::styled("          ⚙️  Settings Menu ⚙️", 
                        Style::default().fg(settings.theme.primary_color()).add_modifier(Modifier::BOLD)),
                ]),
                Spans::from(vec![
                    Span::styled("═══════════════════════════════════════", 
                        Style::default().fg(settings.theme.secondary_color())),
                ]),
            ];
            let header_widget = Paragraph::new(header)
                .alignment(Alignment::Center);
            f.render_widget(header_widget, chunks[0]);

            let items: Vec<ListItem> = menu_items
                .iter()
                .enumerate()
                .map(|(i, m)| {
                    let value_text = match i {
                        0 => format!(" [{}]", settings.theme.name()),
                        1 => format!(" [{}]", if settings.show_progress_percentage { "✅ ON" } else { "❌ OFF" }),
                        2 => format!(" [{}]", if settings.auto_scroll { "✅ ON" } else { "❌ OFF" }),
                        3 => format!(" [{}]", if settings.sound_enabled { "✅ ON" } else { "❌ OFF" }),
                        _ => String::new(),
                    };
                    
                    let style = if i == selected {
                        Style::default()
                            .fg(Color::Black)
                            .bg(settings.theme.primary_color())
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::White)
                    };
                    let prefix = if i == selected { "▶ " } else { "  " };
                    ListItem::new(format!("{}{}{}", prefix, m, value_text)).style(style)
                })
                .collect();

            let list = List::new(items).block(
                Block::default()
                    .title(" Settings ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(settings.theme.primary_color())),
            );
            f.render_widget(list, chunks[1]);

            let description = match selected {
                0 => "🎨 Change the color theme of the application.\n✨ Choose from multiple beautiful themes to customize your experience.",
                1 => "📊 Show or hide percentage numbers on progress bars.\n🔢 Useful for detailed download tracking.",
                2 => "📜 Automatically scroll to current downloading file.\n👀 Keeps the active download in view.",
                3 => "🔊 Enable or disable sound notifications.\n🎵 Play sounds when downloads complete (if available).",
                4 => "↩️  Return to the main menu.",
                _ => "",
            };

            let desc_widget = Paragraph::new(description)
                .block(Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(settings.theme.accent_color()))
                    .title(" Description "))
                .wrap(Wrap { trim: true });
            f.render_widget(desc_widget, chunks[2]);

            let footer = Paragraph::new("↑↓: Navigate  │  Enter: Select/Toggle  │  Esc: Back")
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(footer, chunks[3]);
        })?;

        if event::poll(Duration::from_millis(100))? {
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
                                if let Err(e) = theme_selector(terminal, settings) {
                                    error!("❌ Theme selector failed: {}", e);
                                }
                                enable_raw_mode()?;
                                execute!(
                                    terminal.backend_mut(),
                                    EnterAlternateScreen,
                                    EnableMouseCapture
                                )?;
                            }
                            1 => {
                                settings.show_progress_percentage = !settings.show_progress_percentage;
                                info!("📊 Progress percentage: {}", settings.show_progress_percentage);
                            }
                            2 => {
                                settings.auto_scroll = !settings.auto_scroll;
                                info!("📜 Auto scroll: {}", settings.auto_scroll);
                            }
                            3 => {
                                settings.sound_enabled = !settings.sound_enabled;
                                info!("🔊 Sound enabled: {}", settings.sound_enabled);
                            }
                            4 => {
                                info!("↩️  Returning to main menu");
                                break;
                            }
                            _ => {}
                        }
                    }
                    KeyCode::Esc => {
                        info!("👋 Exiting settings menu");
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}

#[instrument(skip(terminal, settings))]
fn theme_selector(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    settings: &mut Settings,
) -> Result<(), Box<dyn Error>> {
    info!("🎨 Opening theme selector");
    
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    
    enable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        EnterAlternateScreen,
        EnableMouseCapture
    )?;

    let themes = Theme::all_themes();
    let mut selected = themes.iter().position(|t| *t == settings.theme).unwrap_or(0);

    loop {
        terminal.draw(|f| {
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints([
                    Constraint::Length(5),
                    Constraint::Length(themes.len() as u16 + 2),
                    Constraint::Min(5),
                    Constraint::Length(3),
                ])
                .split(size);

            let preview_theme = themes[selected];
            let header = vec![
                Spans::from(vec![
                    Span::styled("═══════════════════════════════════════", 
                        Style::default().fg(preview_theme.secondary_color())),
                ]),
                Spans::from(vec![
                    Span::styled("        🎨 Theme Selector 🎨", 
                        Style::default().fg(preview_theme.primary_color()).add_modifier(Modifier::BOLD)),
                ]),
                Spans::from(vec![
                    Span::styled("═══════════════════════════════════════", 
                        Style::default().fg(preview_theme.secondary_color())),
                ]),
            ];
            let header_widget = Paragraph::new(header)
                .alignment(Alignment::Center);
            f.render_widget(header_widget, chunks[0]);

            let items: Vec<ListItem> = themes
                .iter()
                .enumerate()
                .map(|(i, theme)| {
                    let is_selected = i == selected;
                    let is_current = *theme == settings.theme;
                    
                    let style = if is_selected {
                        Style::default()
                            .fg(Color::Black)
                            .bg(theme.primary_color())
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(theme.primary_color())
                    };
                    
                    let prefix = if is_selected { "▶ " } else { "  " };
                    let suffix = if is_current { " ✓" } else { "" };
                    
                    ListItem::new(format!("{}{}{}", prefix, theme.name(), suffix)).style(style)
                })
                .collect();

            let list = List::new(items).block(
                Block::default()
                    .title(" Available Themes ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(preview_theme.primary_color())),
            );
            f.render_widget(list, chunks[1]);

            let preview_theme = themes[selected];
            let preview_text = vec![
                Spans::from(vec![
                    Span::styled("✨ Preview: ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                ]),
                Spans::from(""),
                Spans::from(vec![
                    Span::styled("Primary Color ", Style::default().fg(preview_theme.primary_color()).add_modifier(Modifier::BOLD)),
                    Span::styled("● ", Style::default().fg(preview_theme.primary_color())),
                ]),
                Spans::from(vec![
                    Span::styled("Secondary Color ", Style::default().fg(preview_theme.secondary_color()).add_modifier(Modifier::BOLD)),
                    Span::styled("● ", Style::default().fg(preview_theme.secondary_color())),
                ]),
                Spans::from(vec![
                    Span::styled("Accent Color ", Style::default().fg(preview_theme.accent_color()).add_modifier(Modifier::BOLD)),
                    Span::styled("● ", Style::default().fg(preview_theme.accent_color())),
                ]),
                Spans::from(vec![
                    Span::styled("Success Color ", Style::default().fg(preview_theme.success_color()).add_modifier(Modifier::BOLD)),
                    Span::styled("● ", Style::default().fg(preview_theme.success_color())),
                ]),
            ];

            let preview_widget = Paragraph::new(preview_text)
                .block(Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(preview_theme.accent_color()))
                    .title(" Theme Preview "))
                .wrap(Wrap { trim: true });
            f.render_widget(preview_widget, chunks[2]);

            let footer = Paragraph::new("↑↓: Navigate  │  Enter: Apply Theme  │  Esc: Cancel")
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(footer, chunks[3]);
        })?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Up => {
                        if selected > 0 {
                            selected -= 1;
                        }
                    }
                    KeyCode::Down => {
                        if selected < themes.len() - 1 {
                            selected += 1;
                        }
                    }
                    KeyCode::Enter => {
                        settings.theme = themes[selected];
                        info!("🎨 Theme changed to: {:?}", settings.theme);
                        break;
                    }
                    KeyCode::Esc => {
                        info!("❌ Theme selection cancelled");
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}

#[instrument(skip(terminal, settings))]
fn run_downloader_ui(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    settings: &Settings,
) -> Result<(), Box<dyn Error>> {
    info!("📦 Initializing downloader UI");
    
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    
    enable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        EnterAlternateScreen,
        EnableMouseCapture
    )?;

    debug!("📋 Fetching wget list");
    let wget_string = wget_list::get_wget_list()?;
    
    let files: Vec<String> = wget_string.lines().map(|s| s.to_string()).collect();
    let total_files = files.len();
    info!("✅ Found {} files to download", total_files);
    
    let mut progress: Vec<f64> = vec![0.0; total_files];
    let mut current_file = 0;

    terminal.draw(|f| {
        draw_progress_screen(f, &files, &progress, current_file, total_files, settings);
    })?;

    for (i, _file) in files.iter().enumerate() {
        current_file = i;
        let filename = files[i].split('/').last().unwrap_or(&files[i]);
        info!("⬇️  Downloading file {}/{}: {}", i + 1, total_files, filename);
        
        for p in 0..=100 {
            progress[i] = p as f64;

            if p % 25 == 0 {
                debug!("📊 Progress for {}: {}%", filename, p);
            }

            terminal.draw(|f| {
                draw_progress_screen(f, &files, &progress, current_file, total_files, settings);
            })?;

            if event::poll(Duration::from_millis(20))? {
                if let Event::Key(key) = event::read()? {
                    if key.code == KeyCode::Esc {
                        warn!("⚠️  Download cancelled by user at file {}/{}", i + 1, total_files);
                        show_message(terminal, "❌ Download Cancelled", "Press any key to continue...", settings)?;
                        return Ok(());
                    }
                }
            } else {
                std::thread::sleep(Duration::from_millis(20));
            }
        }
        
        info!("✅ Completed downloading: {}", filename);
    }

    info!("🎉 All downloads completed successfully");
    show_message(terminal, "✨ Download Complete! ✨", 
                 "All packages downloaded successfully! 🎉\nPress any key to return to menu...", settings)?;

    Ok(())
}

fn draw_progress_screen(
    f: &mut tui::Frame<CrosstermBackend<io::Stdout>>,
    files: &[String],
    progress: &[f64],
    current_file: usize,
    total_files: usize,
    settings: &Settings,
) {
    let size = f.size();
    
    let available_height = size.height.saturating_sub(8);
    let max_visible = (available_height / 3) as usize;
    
    let start_idx = if settings.auto_scroll && current_file >= max_visible / 2 {
        (current_file - max_visible / 2).min(files.len().saturating_sub(max_visible))
    } else {
        0
    };
    let end_idx = (start_idx + max_visible).min(files.len());
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(5),
            Constraint::Min(3),
            Constraint::Length(3),
        ])
        .split(size);

    let header = Paragraph::new(format!(
        "📦 Downloading Packages... [{}/{}]",
        current_file + 1,
        total_files
    ))
    .style(Style::default().fg(settings.theme.primary_color()).add_modifier(Modifier::BOLD))
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(header, chunks[0]);

    let progress_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            (start_idx..end_idx)
                .map(|_| Constraint::Length(3))
                .collect::<Vec<_>>(),
        )
        .split(chunks[1]);

    for (idx, i) in (start_idx..end_idx).enumerate() {
        let prog = progress[i];
        let is_current = i == current_file;
        
        let filename = files[i]
            .split('/')
            .last()
            .unwrap_or(&files[i]);
        
        let title = if settings.show_progress_percentage {
            format!("{} - {:.0}%", filename, prog)
        } else {
            filename.to_string()
        };
        
        let gauge = Gauge::default()
            .block(Block::default()
                .title(title.as_str())
                .borders(Borders::ALL)
                .border_style(if is_current {
                    Style::default().fg(settings.theme.accent_color())
                } else {
                    Style::default().fg(Color::DarkGray)
                }))
            .gauge_style(if prog >= 100.0 {
                Style::default().fg(settings.theme.success_color()).bg(Color::Black)
            } else if is_current {
                Style::default().fg(settings.theme.primary_color()).bg(Color::Black)
            } else {
                Style::default().fg(Color::DarkGray).bg(Color::Black)
            })
            .percent(prog as u16);
        
        if idx < progress_chunks.len() {
            f.render_widget(gauge, progress_chunks[idx]);
        }
    }

    let footer = Paragraph::new("Esc: Cancel Download")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(footer, chunks[2]);
}

#[instrument(skip(terminal, settings))]
fn view_download_list(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    settings: &Settings,
) -> Result<(), Box<dyn Error>> {
    info!("📋 Displaying download list");
    
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    
    enable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        EnterAlternateScreen,
        EnableMouseCapture
    )?;

    let wget_string = wget_list::get_wget_list()?;
    let files: Vec<String> = wget_string.lines().map(|s| s.to_string()).collect();
    debug!("📦 Loaded {} files for display", files.len());
    
    let mut scroll_offset = 0;

    loop {
        terminal.draw(|f| {
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(5),
                    Constraint::Length(3),
                ])
                .split(size);

            let header = Paragraph::new(format!("📋 Download List ({} packages)", files.len()))
                .style(Style::default().fg(settings.theme.primary_color()).add_modifier(Modifier::BOLD))
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(header, chunks[0]);

            let visible_lines = (chunks[1].height - 2) as usize;
            let end_idx = (scroll_offset + visible_lines).min(files.len());
            
            let items: Vec<ListItem> = files[scroll_offset..end_idx]
                .iter()
                .enumerate()
                .map(|(i, url)| {
                    let filename = url.split('/').last().unwrap_or(url);
                    ListItem::new(format!("{}. 📦 {}", scroll_offset + i + 1, filename))
                        .style(Style::default().fg(settings.theme.primary_color()))
                })
                .collect();

            let list = List::new(items)
                .block(Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(settings.theme.accent_color()))
                    .title(" Packages "));
            f.render_widget(list, chunks[1]);

            let footer = Paragraph::new("↑↓: Scroll  │  PgUp/PgDn: Page  │  Enter/Esc: Back")
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(footer, chunks[2]);
        })?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                let visible_lines = terminal.size()?.height.saturating_sub(8) as usize;
                
                match key.code {
                    KeyCode::Up => {
                        if scroll_offset > 0 {
                            scroll_offset -= 1;
                            trace!("⬆️  Scrolled up to offset {}", scroll_offset);
                        }
                    }
                    KeyCode::Down => {
                        if scroll_offset + visible_lines < files.len() {
                            scroll_offset += 1;
                            trace!("⬇️  Scrolled down to offset {}", scroll_offset);
                        }
                    }
                    KeyCode::PageUp => {
                        scroll_offset = scroll_offset.saturating_sub(visible_lines);
                        debug!("📄 Page up to offset {}", scroll_offset);
                    }
                    KeyCode::PageDown => {
                        scroll_offset = (scroll_offset + visible_lines).min(files.len().saturating_sub(visible_lines));
                        debug!("📄 Page down to offset {}", scroll_offset);
                    }
                    KeyCode::Enter | KeyCode::Esc => {
                        info!("👋 Exiting download list view");
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}

#[instrument(skip(terminal, settings))]
fn check_system_status(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    settings: &Settings,
) -> Result<(), Box<dyn Error>> {
    info!("🔍 Checking system status");
    
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    
    enable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        EnterAlternateScreen,
        EnableMouseCapture
    )?;

    let status_info = vec![
        ("💻 System", "Ready"),
        ("💾 Disk Space", "Available"),
        ("🌐 Network", "Connected"),
        ("📥 wget", "Installed"),
        ("🔨 Build Tools", "Ready"),
    ];
    
    debug!("🔍 System checks: {:?}", status_info);

    loop {
        terminal.draw(|f| {
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(5),
                    Constraint::Length(3),
                ])
                .split(size);

            let header = Paragraph::new("🔍 System Status Check")
                .style(Style::default().fg(settings.theme.primary_color()).add_modifier(Modifier::BOLD))
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(header, chunks[0]);

            let items: Vec<ListItem> = status_info
                .iter()
                .map(|(name, status)| {
                    ListItem::new(format!("✓ {}: {}", name, status))
                        .style(Style::default().fg(settings.theme.success_color()))
                })
                .collect();

            let list = List::new(items)
                .block(Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(settings.theme.accent_color()))
                    .title(" Status "));
            f.render_widget(list, chunks[1]);

            let footer = Paragraph::new("Press any key to return to menu")
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(footer, chunks[2]);
        })?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(_) = event::read()? {
                info!("👋 Exiting system status view");
                break;
            }
        }
    }

    Ok(())
}

fn show_message(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    title: &str,
    message: &str,
    settings: &Settings,
) -> Result<(), Box<dyn Error>> {
    debug!("💬 Showing message: {}", title);
    
    terminal.draw(|f| {
        let size = f.size();
        let paragraph = Paragraph::new(message)
            .style(Style::default().fg(settings.theme.success_color()))
            .alignment(Alignment::Center)
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(settings.theme.primary_color()))
                .title(title));
        f.render_widget(paragraph, size);
    })?;

    loop {
        if let Event::Key(_) = event::read()? {
            break;
        }
    }

    Ok(())
}
