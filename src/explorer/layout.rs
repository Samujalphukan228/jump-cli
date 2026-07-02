// src/explorer/layout.rs

use ratatui::layout::Rect;

#[derive(Clone, Copy, Default)]
pub struct ExplorerLayout {
    pub file_list: Rect,
    pub row_height: u16,
}

impl ExplorerLayout {
    pub fn row_at(&self, col: u16, row: u16, scroll_offset: usize) -> Option<usize> {
        if self.file_list.width == 0 || self.file_list.height == 0 || self.row_height == 0 {
            return None;
        }
        if col < self.file_list.x
            || col >= self.file_list.x + self.file_list.width
            || row < self.file_list.y
            || row >= self.file_list.y + self.file_list.height
        {
            return None;
        }
        let rel = (row - self.file_list.y) / self.row_height;
        Some(scroll_offset + rel as usize)
    }
}