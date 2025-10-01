use std::time::Duration;
use rsille::canvas::Canvas;
use super::{Animation, ProgressAnimation};

pub struct ProgressBarAnimation {
    progress: f64,
    width: u16,
    height: u16,
    animation_offset: f64,
}

impl ProgressBarAnimation {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            progress: 0.0,
            width,
            height,
            animation_offset: 0.0,
        }
    }
}

impl Animation for ProgressBarAnimation {
    fn update(&mut self, delta: Duration) {
        self.animation_offset += delta.as_secs_f64() * 2.0;
        if self.animation_offset >= 1.0 {
            self.animation_offset -= 1.0;
        }
    }

    fn render(&self, canvas: &mut Canvas) {
        // Animated progress bar rendering will be implemented here
    }

    fn is_finished(&self) -> bool {
        self.progress >= 1.0
    }
}

impl ProgressAnimation for ProgressBarAnimation {
    fn set_progress(&mut self, progress: f64) {
        self.progress = progress.clamp(0.0, 1.0);
    }

    fn get_progress(&self) -> f64 {
        self.progress
    }
}