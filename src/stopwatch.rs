use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct Model {
    start_time: Option<Instant>,
    elapsed: Duration,
    running: bool,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            start_time: None,
            elapsed: Duration::ZERO,
            running: false,
        }
    }
}

impl Model {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn start(&mut self) {
        if !self.running {
            self.start_time = Some(Instant::now());
            self.running = true;
        }
    }

    pub fn stop(&mut self) {
        if self.running {
            if let Some(start) = self.start_time {
                self.elapsed += start.elapsed();
            }
            self.running = false;
            self.start_time = None;
        }
    }

    pub fn reset(&mut self) {
        self.start_time = None;
        self.elapsed = Duration::ZERO;
        self.running = false;
    }

    pub fn toggle(&mut self) {
        if self.running {
            self.stop();
        } else {
            self.start();
        }
    }

    pub fn elapsed(&self) -> Duration {
        let current_elapsed = if let Some(start) = self.start_time {
            start.elapsed()
        } else {
            Duration::ZERO
        };
        self.elapsed + current_elapsed
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn view(&self) -> String {
        let total_elapsed = self.elapsed();
        let seconds = total_elapsed.as_secs();
        let millis = total_elapsed.subsec_millis();
        
        let hours = seconds / 3600;
        let minutes = (seconds % 3600) / 60;
        let secs = seconds % 60;

        if hours > 0 {
            format!("{:02}:{:02}:{:02}.{:03}", hours, minutes, secs, millis)
        } else {
            format!("{:02}:{:02}.{:03}", minutes, secs, millis)
        }
    }
}