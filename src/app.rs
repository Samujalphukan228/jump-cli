// src/app.rs

use crate::store::VisitStats;
use crate::ui::layout::JumpLayout;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Default)]
pub struct SearchResults {
    pub query: String,
    pub matches: Vec<PathBuf>,
    pub local_count: usize,
    pub searching: bool,
    pub tick: u64,
}

pub struct App {
    pub input: String,
    pub cursor: usize,
    pub selected: usize,
    pub scroll_offset: usize,
    pub results: SearchResults,
    pub pins: HashMap<String, String>,
    pub frecency: HashMap<String, VisitStats>,
    pub layout: JumpLayout,
}

impl App {
    pub fn new(
        initial_query: &str,
        pins: HashMap<String, String>,
        frecency: HashMap<String, VisitStats>,
    ) -> Self {
        Self {
            input: initial_query.to_string(),
            cursor: initial_query.len(),
            selected: 0,
            scroll_offset: 0,
            results: SearchResults::default(),
            pins,
            frecency,
            layout: JumpLayout::default(),
        }
    }

    pub fn visible_count(&self) -> usize {
        self.results.matches.len()
    }

    pub fn clamp_selected(&mut self) {
        let n = self.visible_count();
        if n == 0 {
            self.selected = 0;
            self.scroll_offset = 0;
        } else if self.selected >= n {
            self.selected = n - 1;
        }
    }

    pub fn move_up(&mut self) {
        let n = self.visible_count();
        if n == 0 { return; }
        if self.selected == 0 { self.selected = n - 1; } else { self.selected -= 1; }
    }

    pub fn move_down(&mut self) {
        let n = self.visible_count();
        if n == 0 { return; }
        self.selected = (self.selected + 1) % n;
    }

    pub fn page_up(&mut self, size: usize) {
        self.selected = self.selected.saturating_sub(size);
    }

    pub fn page_down(&mut self, size: usize) {
        let n = self.visible_count();
        if n == 0 { return; }
        self.selected = (self.selected + size).min(n - 1);
    }

    pub fn selected_path(&self) -> Option<&PathBuf> {
        let m = &self.results.matches;
        if m.is_empty() { None } else { Some(&m[self.selected.min(m.len() - 1)]) }
    }

    pub fn matching_pins(&self) -> Vec<(&String, &String)> {
        if self.input.is_empty() {
            let mut v: Vec<_> = self.pins.iter().collect();
            v.sort_by_key(|(k, _)| k.as_str());
            return v;
        }
        let q = self.input.to_lowercase();
        let mut v: Vec<_> = self.pins.iter()
            .filter(|(k, v)| k.to_lowercase().contains(&q) || v.to_lowercase().contains(&q))
            .collect();
        v.sort_by_key(|(k, _)| k.as_str());
        v
    }

    pub fn frecency_score(&self, path: &str) -> f64 {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
        match self.frecency.get(path) {
            None => 0.0,
            Some(s) => {
                let age = (now.saturating_sub(s.last)) as f64 / 86400.0;
                let decay = if age < 1.0 { 4.0 } else if age < 7.0 { 2.0 }
                    else if age < 30.0 { 0.5 } else { 0.25 };
                s.visits as f64 * decay
            }
        }
    }

    pub fn adjust_scroll(&mut self, h: usize) {
        if h == 0 { return; }
        if self.selected < self.scroll_offset { self.scroll_offset = self.selected; }
        else if self.selected >= self.scroll_offset + h { self.scroll_offset = self.selected - h + 1; }
    }

    pub fn clear_input(&mut self) {
        self.input.clear();
        self.cursor = 0;
        self.selected = 0;
        self.scroll_offset = 0;
    }

    pub fn delete_word(&mut self) {
        while self.cursor > 0 {
            let last = self.input[..self.cursor].chars().last().unwrap_or(' ');
            self.cursor -= last.len_utf8();
            self.input.remove(self.cursor);
            if matches!(last, ' ' | '/' | '-' | '_') { break; }
        }
        self.selected = 0;
        self.scroll_offset = 0;
    }

    pub fn backspace(&mut self) {
        if self.cursor > 0 {
            let ch = self.input[..self.cursor].chars().last().unwrap();
            self.cursor -= ch.len_utf8();
            self.input.remove(self.cursor);
            self.selected = 0;
            self.scroll_offset = 0;
        }
    }

    pub fn delete_char(&mut self) {
        if self.cursor < self.input.len() {
            self.input.remove(self.cursor);
            self.selected = 0;
            self.scroll_offset = 0;
        }
    }

    pub fn cursor_left(&mut self) {
        if self.cursor > 0 {
            let ch = self.input[..self.cursor].chars().last().unwrap();
            self.cursor -= ch.len_utf8();
        }
    }

    pub fn cursor_right(&mut self) {
        if self.cursor < self.input.len() {
            let ch = self.input[self.cursor..].chars().next().unwrap();
            self.cursor += ch.len_utf8();
        }
    }

    pub fn insert_char(&mut self, c: char) {
        self.input.insert(self.cursor, c);
        self.cursor += c.len_utf8();
        self.selected = 0;
        self.scroll_offset = 0;
    }
}