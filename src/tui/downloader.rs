use crate::tui::animations::{
    Animation, ProgressAnimation, donut::DonutAnimation, progress::ProgressBarAnimation,
};
use rsille::canvas::Canvas;
use std::{io::Stdout, thread, time::Duration};
use tui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::Style,
    text::Spans,
    widgets::{Block, Borders, List, ListItem},
};

use crate::tui::settings::Settings;

pub struct Downloader;

const TARGET_FPS: u64 = 30;
const FRAME_TIME: Duration = Duration::from_micros(1_000_000 / TARGET_FPS);

impl Downloader {
    pub fn show_downloader(
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
        settings: &Settings,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let files = vec!["file1.tar.gz", "file2.tar.gz", "file3.tar.gz"];
        let progress = vec![0.3, 0.5, 0.9];

        let mut last_update = std::time::Instant::now();
        loop {
            let frame_start = std::time::Instant::now();
            let delta = frame_start - last_update;
            last_update = frame_start;

            terminal.draw(|f| {
                let size = f.size();

                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(2)
                    .constraints(
                        [
                            Constraint::Percentage(50),
                            Constraint::Percentage(25),
                            Constraint::Percentage(25),
                        ]
                        .as_ref(),
                    )
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

                // Progress bar
                let mut progress_canvas = Canvas::new(chunks[1].width, chunks[1].height);
                let mut progress_bar = ProgressBarAnimation::new(chunks[1].width, chunks[1].height);

                for (i, prog) in progress.iter().enumerate() {
                    progress_bar.set_progress(*prog as f64);
                    progress_bar.render(&mut progress_canvas);
                }

                // Render progress bar
                let progress_block = Block::default()
                    .title(files[0])
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(settings.theme.primary_color()));

                f.render_widget(progress_block, chunks[1]);

                // Donut animation
                let mut donut_canvas = Canvas::new(chunks[2].width, chunks[2].height);
                let mut donut = DonutAnimation::new(chunks[2].width, chunks[2].height);
                donut.render(&mut donut_canvas);

                // Render donut
                let donut_block = Block::default()
                    .title("Progress")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(settings.theme.secondary_color()));

                f.render_widget(donut_block, chunks[2]);
            })?;

            // Frame rate limiting
            let frame_time = frame_start.elapsed();
            if frame_time < FRAME_TIME {
                thread::sleep(FRAME_TIME - frame_time);
            }
        }

        Ok(())
    }
}
