//! View rendering methods for the textinput component.

use super::model::Model;
use super::types::EchoMode;

impl Model {
    /// View renders the textinput in its current state.
    /// Matches Go's View method exactly.
    pub fn view(&self) -> String {
        // Placeholder text
        if self.value.is_empty() && !self.placeholder.is_empty() {
            return self.placeholder_view();
        }

        let value_slice = if self.offset_right <= self.value.len() {
            &self.value[self.offset..self.offset_right]
        } else {
            &self.value[self.offset..]
        };

        let pos = self.pos.saturating_sub(self.offset);
        let value_str: String = value_slice.iter().collect();
        let display_value = self.echo_transform(&value_str);

        let mut v = String::new();

        // Text before cursor
        if pos < display_value.len() {
            v.push_str(&self.text_style.render(&display_value[..pos]));
        } else {
            v.push_str(&self.text_style.render(&display_value));
        }

        // Cursor and text under it
        if pos < display_value.len() {
            let char_at_pos = display_value.chars().nth(pos).unwrap_or(' ');
            let mut cur = self.cursor.clone();
            cur.set_char(&char_at_pos.to_string());
            v.push_str(&cur.view());

            // Text after cursor
            if pos + 1 < display_value.len() {
                v.push_str(&self.text_style.render(&display_value[pos + 1..]));
            }

            v.push_str(&self.completion_view(0));
        } else {
            // Cursor at end
            if self.focus && self.can_accept_suggestion() {
                let suggestion = &self.matched_suggestions[self.current_suggestion_index];
                if self.value.len() < suggestion.len() {
                    let next_char = suggestion[pos];
                    let mut cur = self.cursor.clone();
                    cur.set_char(&next_char.to_string());
                    v.push_str(&cur.view());
                    v.push_str(&self.completion_view(1));
                } else {
                    let mut cur = self.cursor.clone();
                    cur.set_char(" ");
                    v.push_str(&cur.view());
                }
            } else {
                let mut cur = self.cursor.clone();
                cur.set_char(" ");
                v.push_str(&cur.view());
            }
        }

        // Fill remaining width with background
        let val_width = display_value.chars().count();
        if self.width > 0 && val_width <= self.width as usize {
            let padding = (self.width as usize).saturating_sub(val_width);
            if val_width + padding <= self.width as usize && pos < display_value.len() {
                // padding += 1; // Adjust for cursor
            }
            v.push_str(&self.text_style.render(&" ".repeat(padding)));
        }

        format!("{}{}", self.prompt_style.render(&self.prompt), v)
    }

    /// Internal placeholder view rendering
    pub(super) fn placeholder_view(&self) -> String {
        let mut v = String::new();

        let placeholder_chars: Vec<char> = self.placeholder.chars().collect();
        let p = if self.width > 0 {
            let mut p_vec = vec![' '; self.width as usize + 1];
            for (i, &ch) in placeholder_chars.iter().enumerate() {
                if i < p_vec.len() {
                    p_vec[i] = ch;
                }
            }
            p_vec
        } else {
            placeholder_chars
        };

        if !p.is_empty() {
            let mut cur = self.cursor.clone();
            cur.set_char(&p[0].to_string());
            v.push_str(&cur.view());
        }

        if self.width < 1 && p.len() <= 1 {
            return format!("{}{}", self.prompt_style.render(&self.prompt), v);
        }

        if self.width > 0 {
            let min_width = self.placeholder.chars().count();
            let avail_width = (self.width as usize).saturating_sub(min_width) + 1;

            if p.len() > 1 {
                let end_idx = std::cmp::min(p.len(), min_width);
                let text: String = p[1..end_idx].iter().collect();
                v.push_str(&self.placeholder_style.render(&text));
            }
            v.push_str(&self.placeholder_style.render(&" ".repeat(avail_width)));
        } else if p.len() > 1 {
            // Include the first character as well to ensure contiguous placeholder text
            let text: String = p[0..].iter().collect();
            v.push_str(&self.placeholder_style.render(&text));
        }

        format!("{}{}", self.prompt_style.render(&self.prompt), v)
    }

    /// Internal echo transformation
    pub(super) fn echo_transform(&self, v: &str) -> String {
        match self.echo_mode {
            EchoMode::EchoPassword => {
                let width = v.chars().count();
                self.echo_character.to_string().repeat(width)
            }
            EchoMode::EchoNone => String::new(),
            EchoMode::EchoNormal => v.to_string(),
        }
    }

    /// Internal completion view rendering
    pub(super) fn completion_view(&self, offset: usize) -> String {
        if self.can_accept_suggestion() {
            let suggestion = &self.matched_suggestions[self.current_suggestion_index];
            if self.value.len() + offset < suggestion.len() {
                let remaining: String = suggestion[self.value.len() + offset..].iter().collect();
                return self.completion_style.render(&remaining);
            }
        }
        String::new()
    }
}
