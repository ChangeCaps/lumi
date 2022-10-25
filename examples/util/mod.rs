mod camera;
mod framework;

pub use camera::*;
pub use framework::*;

use std::time::Instant;

pub struct FpsCounter {
    last_frame: Instant,
    frame_times: Vec<f32>,
}

impl FpsCounter {
    pub fn new() -> Self {
        Self {
            last_frame: Instant::now(),
            frame_times: Vec::new(),
        }
    }

    pub fn update(&mut self) {
        let now = Instant::now();
        let frame_time = now - self.last_frame;
        self.last_frame = now;

        self.frame_times.push(frame_time.as_secs_f32());
        if self.frame_times.len() > 60 {
            self.frame_times.remove(0);
        }
    }

    pub fn get_fps(&self) -> f32 {
        self.frame_times.len() as f32 / self.frame_times.iter().sum::<f32>()
    }
}
