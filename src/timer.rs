use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Model {
    duration: Duration,
    remaining: Duration,
    running: bool,
}

impl Model {
    pub fn new(duration: Duration) -> Self {
        Self {
            duration,
            remaining: duration,
            running: false,
        }
    }

    pub fn start(&mut self) {
        self.running = true;
    }

    pub fn stop(&mut self) {
        self.running = false;
    }

    pub fn reset(&mut self) {
        self.remaining = self.duration;
        self.running = false;
    }

    pub fn toggle(&mut self) {
        self.running = !self.running;
    }

    pub fn tick(&mut self, delta: Duration) {
        if self.running && self.remaining > Duration::ZERO {
            self.remaining = self.remaining.saturating_sub(delta);
            if self.remaining == Duration::ZERO {
                self.running = false;
            }
        }
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn is_finished(&self) -> bool {
        self.remaining == Duration::ZERO
    }

    pub fn remaining(&self) -> Duration {
        self.remaining
    }

    pub fn progress(&self) -> f64 {
        if self.duration == Duration::ZERO {
            return 1.0;
        }
        1.0 - (self.remaining.as_secs_f64() / self.duration.as_secs_f64())
    }

    pub fn view(&self) -> String {
        let total_seconds = self.remaining.as_secs();
        let minutes = total_seconds / 60;
        let seconds = total_seconds % 60;
        
        format!("{:02}:{:02}", minutes, seconds)
    }

    pub fn tick_interval() -> Duration {
        Duration::from_millis(100)
    }
}

impl Default for Model {
    fn default() -> Self {
        Self::new(Duration::from_secs(60))
    }
}