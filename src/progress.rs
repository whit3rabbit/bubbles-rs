use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Model {
    percent: f64,
    width: usize,
    full_char: char,
    empty_char: char,
    show_percentage: bool,
    animated: bool,
    target_percent: f64,
    animation_speed: f64,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            percent: 0.0,
            width: 40,
            full_char: '█',
            empty_char: '░',
            show_percentage: true,
            animated: false,
            target_percent: 0.0,
            animation_speed: 0.1,
        }
    }
}

impl Model {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_width(mut self, width: usize) -> Self {
        self.width = width;
        self
    }

    pub fn with_chars(mut self, full: char, empty: char) -> Self {
        self.full_char = full;
        self.empty_char = empty;
        self
    }

    pub fn with_percentage(mut self, show: bool) -> Self {
        self.show_percentage = show;
        self
    }

    pub fn with_animation(mut self, animated: bool) -> Self {
        self.animated = animated;
        self
    }

    pub fn set_percent(&mut self, percent: f64) {
        let clamped = percent.clamp(0.0, 1.0);
        if self.animated {
            self.target_percent = clamped;
        } else {
            self.percent = clamped;
        }
    }

    pub fn increment(&mut self, amount: f64) {
        self.set_percent(self.percent + amount);
    }

    pub fn view(&self) -> String {
        let filled_width = (self.percent * self.width as f64) as usize;
        let empty_width = self.width - filled_width;

        let mut bar = self.full_char.to_string().repeat(filled_width);
        bar.push_str(&self.empty_char.to_string().repeat(empty_width));

        if self.show_percentage {
            format!("{} {:.0}%", bar, self.percent * 100.0)
        } else {
            bar
        }
    }

    pub fn tick(&mut self) {
        if self.animated && (self.percent - self.target_percent).abs() > 0.001 {
            let diff = self.target_percent - self.percent;
            self.percent += diff * self.animation_speed;
        }
    }

    pub fn tick_interval() -> Duration {
        Duration::from_millis(16) // ~60 FPS
    }
}