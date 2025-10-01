// src/tui/disk_manager.rs
use std::{
    fs::{File, read_dir},
    io::{self, Seek, SeekFrom},
    path::PathBuf,
};

use crossterm::event::{self, Event, KeyCode};
use crossterm::execute;
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use gptman::{GPT, GPTPartitionEntry, PartitionName};
use tui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, List, ListItem, Paragraph},
};
use uuid::Uuid;

/// DiskManager: interactive TUI to view and create GPT partitions on Linux.
///
/// Requirements (add to Cargo.toml):
/// tui = "0.19"
/// crossterm = "0.26"
/// gptman = "2.0"
/// uuid = { version = "1", features = ["v4"] }
pub struct DiskManager;

impl DiskManager {
    /// Entrypoint: run the disk manager UI. This initializes the terminal and starts the loop.
    pub fn run_tui() -> Result<(), Box<dyn std::error::Error>> {
        // init terminal
        let mut stdout = std::io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut term = Terminal::new(backend)?;
        term.clear()?;

        // collect devices (linux-focused: sd*, nvme*, vd*)
        let mut devices: Vec<PathBuf> = Vec::new();
        if let Ok(entries) = read_dir("/dev/") {
            for e in entries.flatten() {
                let path = e.path();
                if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                    if name.starts_with("sd")
                        || name.starts_with("nvme")
                        || name.starts_with("vd")
                        || name.starts_with("mmcblk")
                    {
                        devices.push(path);
                    }
                }
            }
        }

        if devices.is_empty() {
            // restore terminal before printing
            execute!(term.backend_mut(), LeaveAlternateScreen)?;
            println!("No block devices found under /dev (sd*, nvme*, vd*, mmcblk*).");
            return Ok(());
        }

        let mut selected_idx = 0usize;
        let mut status_msg =
            String::from("Select disk. ↑/↓ to navigate, Enter=view, C=create, Q=quit.");

