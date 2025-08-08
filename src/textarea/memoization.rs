//! Memoization utilities for the textarea component.
//!
//! This module contains a simple, hash-keyed cache for memoizing soft-wrapped
//! lines. The `MemoizedWrap` struct provides the wrapping entrypoint used by
//! `textarea::Model` to compute visual sub-lines for a given logical line and
//! width. Results are cached to avoid repeated recomputation while navigating.
//!
//! The generic `MemoCache` is a small LRU cache, provided for parity with the
//! upstream design. The current `MemoizedWrap` uses an internal `HashMap` and
//! fixed capacity to keep the implementation straightforward.

use std::collections::{HashMap, VecDeque};
use std::hash::Hash;
use std::marker::PhantomData;
use unicode_width::UnicodeWidthChar;

/// A trait for objects that can provide a hash for memoization.
/// This matches the Go Hasher interface.
pub trait Hasher {
    /// Returns a stable hash key representing the object's state for caching.
    fn hash_key(&self) -> String;
}

/// Entry in the memoization cache
#[derive(Debug, Clone)]
struct CacheEntry<T> {
    value: T,
    access_order: usize,
}

/// Generic memoization cache with LRU eviction policy.
/// This is a direct port of Go's MemoCache with generics.
#[derive(Debug)]
pub struct MemoCache<H, T>
where
    H: Hasher + Clone,
    T: Clone,
{
    capacity: usize,
    cache: HashMap<String, CacheEntry<T>>,
    access_counter: usize,
    eviction_queue: VecDeque<String>,
    _phantom: PhantomData<H>,
}

impl<H, T> MemoCache<H, T>
where
    H: Hasher + Clone,
    T: Clone,
{
    /// Create a new memoization cache with the given capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            cache: HashMap::new(),
            access_counter: 0,
            eviction_queue: VecDeque::new(),
            _phantom: PhantomData,
        }
    }

    /// Get the capacity of the cache
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Get the current size of the cache
    pub fn size(&self) -> usize {
        self.cache.len()
    }

    /// Get a value from the cache
    pub fn get(&mut self, hashable: &H) -> Option<T> {
        let key = hashable.hash_key();

        if let Some(entry) = self.cache.get_mut(&key) {
            // Update access order for LRU
            self.access_counter += 1;
            entry.access_order = self.access_counter;

            // Move to front in eviction queue
            if let Some(pos) = self.eviction_queue.iter().position(|x| x == &key) {
                self.eviction_queue.remove(pos);
            }
            self.eviction_queue.push_front(key);

            Some(entry.value.clone())
        } else {
            None
        }
    }

    /// Set a value in the cache
    pub fn set(&mut self, hashable: &H, value: T) {
        let key = hashable.hash_key();

        // If key already exists, update it
        if self.cache.contains_key(&key) {
            self.access_counter += 1;
            self.cache.insert(
                key.clone(),
                CacheEntry {
                    value,
                    access_order: self.access_counter,
                },
            );

            // Move to front
            if let Some(pos) = self.eviction_queue.iter().position(|x| x == &key) {
                self.eviction_queue.remove(pos);
            }
            self.eviction_queue.push_front(key);
            return;
        }

        // Check if we need to evict
        if self.cache.len() >= self.capacity {
            self.evict_lru();
        }

        // Add new entry
        self.access_counter += 1;
        self.cache.insert(
            key.clone(),
            CacheEntry {
                value,
                access_order: self.access_counter,
            },
        );
        self.eviction_queue.push_front(key);
    }

    /// Evict the least recently used item
    fn evict_lru(&mut self) {
        if let Some(lru_key) = self.eviction_queue.pop_back() {
            self.cache.remove(&lru_key);
        }
    }

    /// Clear the cache
    pub fn clear(&mut self) {
        self.cache.clear();
        self.eviction_queue.clear();
        self.access_counter = 0;
    }
}

