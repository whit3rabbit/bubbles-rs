use crate::viewport::Model as Viewport;

#[derive(Debug, Clone)]
pub struct Column {
    pub title: String,
    pub width: Option<usize>,
}

impl Column {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            width: None,
        }
    }

    pub fn with_width(mut self, width: usize) -> Self {
        self.width = Some(width);
        self
    }
}

#[derive(Debug, Clone)]
pub struct Row {
    pub cells: Vec<String>,
}

impl Row {
    pub fn new(cells: Vec<String>) -> Self {
        Self { cells }
    }
}

#[derive(Debug, Clone)]
pub struct Model {
    columns: Vec<Column>,
    rows: Vec<Row>,
    selected: usize,
    viewport: Viewport,
}

impl Model {
    pub fn new(columns: Vec<Column>) -> Self {
        Self {
            columns,
            rows: Vec::new(),
            selected: 0,
            viewport: Viewport::new(),
        }
    }

    pub fn with_rows(mut self, rows: Vec<Row>) -> Self {
        self.rows = rows;
        self
    }

    pub fn add_row(&mut self, row: Row) {
        self.rows.push(row);
    }

    pub fn selected_row(&self) -> Option<&Row> {
        self.rows.get(self.selected)
    }

    pub fn select_next(&mut self) {
        if !self.rows.is_empty() {
            self.selected = (self.selected + 1) % self.rows.len();
        }
    }

    pub fn select_prev(&mut self) {
        if !self.rows.is_empty() {
            self.selected = if self.selected == 0 {
                self.rows.len() - 1
            } else {
                self.selected - 1
            };
        }
    }

    pub fn view(&self) -> String {
        let mut output = String::new();

        // Header
        for (i, column) in self.columns.iter().enumerate() {
            if i > 0 {
                output.push_str(" | ");
            }
            output.push_str(&column.title);
        }
        output.push('\n');

        // Separator
        for (i, column) in self.columns.iter().enumerate() {
            if i > 0 {
                output.push_str("-+-");
            }
            let width = column.width.unwrap_or(column.title.len());
            output.push_str(&"-".repeat(width));
        }
        output.push('\n');

        // Rows
        for (row_index, row) in self.rows.iter().enumerate() {
            let selected = row_index == self.selected;
            if selected {
                output.push_str("> ");
            } else {
                output.push_str("  ");
            }

            for (i, cell) in row.cells.iter().enumerate() {
                if i > 0 {
                    output.push_str(" | ");
                }
                output.push_str(cell);
            }
            output.push('\n');
        }

        output
    }
}