        loop {
            term.draw(|f| {
                let size = f.size();

                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(1)
                    .constraints(
                        [
                            Constraint::Length(3),
                            Constraint::Min(6),
                            Constraint::Length(3),
                        ]
                        .as_ref(),
                    )
                    .split(size);

                // header
                let header = Paragraph::new(Span::styled(
                    "🔧 Disk Manager — Linux GPT (use carefully!)",
                    Style::default().add_modifier(Modifier::BOLD),
                ))
                .block(Block::default().borders(Borders::ALL));
                f.render_widget(header, chunks[0]);

                // device list + selection
                let items: Vec<ListItem> = devices
                    .iter()
                    .enumerate()
                    .map(|(i, d)| {
                        let label = format!(
                            "{} {}",
                            if i == selected_idx { "▶" } else { " " },
                            d.display()
                        );
                        let mut li = ListItem::new(label);
                        if i == selected_idx {
                            li = li.style(Style::default().fg(Color::Yellow));
                        }
                        li
                    })
                    .collect();

                let list =
                    List::new(items).block(Block::default().borders(Borders::ALL).title("Disks"));
                f.render_widget(list, chunks[1]);

                // status/footer
                let footer = Paragraph::new(status_msg.as_str())
                    .style(Style::default().fg(Color::Green))
                    .block(Block::default().borders(Borders::ALL).title("Status"));
                f.render_widget(footer, chunks[2]);
            })?;

            // Input handling
            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Char('Q') => break,
                        KeyCode::Up => {
                            if selected_idx > 0 {
                                selected_idx -= 1;
                            }
                        }
                        KeyCode::Down => {
                            if selected_idx + 1 < devices.len() {
                                selected_idx += 1;
                            }
                        }
                        KeyCode::Enter => {
                            let path = devices[selected_idx].clone();
                            match Self::view_partitions_tui(&path, &mut term) {
                                Ok(m) => status_msg = m,
                                Err(e) => status_msg = format!("Error reading partitions: {}", e),
                            }
                        }
                        KeyCode::Char('c') | KeyCode::Char('C') => {
                            let path = devices[selected_idx].clone();
                            match Self::create_partition_tui(&path, &mut term) {
                                Ok(m) => {
                                    println!("[disk-manager] {}", m);
                                    status_msg = m;
                                }
                                Err(e) => {
                                    eprintln!("[disk-manager] create partition error: {e}");
                                    status_msg = format!("Create failed: {}", e);
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        // restore terminal
        execute!(term.backend_mut(), LeaveAlternateScreen)?;
        term.show_cursor()?;
        Ok(())
    }

    /// Show GPT partitions for the chosen disk in a paged TUI view.
    fn view_partitions_tui(
        disk: &PathBuf,
        term: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // try to open & read GPT (512 sector size)
        let mut file = File::open(disk)?;
        let gpt = match GPT::read_from(&mut file, 512) {
            Ok(g) => g,
            Err(_) => match GPT::find_from(&mut file) {
                Ok(g) => g,
                Err(_) => {
                    return Ok(format!("No GPT found on {}", disk.display()));
                }
            },
        };

        // Create list of lines to display using public GPT API:
        let mut lines: Vec<String> = Vec::new();
        lines.push(format!("Partitions on {}:", disk.display()));
        for (i, entry) in gpt.iter() {
            if entry.is_used() {
                let name = entry.partition_name.as_str();
                lines.push(format!(
                    "{}: {} -> {}  (type: {})",
                    i,
                    entry.starting_lba,
                    entry.ending_lba,
                    // show a short GUID hex for partition type
                    hex::encode_upper(&entry.partition_type_guid)
                ));
                lines.push(format!("    name: {}", name));
            }
        }
        if lines.len() == 1 {
            lines.push("No partitions found.".into());
        }
        // paged view loop
        let mut top = 0usize;
        loop {
            term.draw(|f| {
                let size = f.size();
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(1)
                    .constraints([Constraint::Min(1), Constraint::Length(1)].as_ref())
                    .split(size);

                let page_lines: Vec<ListItem> = lines
                    .iter()
                    .skip(top)
                    .take((chunks[0].height as usize).saturating_sub(2))
                    .map(|l| ListItem::new(l.clone()))
                    .collect();

                let list = List::new(page_lines).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(format!("Partitions: {}", disk.display())),
                );
                f.render_widget(list, chunks[0]);

                let footer = Paragraph::new("↑/↓ scroll  •  q to go back")
                    .block(Block::default().borders(Borders::ALL));
                f.render_widget(footer, chunks[1]);
            })?;

            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(k) = event::read()? {
                    match k.code {
                        KeyCode::Char('q') | KeyCode::Esc => break,
                        KeyCode::Up => {
                            if top > 0 {
                                top = top.saturating_sub(1);
                            }
                        }
                        KeyCode::Down => {
                            if top + 1 < lines.len() {
                                top = top.saturating_add(1);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(format!("Viewed partitions on {}", disk.display()))
    }

    /// Fully-TUI flow to enter partition name, size (MB), and choose partition type.
    /// Writes GPT changes to disk.
    fn create_partition_tui(
        disk: &PathBuf,
        term: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // open file read+write
        let mut file = File::options().read(true).write(true).open(disk)?;

        // Read or create GPT
        let mut gpt = match GPT::read_from(&mut file, 512) {
            Ok(g) => g,
            Err(_) => match GPT::find_from(&mut file) {
                Ok(g) => g,
                Err(_) => {
                    // If there's no GPT, create one with a random disk GUID
                    let disk_guid_raw: [u8; 16] = *Uuid::new_v4().as_bytes();
                    // new_from requires a Seek+Read; create new GPT structure on-disk
                    GPT::new_from(&mut file, 512, disk_guid_raw)?
                }
            },
        };

        // interactive fields
        let mut name = String::from("new_partition");
        let mut size_mb: u64 = 100; // default 100 MB
        let mut type_choice = 1usize; // 0 = EFI, 1 = Linux filesystem

        // known GUIDs (string repr) -> will be parsed to raw bytes as required
        let efi_guid = Uuid::parse_str("C12A7328-F81F-11D2-BA4B-00A0C93EC93B")?; // EFI System
        let linux_fs_guid = Uuid::parse_str("0FC63DAF-8483-4772-8E79-3D69D8477DE4")?; // Linux filesystem

        loop {
            // Render UI
            term.draw(|f| {
                let size = f.size();
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(1)
                    .constraints(
                        [
                            Constraint::Length(3),
                            Constraint::Length(3),
                            Constraint::Length(3),
                            Constraint::Min(3),
                        ]
                        .as_ref(),
                    )
                    .split(size);

                let title = Paragraph::new(Span::styled(
                    format!("Create partition on {}", disk.display()),
                    Style::default().add_modifier(Modifier::BOLD),
                ))
                .block(Block::default().borders(Borders::ALL));
                f.render_widget(title, chunks[0]);

                let name_widget = Paragraph::new(format!("Name: {}", name)).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Partition Name"),
                );
                f.render_widget(name_widget, chunks[1]);

                let size_widget = Paragraph::new(format!("Size (MB): {}", size_mb))
                    .block(Block::default().borders(Borders::ALL).title("Size"));
                f.render_widget(size_widget, chunks[2]);

                let types = vec![
                    format!(
                        "{} EFI System Partition",
                        if type_choice == 0 { "▶" } else { " " }
                    ),
                    format!(
                        "{} Linux filesystem",
                        if type_choice == 1 { "▶" } else { " " }
                    ),
                ];
                let type_items: Vec<ListItem> = types.into_iter().map(ListItem::new).collect();
                let type_list = List::new(type_items).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Partition Type (use ←/→)"),
                );
                f.render_widget(type_list, chunks[3]);
            })?;

            // Input
            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(k) = event::read()? {
                    match k.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            return Ok("Creation cancelled".to_string());
                        }
                        KeyCode::Left => {
                            type_choice = type_choice.saturating_sub(1);
                        }
                        KeyCode::Right => {
                            type_choice = (type_choice + 1) % 2;
                        }
                        KeyCode::Char(c) => {
                            // typing to name: accept visible characters; digits typed also append
                            if !c.is_control() {
                                name.push(c);
                            }
                        }
                        KeyCode::Backspace => {
                            name.pop();
                        }
                        KeyCode::Up => {
                            // increase size by 10MB
                            size_mb = size_mb.saturating_add(10);
                        }
                        KeyCode::Down => {
                            size_mb = size_mb.saturating_sub(10);
                        }
                        KeyCode::Enter => {
                            break;
                        }
                        _ => {}
                    }
                }
            }
        }

        // convert MB -> sectors (512 bytes per sector)
        let sectors = (size_mb as u128 * 1024 * 1024 / 512) as u64;
        // choose starting LBA: find max ending_lba among existing partitions; align to 2048
        let last_end = gpt
            .iter()
            .filter(|(_, e)| e.is_used())
            .map(|(_, e)| e.ending_lba)
            .max()
            .unwrap_or(2048);
        let start = ((last_end + 2048) / 2048) * 2048 + 1;
        let end = start + sectors.saturating_sub(1);

        // build partition entry
        let mut new_entry = GPTPartitionEntry::empty();
        new_entry.starting_lba = start;
        new_entry.ending_lba = end;
        new_entry.partition_name = PartitionName::from(name.as_str());

        // set partition type GUID
        let type_guid = if type_choice == 0 {
            *efi_guid.as_bytes()
        } else {
            *linux_fs_guid.as_bytes()
        };
        new_entry.partition_type_guid = type_guid;

        // find first empty partition slot (indexing is 1-based for gptman::GPT)
        let idx_opt = gpt.iter().find(|(_, e)| e.is_unused()).map(|(i, _)| i);
        let idx = match idx_opt {
            Some(i) => i,
            None => return Err("No free GPT partition entries (maxed out)".into()),
        };

        // assign and write
        gpt[idx] = new_entry;

        // Seek to start (important)
        file.seek(SeekFrom::Start(0))?;
        gpt.write_into(&mut file)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        Ok(format!(
            "Created partition '{}' on {} ({} MB, sectors {}..{})",
            name,
            disk.display(),
            size_mb,
            start,
            end
        ))
    }
}
