use std::io::Stdout;
use tracing::instrument;
use tui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::Style,
    text::Spans,
    widgets::{Block, Borders, Gauge, List, ListItem},
};

use crate::tui::settings::Settings;

pub struct Downloader;

impl Downloader {
    #[instrument(skip(terminal, settings))]
    pub fn show_downloader(
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
        settings: &Settings,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let files = vec!["file1.tar.gz", "file2.tar.gz", "file3.tar.gz"];
        let progress = vec![0.3, 0.5, 0.9];

        loop {
            terminal.draw(|f| {
                let size = f.size();

                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(2)
                    .constraints([Constraint::Percentage(70), Constraint::Percentage(30)].as_ref())
                    .split(size);

                let items: Vec<ListItem> = files
                    .iter()
                    .map(|f| ListItem::new(Spans::from(*f)))
                    .collect();
                let list = List::new(items).block(
                    Block::default()
                        .title(Spans::from("Downloads"))
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(settings.theme.secondary_color())),
                );
                f.render_widget(list, chunks[0]);

                for (i, prog) in progress.iter().enumerate() {
                    let gauge = Gauge::default()
                        .block(Block::default().title(files[i]))
                        .gauge_style(Style::default().fg(settings.theme.primary_color()))
                        .ratio(*prog as f64);
                    f.render_widget(gauge, chunks[1]);
                }
            })?;

            break; // remove in real async loop
        }

        Ok(())
    }
}
