use rsille::canvas::Canvas;
use std::time::Duration;

pub trait Animation {
    fn update(&mut self, delta: Duration);
    fn render(&self, canvas: &mut Canvas);
    fn is_finished(&self) -> bool;
}

pub trait ProgressAnimation: Animation {
    fn set_progress(&mut self, progress: f64);
    fn get_progress(&self) -> f64;
}
