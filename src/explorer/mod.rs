// src/explorer/mod.rs

mod actions;
pub mod draw;
mod layout;
pub mod state;

use crossterm::event::{self, Event, KeyCode, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

use std::path::PathBuf;
use std::time::{Duration, Instant};

use crate::ui::helpers::{tui_init, tui_restore};
use state::ExplorerState;

pub fn run_explorer(start: &PathBuf) -> std::io::Result<Option<PathBuf>> {
    let mut terminal = tui_init()?;
    let mut state = ExplorerState::new(start);
    state.refresh();

    let mut tick: u64 = 0;
    let mut last_click: Option<(Instant, u16, u16, usize)> = None;

    let result = loop {
        tick += 1;
        state.tick = tick;

        terminal.draw(|f| draw::draw_explorer(f, &mut state))?;

        if event::poll(Duration::from_millis(50))? {
            match event::read()? {
                Event::Mouse(mouse) => {
                    handle_mouse(&mut state, mouse, &mut last_click);
                }
                Event::Key(key) => {

                // ── Live filter mode ──────────────────────────────────────
                if state.filter_active {
                    match key.code {
                        KeyCode::Esc => {
                            state.clear_filter();
                        }
                        KeyCode::Enter => {
                            state.end_filter();
                            if state.entries.len() == 1 {
                                // Auto-select single match
                                state.selected = 0;
                            }
                        }
                        KeyCode::Backspace => {
                            state.filter_pop();
                        }
                        KeyCode::Char(c) => {
                            state.filter_push(c);
                        }
                        KeyCode::Up => {
                            state.move_up();
                        }
                        KeyCode::Down => {
                            state.move_down();
                        }
                        _ => {}
                    }
                    continue;
                }

                // ── Open With menu ────────────────────────────────────────
                if state.show_open_with {
                    match key.code {
                        KeyCode::Esc | KeyCode::Char('q') => { state.show_open_with = false; }
                        KeyCode::Up | KeyCode::Char('k') => {
                            if state.open_with_selected > 0 { state.open_with_selected -= 1; }
                            else { state.open_with_selected = state.openers.len().saturating_sub(1); }
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            if !state.openers.is_empty() { state.open_with_selected = (state.open_with_selected + 1) % state.openers.len(); }
                        }
                        KeyCode::Enter => {
                            let idx = state.open_with_selected;
                            state.show_open_with = false;
                            if let Some(msg) = state.open_with(idx) { state.status_msg = Some(msg); }
                        }
                        KeyCode::Char(c) if c.is_ascii_digit() && c != '0' => {
                            let idx = (c as usize) - ('1' as usize);
                            if idx < state.openers.len() {
                                state.show_open_with = false;
                                if let Some(msg) = state.open_with(idx) { state.status_msg = Some(msg); }
                            }
                        }
                        _ => {}
                    }
                    continue;
                }

                // ── Dialog input ──────────────────────────────────────────
                if state.dialog.is_some() {
                    match key.code {
                        KeyCode::Esc => { state.dialog = None; state.dialog_input.clear(); state.status_msg = None; }
                        KeyCode::Enter => {
                            let result = actions::execute_dialog(&mut state);
                            if let Some(msg) = result { state.status_msg = Some(msg); }
                            state.dialog = None; state.dialog_input.clear(); state.refresh();
                        }
                        KeyCode::Backspace => { state.dialog_input.pop(); }
                        KeyCode::Char(c) => { state.dialog_input.push(c); }
                        _ => {}
                    }
                    continue;
                }

                // ── Confirm dialog ────────────────────────────────────────
                if state.confirm.is_some() {
                    match key.code {
                        KeyCode::Char('y') | KeyCode::Char('Y') => {
                            let result = actions::execute_confirm(&mut state);
                            if let Some(msg) = result { state.status_msg = Some(msg); }
                            state.confirm = None; state.refresh();
                        }
                        _ => { state.confirm = None; state.status_msg = Some("cancelled".to_string()); }
                    }
                    continue;
                }

                // ── Normal mode ───────────────────────────────────────────
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break None,
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break None,

                    // Navigation
                    KeyCode::Up | KeyCode::Char('k') => state.move_up(),
                    KeyCode::Down | KeyCode::Char('j') => state.move_down(),
                    KeyCode::PageUp => state.page_up(15),
                    KeyCode::PageDown => state.page_down(15),
                    KeyCode::Home | KeyCode::Char('g') => { state.selected = 0; state.invalidate_preview(); }
                    KeyCode::End | KeyCode::Char('G') => {
                        let n = state.entries.len();
                        if n > 0 { state.selected = n - 1; state.invalidate_preview(); }
                    }

                    // Open
                    KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => {
                        open_entry(&mut state);
                    }

                    // Open with
                    KeyCode::Char('o') => state.show_open_with_menu(),

                    // Go up
                    KeyCode::Backspace | KeyCode::Left | KeyCode::Char('h') => state.go_up(),

                    // Home
                    KeyCode::Char('~') => { if let Some(home) = dirs::home_dir() { state.enter_dir(&home); } }

                    // Mounted drives menu
                    KeyCode::Char('m') => {
                        if !state.mounted_drives.is_empty() {
                            // Jump to first mounted drive
                            let path = state.mounted_drives[0].path.clone();
                            state.enter_dir(&path);
                            state.status_msg = Some(format!("📱 {}", state.mounted_drives[0].name));
                        } else {
                            state.status_msg = Some("no external drives found".to_string());
                        }
                    }

                    // Drives list
                    KeyCode::Char('M') => {
                        // Refresh and show drives
                        state.mounted_drives = state::detect_mounted_drives();
                        if state.mounted_drives.is_empty() {
                            state.status_msg = Some("no drives at /media, /mnt, or /run/media".to_string());
                        } else {
                            let names: Vec<String> = state.mounted_drives.iter()
                                .map(|d| format!("{} {} ({})", d.icon, d.name, crate::utils::format_size(d.size_total)))
                                .collect();
                            state.status_msg = Some(format!("drives: {}", names.join(" | ")));
                        }
                    }

                    // Hidden files
                    KeyCode::Char('.') => {
                        state.show_hidden = !state.show_hidden;
                        state.refresh();
                        state.status_msg = Some(if state.show_hidden { "showing hidden".to_string() } else { "hiding hidden".to_string() });
                    }

                    // Sort
                    KeyCode::Char('s') => { state.cycle_sort(); state.refresh(); }

                    // New file
                    KeyCode::Char('n') => { state.dialog = Some(state::Dialog::NewFile); state.dialog_input.clear(); }

                    // New dir
                    KeyCode::Char('N') => { state.dialog = Some(state::Dialog::NewDir); state.dialog_input.clear(); }

                    // Rename
                    KeyCode::Char('r') => {
                        if let Some(entry) = state.selected_entry() {
                            let name = entry.name.clone();
                            state.dialog = Some(state::Dialog::Rename);
                            state.dialog_input = name;
                        }
                    }

                    // Delete
                    KeyCode::Char('d') | KeyCode::Delete => {
                        if let Some(entry) = state.selected_entry() {
                            let name = entry.name.clone();
                            let is_dir = entry.is_dir;
                            state.confirm = Some(state::Confirm::Delete { name, is_dir });
                        }
                    }

                    // Yank
                    KeyCode::Char('y') => {
                        if let Some(entry) = state.selected_entry() {
                            let path = entry.path.clone();
                            let size = entry.size;
                            let name = entry.name.clone();
                            state.yanked = Some(path);
                            state.cut_mode = false;
                            state.status_msg = Some(format!("📋 yanked: {} ({})", name, crate::utils::format_size(size)));
                        }
                    }

                    // Paste
                    KeyCode::Char('p') => {
                        if let Some(src) = state.yanked.clone() {
                            let result = actions::paste(&state.cwd, &src);
                            state.status_msg = Some(result);
                            state.refresh();
                        } else { state.status_msg = Some("nothing yanked — press y first".to_string()); }
                    }

                    // Cut
                    KeyCode::Char('x') => {
                        if let Some(entry) = state.selected_entry() {
                            let path = entry.path.clone();
                            let name = entry.name.clone();
                            state.yanked = Some(path);
                            state.cut_mode = true;
                            state.status_msg = Some(format!("✂️ cut: {}", name));
                        }
                    }

                    // Paste-move
                    KeyCode::Char('P') => {
                        if state.cut_mode {
                            if let Some(src) = state.yanked.clone() {
                                let result = actions::move_entry(&state.cwd, &src);
                                state.status_msg = Some(result);
                                state.yanked = None; state.cut_mode = false; state.refresh();
                            }
                        } else { state.status_msg = Some("nothing to move — press x first".to_string()); }
                    }

                    // Import from external (copy selected to home Downloads)
                    KeyCode::Char('I') => {
                        if let Some(entry) = state.selected_entry() {
                            let src = entry.path.clone();
                            #[allow(unused_variables)]
                            let name = entry.name.clone();
                            let dl = dirs::home_dir().map(|h| h.join("Downloads")).unwrap_or_else(|| state.cwd.clone());
                            let _ = std::fs::create_dir_all(&dl);
                            let result = actions::copy_from_external(&src, &dl);
                            state.status_msg = Some(result);
                        }
                    }

                    // Cd here
                    KeyCode::Char('c') => {
                        let target = if let Some(entry) = state.selected_entry() {
                            if entry.is_dir { Some(entry.path.clone()) } else { Some(state.cwd.clone()) }
                        } else { Some(state.cwd.clone()) };
                        if let Some(t) = target { break Some(t); }
                    }

                    // Preview
                    KeyCode::Char('v') => { state.show_preview = !state.show_preview; }

                    // Filter (live search)
                    KeyCode::Char('/') => { state.start_filter(); }

                    // Clear filter
                    KeyCode::Char('\\') => { state.clear_filter(); state.status_msg = Some("filter cleared".to_string()); }

                    // Chmod
                    #[cfg(unix)]
                    KeyCode::Char('X') => {
                        if let Some(entry) = state.selected_entry() {
                            let path = entry.path.clone();
                            let result = actions::toggle_executable(&path);
                            state.status_msg = Some(result); state.refresh();
                        }
                    }

                    _ => {}
                }
                }
                _ => {}
            }
        }
    };

    tui_restore(&mut terminal);
    Ok(result)
}

