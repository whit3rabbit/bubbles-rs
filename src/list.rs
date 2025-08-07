use std::fmt::Display;

pub trait Item: Display + Clone {
    fn filter_value(&self) -> String;
}

pub trait ItemDelegate<T: Item> {
    fn render(&self, item: &T, selected: bool) -> String;
    fn height(&self) -> usize {
        1
    }
    fn spacing(&self) -> usize {
        0
    }
}

#[derive(Debug, Clone)]
pub struct Model<T: Item> {
    items: Vec<T>,
    selected: usize,
    filter: String,
    filtered_items: Vec<(usize, T)>,
    show_filter: bool,
}

impl<T: Item> Model<T> {
    pub fn new(items: Vec<T>) -> Self {
        let filtered_items = items.iter().enumerate().map(|(i, item)| (i, item.clone())).collect();
        Self {
            items,
            selected: 0,
            filter: String::new(),
            filtered_items,
            show_filter: false,
        }
    }

    pub fn view<D: ItemDelegate<T>>(&self, delegate: &D) -> String {
        let mut output = String::new();
        
        if self.show_filter {
            output.push_str(&format!("Filter: {}\n", self.filter));
        }

        for (display_index, (_, item)) in self.filtered_items.iter().enumerate() {
            let selected = display_index == self.selected;
            output.push_str(&delegate.render(item, selected));
            output.push('\n');
        }

        output
    }

    pub fn selected_item(&self) -> Option<&T> {
        self.filtered_items.get(self.selected).map(|(_, item)| item)
    }

    pub fn select_next(&mut self) {
        if !self.filtered_items.is_empty() {
            self.selected = (self.selected + 1) % self.filtered_items.len();
        }
    }

    pub fn select_prev(&mut self) {
        if !self.filtered_items.is_empty() {
            self.selected = if self.selected == 0 {
                self.filtered_items.len() - 1
            } else {
                self.selected - 1
            };
        }
    }
}