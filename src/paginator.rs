#[derive(Debug, Clone)]
pub struct Model {
    page: usize,
    per_page: usize,
    total_pages: usize,
    total_items: usize,
}

impl Model {
    pub fn new() -> Self {
        Self {
            page: 0,
            per_page: 10,
            total_pages: 0,
            total_items: 0,
        }
    }

    pub fn with_per_page(mut self, per_page: usize) -> Self {
        self.per_page = per_page;
        self.update_total_pages();
        self
    }

    pub fn set_total_items(&mut self, total: usize) {
        self.total_items = total;
        self.update_total_pages();
    }

    pub fn next_page(&mut self) {
        if self.page < self.total_pages.saturating_sub(1) {
            self.page += 1;
        }
    }

    pub fn prev_page(&mut self) {
        if self.page > 0 {
            self.page -= 1;
        }
    }

    pub fn first_page(&mut self) {
        self.page = 0;
    }

    pub fn last_page(&mut self) {
        self.page = self.total_pages.saturating_sub(1);
    }

    pub fn current_page(&self) -> usize {
        self.page
    }

    pub fn total_pages(&self) -> usize {
        self.total_pages
    }

    pub fn start_index(&self) -> usize {
        self.page * self.per_page
    }

    pub fn end_index(&self) -> usize {
        std::cmp::min(self.start_index() + self.per_page, self.total_items)
    }

    pub fn view(&self) -> String {
        if self.total_pages <= 1 {
            return String::new();
        }
        format!("Page {} of {}", self.page + 1, self.total_pages)
    }

    fn update_total_pages(&mut self) {
        self.total_pages = if self.per_page > 0 {
            (self.total_items + self.per_page - 1) / self.per_page
        } else {
            0
        };
        if self.page >= self.total_pages && self.total_pages > 0 {
            self.page = self.total_pages - 1;
        }
    }
}

impl Default for Model {
    fn default() -> Self {
        Self::new()
    }
}