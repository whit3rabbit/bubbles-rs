#[derive(Debug, Clone)]
pub struct Model {
    content: Vec<String>,
    y_offset: usize,
    width: usize,
    height: usize,
    y_position: f64,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            content: Vec::new(),
            y_offset: 0,
            width: 80,
            height: 24,
            y_position: 0.0,
        }
    }
}

impl Model {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_dimensions(mut self, width: usize, height: usize) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    pub fn set_content(&mut self, content: Vec<String>) {
        self.content = content;
        self.y_offset = 0;
        self.y_position = 0.0;
    }

    pub fn set_content_from_string(&mut self, content: &str) {
        self.set_content(content.lines().map(|s| s.to_string()).collect());
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn y_offset(&self) -> usize {
        self.y_offset
    }

    pub fn scroll_down(&mut self, lines: usize) {
        let max_offset = self.content.len().saturating_sub(self.height);
        self.y_offset = (self.y_offset + lines).min(max_offset);
        self.y_position = self.y_offset as f64;
    }

    pub fn scroll_up(&mut self, lines: usize) {
        self.y_offset = self.y_offset.saturating_sub(lines);
        self.y_position = self.y_offset as f64;
    }

    pub fn scroll_to_top(&mut self) {
        self.y_offset = 0;
        self.y_position = 0.0;
    }

    pub fn scroll_to_bottom(&mut self) {
        let max_offset = self.content.len().saturating_sub(self.height);
        self.y_offset = max_offset;
        self.y_position = max_offset as f64;
    }

    pub fn half_viewport_down(&mut self) {
        self.scroll_down(self.height / 2);
    }

    pub fn half_viewport_up(&mut self) {
        self.scroll_up(self.height / 2);
    }

    pub fn at_top(&self) -> bool {
        self.y_offset == 0
    }

    pub fn at_bottom(&self) -> bool {
        self.y_offset >= self.content.len().saturating_sub(self.height)
    }

    pub fn scroll_percent(&self) -> f64 {
        if self.content.len() <= self.height {
            return 0.0;
        }
        let max_offset = self.content.len() - self.height;
        self.y_offset as f64 / max_offset as f64
    }

    pub fn visible_lines(&self) -> &[String] {
        let start = self.y_offset;
        let end = (start + self.height).min(self.content.len());
        &self.content[start..end]
    }

    pub fn view(&self) -> String {
        let visible = self.visible_lines();
        let mut output = String::new();
        
        for (i, line) in visible.iter().enumerate() {
            if i > 0 {
                output.push('\n');
            }
            
            if line.len() > self.width {
                output.push_str(&line[..self.width]);
            } else {
                output.push_str(line);
                if line.len() < self.width {
                    output.push_str(&" ".repeat(self.width - line.len()));
                }
            }
        }

        // Fill remaining height with empty lines
        let lines_shown = visible.len();
        if lines_shown < self.height {
            for _ in lines_shown..self.height {
                output.push('\n');
                output.push_str(&" ".repeat(self.width));
            }
        }

        output
    }
}