fn handle_mouse(
    state: &mut ExplorerState,
    mouse: MouseEvent,
    last_click: &mut Option<(Instant, u16, u16, usize)>,
) {
    match mouse.kind {
        MouseEventKind::ScrollUp => state.move_up(),
        MouseEventKind::ScrollDown => state.move_down(),
        MouseEventKind::Down(MouseButton::Left) => {
            if let Some(idx) = state
                .layout
                .row_at(mouse.column, mouse.row, state.scroll_offset)
            {
                if idx < state.entries.len() {
                    let now = Instant::now();
                    let double = last_click
                        .map(|(t, c, r, i)| {
                            t.elapsed() < Duration::from_millis(400)
                                && c == mouse.column
                                && r == mouse.row
                                && i == idx
                        })
                        .unwrap_or(false);
                    state.selected = idx;
                    state.invalidate_preview();
                    if double {
                        open_entry(state);
                        *last_click = None;
                    } else {
                        *last_click = Some((now, mouse.column, mouse.row, idx));
                    }
                }
            }
        }
        _ => {}
    }
}

fn open_entry(state: &mut ExplorerState) {
    if let Some(entry) = state.selected_entry() {
        if ExplorerState::is_parent_entry(entry) {
            state.go_up();
            return;
        }
        let path = entry.path.clone();
        let is_dir = entry.is_dir;
        if is_dir {
            state.enter_dir(&path);
        } else {
            let _ = open::that(&path);
            state.status_msg = Some(format!("opened {}", path.display()));
        }
    }
}