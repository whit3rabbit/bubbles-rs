use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct MemoizedWrap {
    cache: HashMap<(String, usize), Vec<String>>,
}

impl MemoizedWrap {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    pub fn wrap_text(&mut self, text: &str, width: usize) -> Vec<String> {
        let key = (text.to_string(), width);
        
        if let Some(cached) = self.cache.get(&key) {
            return cached.clone();
        }

        let wrapped = self.do_wrap(text, width);
        self.cache.insert(key, wrapped.clone());
        wrapped
    }

    fn do_wrap(&self, text: &str, width: usize) -> Vec<String> {
        if width == 0 {
            return vec![text.to_string()];
        }

        let mut result = Vec::new();
        let mut current_line = String::new();
        let mut current_width = 0;

        for ch in text.chars() {
            let char_width = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
            
            if current_width + char_width > width && !current_line.is_empty() {
                result.push(current_line);
                current_line = String::new();
                current_width = 0;
            }

            current_line.push(ch);
            current_width += char_width;
        }

        if !current_line.is_empty() {
            result.push(current_line);
        }

        if result.is_empty() {
            result.push(String::new());
        }

        result
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

impl Default for MemoizedWrap {
    fn default() -> Self {
        Self::new()
    }
}