/// Line input for text wrapping, implementing Hasher trait
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Line {
    /// Characters comprising the line to wrap.
    pub runes: Vec<char>,
    /// Target wrap width in columns.
    pub width: usize,
}

impl Hasher for Line {
    fn hash_key(&self) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher as StdHasher};

        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}

/// Memoized text wrapping functionality
#[derive(Debug)]
pub struct MemoizedWrap {
    cache: HashMap<String, Vec<Vec<char>>>,
}

impl MemoizedWrap {
    /// Create a new memoized wrapper with default capacity
    pub fn new() -> Self {
        Self::with_capacity(10000) // Match Go's maxLines constant
    }

    /// Create a new memoized wrapper with specified capacity
    pub fn with_capacity(_capacity: usize) -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    /// Get memoized wrapped text
    pub fn wrap(&mut self, runes: &[char], width: usize) -> Vec<Vec<char>> {
        let line = Line {
            runes: runes.to_vec(),
            width,
        };

        let key = line.hash_key();
        if let Some(cached) = self.cache.get(&key) {
            return cached.clone();
        }

        let wrapped = self.do_wrap(runes, width);
        self.cache.insert(key, wrapped.clone());
        wrapped
    }

    /// Wrap text lines (port of Go wrap function)
    fn do_wrap(&self, runes: &[char], width: usize) -> Vec<Vec<char>> {
        let mut lines = vec![Vec::new()];
        let mut word = Vec::new();
        let mut row = 0;
        let mut spaces = 0;

        // Word wrap the runes
        for &r in runes {
            if r.is_whitespace() {
                spaces += 1;
            } else {
                word.push(r);
            }

            if spaces > 0 {
                let current_line_width = self.line_width(&lines[row]);
                let word_width = self.line_width(&word);

                if current_line_width + word_width + spaces > width {
                    row += 1;
                    lines.push(Vec::new());
                    lines[row].extend_from_slice(&word);
                    lines[row].extend(std::iter::repeat_n(' ', spaces));
                    spaces = 0;
                    word.clear();
                } else {
                    lines[row].extend_from_slice(&word);
                    lines[row].extend(std::iter::repeat_n(' ', spaces));
                    spaces = 0;
                    word.clear();
                }
            } else {
                // If the last character is double-width, check if we can add it
                let last_char_width = word
                    .last()
                    .map(|&ch| UnicodeWidthChar::width(ch).unwrap_or(0))
                    .unwrap_or(0);
                let word_width = self.line_width(&word);

                if word_width + last_char_width > width {
                    // Move to next line if current line has content
                    if !lines[row].is_empty() {
                        row += 1;
                        lines.push(Vec::new());
                    }
                    lines[row].extend_from_slice(&word);
                    word.clear();
                }
            }
        }

        // Handle remaining word and spaces
        let current_line_width = self.line_width(&lines[row]);
        let word_width = self.line_width(&word);

        if current_line_width + word_width + spaces >= width {
            lines.push(Vec::new());
            lines[row + 1].extend_from_slice(&word);
            // Add trailing space for soft-wrapped lines
            spaces += 1;
            lines[row + 1].extend(std::iter::repeat_n(' ', spaces));
        } else {
            lines[row].extend_from_slice(&word);
            spaces += 1;
            lines[row].extend(std::iter::repeat_n(' ', spaces));
        }

        lines
    }

    /// Calculate the display width of a line
    fn line_width(&self, line: &[char]) -> usize {
        line.iter()
            .map(|&ch| UnicodeWidthChar::width(ch).unwrap_or(0))
            .sum()
    }

    /// Clear the memoization cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Get cache capacity
    pub fn capacity(&self) -> usize {
        10000 // Fixed capacity
    }

    /// Get current cache size
    pub fn size(&self) -> usize {
        self.cache.len()
    }
}

impl Default for MemoizedWrap {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for MemoizedWrap {
    fn clone(&self) -> Self {
        // Create a new cache since MemoCache doesn't implement Clone
        Self::with_capacity(self.capacity())
    }
}
