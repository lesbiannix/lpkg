use std::time::Duration;
use rsille::canvas::Canvas;
use super::Animation;

const THETA_SPACING: f64 = 0.07;
const PHI_SPACING: f64 = 0.02;

pub struct DonutAnimation {
    a: f64, // rotation around X
    b: f64, // rotation around Z
    size: (u16, u16),
}

impl DonutAnimation {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            a: 0.0,
            b: 0.0,
            size: (width, height),
        }
    }
}

impl Animation for DonutAnimation {
    fn update(&mut self, delta: Duration) {
        let delta_secs = delta.as_secs_f64();
        self.a += delta_secs;
        self.b += delta_secs * 0.5;
    }

    fn render(&self, canvas: &mut Canvas) {
        let (width, height) = self.size;
        let (width_f, height_f) = (width as f64, height as f64);
        let k2 = 5.0;
        let k1 = width_f * k2 * 3.0 / (8.0 * (height_f + width_f));
        
        for theta in 0..((2.0 * std::f64::consts::PI / THETA_SPACING) as i32) {
            let theta_f = theta as f64 * THETA_SPACING;
            let cos_theta = theta_f.cos();
            let sin_theta = theta_f.sin();

            for phi in 0..((2.0 * std::f64::consts::PI / PHI_SPACING) as i32) {
                let phi_f = phi as f64 * PHI_SPACING;
                let cos_phi = phi_f.cos();
                let sin_phi = phi_f.sin();

                let cos_a = self.a.cos();
                let sin_a = self.a.sin();
                let cos_b = self.b.cos();
                let sin_b = self.b.sin();

                let h = cos_theta + 2.0;
                let d = 1.0 / (sin_phi * h * sin_a + sin_theta * cos_a + 5.0);
                let t = sin_phi * h * cos_a - sin_theta * sin_a;

                let x = (width_f / 2.0 + 30.0 * d * (cos_phi * h * cos_b - t * sin_b)) as i32;
                let y = (height_f / 2.0 + 15.0 * d * (cos_phi * h * sin_b + t * cos_b)) as i32;
                let z = (1.0 / d) as u8;

                if x >= 0 && x < width as i32 && y >= 0 && y < height as i32 {
                    let luminance = if z > 0 { z } else { 1 };
                    let c = match luminance {
                        0..=31 => '.',
                        32..=63 => '*',
                        64..=95 => 'o',
                        96..=127 => '&',
                        128..=159 => '8',
                        160..=191 => '#',
                        _ => '@',
                    };
                    canvas.put_char(x as u16, y as u16, c);
                }
            }
        }
    }

    fn is_finished(&self) -> bool {
        false // continuous animation
    }
}