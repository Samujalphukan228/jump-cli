// src/ui/layout.rs

use ratatui::layout::Rect;

#[derive(Clone, Copy, Default)]
pub struct JumpLayout {
    pub search: Rect,
    pub list: Rect,
    /// Screen column where typed text begins inside the search field.
    pub search_text_x: u16,
}

impl JumpLayout {
    pub fn cursor_at_search_click(&self, col: u16, row: u16, input_len: usize) -> Option<usize> {
        if self.search.width == 0 || self.search.height == 0 {
            return None;
        }
        if row < self.search.y || row >= self.search.y + self.search.height {
            return None;
        }
        if col < self.search_text_x {
            return Some(0);
        }
        let rel = (col - self.search_text_x) as usize;
        Some(rel.min(input_len))
    }

    pub fn row_at(&self, col: u16, row: u16, scroll_offset: usize) -> Option<usize> {
        if self.list.width == 0 || self.list.height == 0 {
            return None;
        }
        if col < self.list.x
            || col >= self.list.x + self.list.width
            || row < self.list.y
            || row >= self.list.y + self.list.height
        {
            return None;
        }
        Some(scroll_offset + (row - self.list.y) as usize)
    }
}