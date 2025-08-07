use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Spinner {
    frames: Vec<String>,
    fps: u64,
}

impl Spinner {
    pub fn line() -> Self {
        Self {
            frames: vec![
                "|".to_string(),
                "/".to_string(),
                "-".to_string(),
                "\\".to_string(),
            ],
            fps: 9,
        }
    }

    pub fn dot() -> Self {
        Self {
            frames: vec![
                "⣾".to_string(),
                "⣽".to_string(),
                "⣻".to_string(),
                "⢿".to_string(),
                "⡿".to_string(),
                "⣟".to_string(),
                "⣯".to_string(),
                "⣷".to_string(),
            ],
            fps: 10,
        }
    }

    pub fn custom(frames: Vec<String>, fps: u64) -> Self {
        Self { frames, fps }
    }
}

#[derive(Debug, Clone)]
pub struct Model {
    spinner: Spinner,
    frame: usize,
    style: String,
}

impl Model {
    pub fn new(spinner: Spinner) -> Self {
        Self {
            spinner,
            frame: 0,
            style: String::new(),
        }
    }

    pub fn with_style(mut self, style: String) -> Self {
        self.style = style;
        self
    }

    pub fn tick(&mut self) {
        self.frame = (self.frame + 1) % self.spinner.frames.len();
    }

    pub fn view(&self) -> String {
        self.spinner.frames[self.frame].clone()
    }

    pub fn tick_interval(&self) -> Duration {
        Duration::from_millis(1000 / self.spinner.fps)
    }
}

impl Default for Model {
    fn default() -> Self {
        Self::new(Spinner::line())
    }
}