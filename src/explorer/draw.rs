// src/explorer/draw.rs

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::config::*;
use crate::utils::{file_icon, file_icon_color, format_size, format_time, shorten_path};
use super::state::{Confirm, Dialog, ExplorerState, PreviewKind};

pub fn draw_explorer(f: &mut Frame, state: &mut ExplorerState) {
    let area = f.area();
    f.render_widget(Block::default().style(Style::default().bg(BG)), area);

    let path_parts = build_breadcrumb(&state.cwd);
    let outer = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(BORDER))
        .title(Line::from({
            let mut spans = vec![
                Span::styled(" 📂 ", Style::default().fg(CYAN).add_modifier(Modifier::BOLD)),
            ];
            spans.extend(path_parts);
            spans.push(Span::raw(" "));
            spans
        }))
        .title_alignment(Alignment::Left)
        .style(Style::default().bg(BG_SURFACE));
    f.render_widget(outer, area);

    let inner = area.inner(Margin { horizontal: 1, vertical: 1 });

    // Add filter bar when active
    let has_filter = state.filter_active || !state.filter.is_empty();
    let chunks = if has_filter {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),  // info bar
                Constraint::Length(3),  // filter bar
                Constraint::Min(5),     // main content
                Constraint::Length(1),  // status
            ])
            .split(inner)
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(0),  // no filter bar
                Constraint::Min(5),
                Constraint::Length(1),
            ])
            .split(inner)
    };

    draw_info_bar(f, state, chunks[0]);

    if has_filter {
        draw_filter_bar(f, state, chunks[1]);
    }

    if state.show_preview {
        let split = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(chunks[2]);
        draw_file_list(f, state, split[0]);
        draw_preview_panel(f, state, split[1]);
    } else {
        draw_file_list(f, state, chunks[2]);
    }

    draw_status_bar(f, state, chunks[3]);

    if let Some(ref dialog) = state.dialog {
        draw_dialog(f, dialog, &state.dialog_input, area);
    }
    if let Some(ref confirm) = state.confirm {
        draw_confirm(f, confirm, area);
    }
    if state.show_open_with {
        draw_open_with(f, state, area);
    }
}

fn build_breadcrumb(path: &std::path::PathBuf) -> Vec<Span<'static>> {
    let display = shorten_path(path);
    let parts: Vec<&str> = display.split('/').collect();
    let mut spans = Vec::new();
    for (i, part) in parts.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled(" › ", Style::default().fg(MUTED)));
        }
        if i == parts.len() - 1 {
            spans.push(Span::styled(
                part.to_string(),
                Style::default().fg(CYAN_BRIGHT).add_modifier(Modifier::BOLD),
            ));
        } else {
            spans.push(Span::styled(part.to_string(), Style::default().fg(TEXT_DIM)));
        }
    }
    spans
}

fn draw_filter_bar(f: &mut Frame, state: &ExplorerState, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(if state.filter_active { CYAN } else { BORDER }))
        .style(Style::default().bg(BG_PANEL));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let blink = (SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().subsec_millis() / 530) % 2 == 0;
    let cursor = if state.filter_active && blink { "▌" } else if state.filter_active { " " } else { "" };

    let match_count = state.entries.len();
    let total = state.all_entries.len();

    let line = Line::from(vec![
        Span::styled(" 🔍 ", Style::default().fg(CYAN)),
        Span::styled(&state.filter, Style::default().fg(CYAN_BRIGHT).add_modifier(Modifier::BOLD)),
        Span::styled(cursor, Style::default().fg(CYAN)),
        Span::styled(
            if state.filter.is_empty() && state.filter_active {
                " type to filter...".to_string()
            } else {
                String::new()
            },
            Style::default().fg(MUTED),
        ),
        Span::styled(
            format!("  {}/{} ", match_count, total),
            Style::default().fg(if match_count == 0 { RED } else { GREEN }),
        ),
    ]);
    f.render_widget(Paragraph::new(line), inner);
}

fn draw_info_bar(f: &mut Frame, state: &ExplorerState, area: Rect) {
    let dir_count = state.entries.iter().filter(|e| e.is_dir).count();
    let file_count = state.entries.iter().filter(|e| !e.is_dir).count();
    let total_size: u64 = state.entries.iter().map(|e| e.size).sum();

    let mut spans = vec![
        Span::styled(" ", Style::default()),
        Span::styled(format!("📁 {} ", dir_count), Style::default().fg(BLUE)),
        Span::styled(format!("📄 {} ", file_count), Style::default().fg(TEXT_DIM)),
        Span::styled(format!("💾 {} ", format_size(total_size)), Style::default().fg(YELLOW)),
        Span::styled(" │ ", Style::default().fg(BORDER)),
    ];

    let sort_icon = match state.sort_mode {
        super::state::SortMode::Name => "🔤",
        super::state::SortMode::Size => "📏",
        super::state::SortMode::Modified => "🕐",
        super::state::SortMode::Type => "📎",
    };
    spans.push(Span::styled(format!("{} {} ", sort_icon, state.sort_mode.label()), Style::default().fg(MUTED)));

    if state.show_hidden {
        spans.push(Span::styled("│ ", Style::default().fg(BORDER)));
        spans.push(Span::styled("👁️ hidden ", Style::default().fg(YELLOW)));
    }
    if let Some(ref yanked) = state.yanked {
        let icon = if state.cut_mode { "✂️" } else { "📋" };
        let color = if state.cut_mode { ORANGE } else { TEAL };
        let name = yanked.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
        spans.push(Span::styled("│ ", Style::default().fg(BORDER)));
        spans.push(Span::styled(format!("{} {} ", icon, name), Style::default().fg(color)));
    }
    if !state.mounted_drives.is_empty() {
        spans.push(Span::styled("│ ", Style::default().fg(BORDER)));
        spans.push(Span::styled(format!("🔌 {} drives ", state.mounted_drives.len()), Style::default().fg(MAGENTA)));
    }

    f.render_widget(Paragraph::new(Line::from(spans)), area);
}

fn draw_file_list(f: &mut Frame, state: &mut ExplorerState, area: Rect) {
    let total = state.entries.len();
    let position = if total > 0 { format!(" {}/{} ", state.selected + 1, total) } else { String::new() };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(BORDER))
        .title(Span::styled(position, Style::default().fg(MUTED)))
        .title_alignment(Alignment::Right)
        .style(Style::default().bg(BG_SURFACE));
    let block_inner = block.inner(area);
    f.render_widget(block, area);

    if state.entries.is_empty() {
        let msg = if !state.filter.is_empty() { "  🔍 no matches" } else { "  📂 empty directory" };
        f.render_widget(Paragraph::new(vec![Line::from(""), Line::from(vec![
            Span::styled(msg, Style::default().fg(MUTED))
        ])]), block_inner);
        return;
    }

    let list_height = block_inner.height as usize;
    state.adjust_scroll(list_height);

    let filter_lower = state.filter.to_lowercase();

    let items: Vec<ListItem> = state.entries.iter().enumerate()
        .skip(state.scroll_offset).take(list_height)
        .map(|(idx, entry)| {
            let is_sel = idx == state.selected;
            let bg = if is_sel { BG_SELECTED } else { BG_SURFACE };
            let icon = file_icon(&entry.name, entry.is_dir);
            let icon_color = file_icon_color(&entry.name, entry.is_dir);

            let name_color = if entry.is_dir { BLUE }
                else if entry.is_executable { GREEN }
                else if entry.is_symlink { MAGENTA }
                else if entry.is_hidden { MUTED }
                else { TEXT };

            let name_style = if is_sel {
                Style::default().fg(CYAN_BRIGHT).bg(bg).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(name_color).bg(bg)
            };

            let dn = if entry.is_dir { format!("{}/", entry.name) } else { entry.name.clone() };
            let ss = if entry.is_dir { "      ".to_string() } else { format!("{:>6}", format_size(entry.size)) };
            let ts = format_time(entry.modified);

            let badge = if entry.is_symlink { Span::styled("🔗 ", Style::default().bg(bg)) }
                else if entry.is_readonly { Span::styled("🔒 ", Style::default().bg(bg)) }
                else if entry.is_executable && !entry.is_dir { Span::styled("⚡ ", Style::default().bg(bg)) }
                else { Span::styled("   ", Style::default().bg(bg)) };

            let gutter = if is_sel {
                Span::styled(" ▸ ", Style::default().fg(CYAN).bg(bg).add_modifier(Modifier::BOLD))
            } else {
                Span::styled("   ", Style::default().bg(bg))
            };

            // Highlight matching characters in name
            let name_spans = if !filter_lower.is_empty() {
                highlight_match(&dn, &filter_lower, name_style, Style::default().fg(YELLOW).bg(bg).add_modifier(Modifier::BOLD))
            } else {
                let nw = 30usize;
                let pn = if dn.len() >= nw { dn[..nw].to_string() } else { format!("{:<w$}", dn, w = nw) };
                vec![Span::styled(pn, name_style)]
            };

            let mut line_spans = vec![
                gutter,
                Span::styled(format!("{} ", icon), Style::default().fg(icon_color).bg(bg)),
            ];
            line_spans.extend(name_spans);
            line_spans.extend(vec![
                badge,
                Span::styled(ss, Style::default().fg(TEXT_DIM).bg(bg)),
                Span::styled(" ", Style::default().bg(bg)),
                Span::styled(format!("{:>5}", ts), Style::default().fg(MUTED).bg(bg)),
                Span::styled(" ", Style::default().bg(bg)),
            ]);

            ListItem::new(Line::from(line_spans)).style(Style::default().bg(bg))
        }).collect();

    if total > list_height {
        let th = block_inner.height as f64;
        let tsz = ((list_height as f64 / total as f64) * th).max(1.0) as u16;
        let tp = ((state.scroll_offset as f64 / total as f64) * th) as u16;
        for y in 0..block_inner.height {
            let (ch, c) = if y >= tp && y < tp + tsz { ("┃", CYAN_DIM) } else { ("│", BORDER_DIM) };
            f.render_widget(
                Paragraph::new(Span::styled(ch, Style::default().fg(c))),
                Rect { x: block_inner.x + block_inner.width - 1, y: block_inner.y + y, width: 1, height: 1 },
            );
        }
    }

    let mut ls = ListState::default();
    ls.select(Some(state.selected.saturating_sub(state.scroll_offset).min(items.len().saturating_sub(1))));
    f.render_stateful_widget(
        List::new(items).highlight_style(Style::default().bg(BG_SELECTED).add_modifier(Modifier::BOLD)),
        Rect { width: block_inner.width.saturating_sub(1), ..block_inner },
        &mut ls,
    );
}

/// Highlight matching characters in a filename
fn highlight_match(text: &str, query: &str, normal: Style, highlight: Style) -> Vec<Span<'static>> {
    let nw = 30usize;
    let display = if text.len() >= nw { text[..nw].to_string() } else { format!("{:<w$}", text, w = nw) };
    let text_lower = display.to_lowercase();

    // Try substring match first
    if let Some(pos) = text_lower.find(query) {
        let before = &display[..pos];
        let matched = &display[pos..pos + query.len()];
        let after = &display[pos + query.len()..];
        return vec![
            Span::styled(before.to_string(), normal),
            Span::styled(matched.to_string(), highlight),
            Span::styled(after.to_string(), normal),
        ];
    }

    // Fuzzy highlight
    let mut spans = Vec::new();
    let mut qi = 0;
    let query_chars: Vec<char> = query.chars().collect();
    let mut current = String::new();
    let mut current_is_match = false;

    for ch in display.chars() {
        let is_match = qi < query_chars.len() && ch.to_lowercase().next() == query_chars[qi].to_lowercase().next();

        if is_match != current_is_match && !current.is_empty() {
            spans.push(Span::styled(current.clone(), if current_is_match { highlight } else { normal }));
            current.clear();
        }

        current.push(ch);
        current_is_match = is_match;
        if is_match { qi += 1; }
    }

    if !current.is_empty() {
        spans.push(Span::styled(current, if current_is_match { highlight } else { normal }));
    }

    spans
}

fn draw_preview_panel(f: &mut Frame, state: &mut ExplorerState, area: Rect) {
    let entry = state.selected_entry().cloned();
    let title_name = entry.as_ref().map(|e| e.name.clone()).unwrap_or_default();
    let is_dir = entry.as_ref().map(|e| e.is_dir).unwrap_or(false);
    let title_icon = file_icon(&title_name, is_dir);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(BORDER))
        .title(Line::from(vec![
            Span::styled(format!(" {} ", title_icon), Style::default().fg(CYAN)),
            Span::styled(format!("{} ", title_name), Style::default().fg(CYAN)),
        ]))
        .style(Style::default().bg(BG_SURFACE));
    let block_inner = block.inner(area);
    f.render_widget(block, area);

    let pw = block_inner.width as usize;
    let ph = block_inner.height as usize;
    let (lines, kind, info) = state.get_preview(pw, ph);
    let lines = lines.to_vec();
    let kind = kind.clone();
    let info = info.to_vec();

    match kind {
        PreviewKind::Image => {
            let info_height = (info.len() as u16 + 1).min(4);
            let split = Layout::default().direction(Direction::Vertical)
                .constraints([Constraint::Length(info_height), Constraint::Min(2)]).split(block_inner);
            let info_lines: Vec<Line> = info.iter()
                .map(|l| Line::from(vec![Span::styled(format!("  {}", l), Style::default().fg(TEXT_DIM))])).collect();
            f.render_widget(Paragraph::new(info_lines), split[0]);
            render_image_preview(f, &lines, split[1]);
        }
        PreviewKind::Video => {
            let info_height = (info.len() as u16 + 1).min(6);
            let split = Layout::default().direction(Direction::Vertical)
                .constraints([Constraint::Length(info_height), Constraint::Min(2)]).split(block_inner);
            let info_lines: Vec<Line> = info.iter()
                .map(|l| Line::from(vec![Span::styled(format!("  {}", l), Style::default().fg(TEXT_DIM))])).collect();
            f.render_widget(Paragraph::new(info_lines), split[0]);
            let has_image = lines.iter().any(|l| l.contains("\x1b["));
            if has_image { render_image_preview(f, &lines, split[1]); }
            else {
                let tl: Vec<Line> = lines.iter().take(split[1].height as usize)
                    .map(|l| Line::from(vec![Span::styled(format!("  {}", l), Style::default().fg(TEXT_DIM))])).collect();
                f.render_widget(Paragraph::new(tl), split[1]);
            }
        }
        PreviewKind::Audio => {
            let tl: Vec<Line> = lines.iter().take(ph).map(|l| {
                let color = if l.starts_with("🎵") || l.starts_with("🎤") || l.starts_with("💿")
                    || l.starts_with("🏷️") || l.starts_with("📅") || l.starts_with("🔢") { CYAN }
                    else if l.contains("╭") || l.contains("╰") || l.contains("│") || l.contains("▁") { MAGENTA }
                    else { TEXT_DIM };
                Line::from(vec![Span::styled(format!("  {}", l), Style::default().fg(color))])
            }).collect();
            f.render_widget(Paragraph::new(tl), block_inner);
        }
        PreviewKind::Binary => {
            let tl: Vec<Line> = lines.iter().take(ph).enumerate().map(|(i, l)| {
                Line::from(vec![Span::styled(l.to_string(), Style::default().fg(if i == 0 { CYAN } else { MUTED }))])
            }).collect();
            f.render_widget(Paragraph::new(tl), block_inner);
        }
        PreviewKind::Text => {
            let text: Vec<Line> = lines.iter().take(ph).enumerate().map(|(i, line)| {
                Line::from(vec![
                    Span::styled(format!(" {:>3} ", i + 1), Style::default().fg(BORDER)),
                    Span::styled("│ ", Style::default().fg(BORDER_DIM)),
                    Span::styled(line.to_string(), Style::default().fg(syntax_color(line))),
                ])
            }).collect();
            if text.is_empty() {
                f.render_widget(Paragraph::new(Line::from(vec![Span::styled("  📄 empty file", Style::default().fg(MUTED))])), block_inner);
            } else {
                f.render_widget(Paragraph::new(text).wrap(Wrap { trim: false }), block_inner);
            }
        }
        PreviewKind::Directory => {
            let tl: Vec<Line> = lines.iter().take(ph).map(|l| {
                let color = if l.starts_with("📁") && !l.starts_with("  📁") { CYAN }
                    else if l.starts_with("  📁") { BLUE }
                    else if l.starts_with("  📄") { TEXT_DIM }
                    else if l.contains("dirs") && l.contains("files") { YELLOW }
                    else { TEXT_DIM };
                Line::from(vec![Span::styled(l.to_string(), Style::default().fg(color))])
            }).collect();
            f.render_widget(Paragraph::new(tl), block_inner);
        }
        _ => {
            f.render_widget(Paragraph::new(Line::from(vec![
                Span::styled("  📄 no preview", Style::default().fg(MUTED)),
            ])), block_inner);
        }
    }
}

fn render_image_preview(f: &mut Frame, lines: &[String], area: Rect) {
    let rendered: Vec<Line> = lines.iter()
        .take(area.height as usize)
        .map(|line| Line::from(parse_ansi_line(line)))
        .collect();
    f.render_widget(Paragraph::new(rendered), area);
}

fn parse_ansi_line(input: &str) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    let mut remaining = input;
    let mut current_fg = TEXT_DIM;
    let mut current_bg = BG_SURFACE;

    while !remaining.is_empty() {
        if let Some(esc_pos) = remaining.find("\x1b[") {
            if esc_pos > 0 {
                spans.push(Span::styled(remaining[..esc_pos].to_string(), Style::default().fg(current_fg).bg(current_bg)));
            }
            let after_esc = &remaining[esc_pos + 2..];
            if let Some(m_pos) = after_esc.find('m') {
                let codes = &after_esc[..m_pos];
                let parts: Vec<&str> = codes.split(';').collect();
                let mut i = 0;
                while i < parts.len() {
                    match parts[i] {
                        "0" => { current_fg = TEXT_DIM; current_bg = BG_SURFACE; }
                        "38" if i + 4 < parts.len() && parts[i + 1] == "2" => {
                            if let (Ok(r), Ok(g), Ok(b)) = (parts[i+2].parse::<u8>(), parts[i+3].parse::<u8>(), parts[i+4].parse::<u8>()) {
                                current_fg = Color::Rgb(r, g, b);
                            }
                            i += 4;
                        }
                        "48" if i + 4 < parts.len() && parts[i + 1] == "2" => {
                            if let (Ok(r), Ok(g), Ok(b)) = (parts[i+2].parse::<u8>(), parts[i+3].parse::<u8>(), parts[i+4].parse::<u8>()) {
                                current_bg = Color::Rgb(r, g, b);
                            }
                            i += 4;
                        }
                        _ => {}
                    }
                    i += 1;
                }
                remaining = &after_esc[m_pos + 1..];
            } else {
                spans.push(Span::styled(remaining.to_string(), Style::default().fg(current_fg).bg(current_bg)));
                break;
            }
        } else {
            if !remaining.is_empty() {
                spans.push(Span::styled(remaining.to_string(), Style::default().fg(current_fg).bg(current_bg)));
            }
            break;
        }
    }
    if spans.is_empty() { spans.push(Span::raw("")); }
    spans
}

fn syntax_color(line: &str) -> Color {
    let t = line.trim();
    if t.starts_with("//") || t.starts_with("#") || t.starts_with("--") { MUTED }
    else if t.starts_with("fn ") || t.starts_with("def ") || t.starts_with("func ")
        || t.starts_with("function ") || t.starts_with("class ") || t.starts_with("struct ")
        || t.starts_with("pub ") || t.starts_with("impl ") || t.starts_with("trait ")
        || t.starts_with("enum ") || t.starts_with("type ") || t.starts_with("interface ") { CYAN }
    else if t.starts_with("import ") || t.starts_with("use ") || t.starts_with("from ")
        || t.starts_with("require") || t.starts_with("include") { MAGENTA }
    else if t.starts_with("return ") || t.starts_with("if ") || t.starts_with("else")
        || t.starts_with("for ") || t.starts_with("while ") || t.starts_with("match ")
        || t.starts_with("switch ") || t.starts_with("case ") { ORANGE }
    else if line.contains("TODO") || line.contains("FIXME") || line.contains("HACK") { YELLOW }
    else { TEXT_DIM }
}

fn draw_status_bar(f: &mut Frame, state: &ExplorerState, area: Rect) {
    if let Some(ref msg) = state.status_msg {
        let (icon, color) = if msg.starts_with('✓') { ("✅", GREEN) }
            else if msg.starts_with('✗') { ("❌", RED) }
            else { ("ℹ️", TEXT_DIM) };
        f.render_widget(Paragraph::new(Line::from(vec![
            Span::styled(format!(" {} ", icon), Style::default().fg(color)),
            Span::styled(msg.trim_start_matches('✓').trim_start_matches('✗').trim(), Style::default().fg(color)),
        ])), area);
        return;
    }

    let kb: Vec<(&str, &str, Color)> = vec![
        ("↑↓", "nav", MUTED), ("⏎", "open", BLUE), ("o", "with", CYAN),
        ("⌫", "back", MUTED), ("n", "file", GREEN), ("N", "dir", GREEN),
        ("r", "ren", YELLOW), ("d", "del", RED), ("y", "yank", TEAL),
        ("p", "paste", TEAL), ("x", "cut", ORANGE), ("P", "move", ORANGE),
        ("v", "view", MAGENTA), ("/", "find", BLUE), ("m", "drives", MAGENTA),
        ("I", "import", TEAL), ("c", "cd", CYAN),
    ];
    let spans: Vec<Span> = kb.iter().flat_map(|(k, d, c)| vec![
        Span::styled(format!(" {}", k), Style::default().fg(*c).add_modifier(Modifier::BOLD)),
        Span::styled(format!("·{} ", d), Style::default().fg(MUTED)),
    ]).collect();
    f.render_widget(Paragraph::new(Line::from(spans)), area);
}

fn draw_open_with(f: &mut Frame, state: &ExplorerState, area: Rect) {
    let entry_name = state.selected_entry().map(|e| e.name.clone()).unwrap_or_default();
    let oc = state.openers.len();
    let ph = (oc as u16 + 6).min(area.height - 4);
    let popup = centered_popup(55, ph, area);
    f.render_widget(Clear, popup);

    let block = Block::default().borders(Borders::ALL)
        .border_style(Style::default().fg(BORDER_ACTIVE))
        .title(Line::from(vec![
            Span::styled(" 📂 ", Style::default().fg(CYAN)),
            Span::styled("Open With ", Style::default().fg(CYAN_BRIGHT).add_modifier(Modifier::BOLD)),
        ])).style(Style::default().bg(BG_PANEL));
    let inner = block.inner(popup);
    f.render_widget(block, popup);

    let fi = file_icon(&entry_name, false);
    let fc = file_icon_color(&entry_name, false);
    let mut lines: Vec<Line> = vec![
        Line::from(vec![
            Span::styled(format!("  {} ", fi), Style::default().fg(fc)),
            Span::styled(&entry_name, Style::default().fg(TEXT).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
    ];
    for (i, opener) in state.openers.iter().enumerate() {
        let is_sel = i == state.open_with_selected;
        let bg = if is_sel { BG_SELECTED } else { BG_PANEL };
        let num = if i < 9 { format!(" {} ", i + 1) } else { "   ".to_string() };
        let gutter = if is_sel {
            Span::styled(" ▸ ", Style::default().fg(CYAN).bg(bg).add_modifier(Modifier::BOLD))
        } else { Span::styled("   ", Style::default().bg(bg)) };
        let ns = if is_sel { Style::default().fg(CYAN_BRIGHT).bg(bg).add_modifier(Modifier::BOLD) }
            else { Style::default().fg(TEXT).bg(bg) };
        lines.push(Line::from(vec![
            gutter,
            Span::styled(&opener.icon, Style::default().fg(if is_sel { CYAN } else { TEXT_DIM }).bg(bg)),
            Span::styled(" ", Style::default().bg(bg)),
            Span::styled(&opener.name, ns),
            Span::styled(format!("  ({})", opener.command), Style::default().fg(MUTED).bg(bg)),
            Span::styled(num, Style::default().fg(MUTED).bg(bg)),
        ]));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("  ⏎ ", Style::default().fg(GREEN)),
        Span::styled("select   ", Style::default().fg(MUTED)),
        Span::styled("1-9 ", Style::default().fg(CYAN)),
        Span::styled("quick pick   ", Style::default().fg(MUTED)),
        Span::styled("esc ", Style::default().fg(RED)),
        Span::styled("cancel", Style::default().fg(MUTED)),
    ]));
    f.render_widget(Paragraph::new(lines), inner);
}

fn draw_dialog(f: &mut Frame, dialog: &Dialog, input: &str, area: Rect) {
    let (title, icon) = match dialog {
        Dialog::NewFile => ("new file", "📄"),
        Dialog::NewDir => ("new folder", "📁"),
        Dialog::Rename => ("rename", "✏️"),
    };
    let popup = centered_popup(50, 7, area);
    f.render_widget(Clear, popup);
    let block = Block::default().borders(Borders::ALL)
        .border_style(Style::default().fg(BORDER_ACTIVE))
        .title(Line::from(vec![
            Span::styled(format!(" {} ", icon), Style::default().fg(CYAN)),
            Span::styled(format!("{} ", title), Style::default().fg(CYAN_BRIGHT).add_modifier(Modifier::BOLD)),
        ])).style(Style::default().bg(BG_PANEL));
    let inner = block.inner(popup);
    f.render_widget(block, popup);

    let blink = (SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().subsec_millis() / 530) % 2 == 0;
    let cursor = if blink { "▌" } else { " " };
    f.render_widget(Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  name: ", Style::default().fg(TEXT_DIM)),
            Span::styled(input, Style::default().fg(TEXT_BRIGHT)),
            Span::styled(cursor, Style::default().fg(CYAN)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ⏎ ", Style::default().fg(GREEN)),
            Span::styled("confirm   ", Style::default().fg(MUTED)),
            Span::styled("esc ", Style::default().fg(RED)),
            Span::styled("cancel", Style::default().fg(MUTED)),
        ]),
    ]), inner);
}

fn draw_confirm(f: &mut Frame, confirm: &Confirm, area: Rect) {
    let popup = centered_popup(50, 9, area);
    f.render_widget(Clear, popup);

    match confirm {
        Confirm::Delete { name, is_dir } => {
            let block = Block::default().borders(Borders::ALL)
                .border_style(Style::default().fg(RED))
                .title(Line::from(vec![
                    Span::styled(" ⚠️ ", Style::default().fg(RED)),
                    Span::styled("confirm delete ", Style::default().fg(RED).add_modifier(Modifier::BOLD)),
                ])).style(Style::default().bg(BG_PANEL));
            let inner = block.inner(popup);
            f.render_widget(block, popup);

            let kind = if *is_dir { "folder" } else { "file" };
            let ki = if *is_dir { "📁" } else { "📄" };
            f.render_widget(Paragraph::new(vec![
                Line::from(""),
                Line::from(vec![
                    Span::styled("  permanently delete ", Style::default().fg(TEXT)),
                    Span::styled(kind, Style::default().fg(RED)),
                    Span::styled(":", Style::default().fg(TEXT)),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled(format!("  {} ", ki), Style::default().fg(TEXT_DIM)),
                    Span::styled(name, Style::default().fg(CYAN_BRIGHT).add_modifier(Modifier::BOLD)),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("  y ", Style::default().fg(RED).add_modifier(Modifier::BOLD)),
                    Span::styled("delete     ", Style::default().fg(MUTED)),
                    Span::styled("any ", Style::default().fg(GREEN).add_modifier(Modifier::BOLD)),
                    Span::styled("cancel", Style::default().fg(MUTED)),
                ]),
            ]), inner);
        }
        Confirm::Overwrite { src, dest: _ } => {
            let block = Block::default().borders(Borders::ALL)
                .border_style(Style::default().fg(YELLOW))
                .title(Line::from(vec![
                    Span::styled(" ⚠️ ", Style::default().fg(YELLOW)),
                    Span::styled("overwrite? ", Style::default().fg(YELLOW).add_modifier(Modifier::BOLD)),
                ])).style(Style::default().bg(BG_PANEL));
            let inner = block.inner(popup);
            f.render_widget(block, popup);

            let name = src.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
            f.render_widget(Paragraph::new(vec![
                Line::from(""),
                Line::from(vec![Span::styled("  file already exists:", Style::default().fg(TEXT))]),
                Line::from(""),
                Line::from(vec![
                    Span::styled(format!("  📄 {}", name), Style::default().fg(CYAN_BRIGHT).add_modifier(Modifier::BOLD)),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("  y ", Style::default().fg(YELLOW).add_modifier(Modifier::BOLD)),
                    Span::styled("overwrite  ", Style::default().fg(MUTED)),
                    Span::styled("any ", Style::default().fg(GREEN).add_modifier(Modifier::BOLD)),
                    Span::styled("cancel", Style::default().fg(MUTED)),
                ]),
            ]), inner);
        }
    }
}

fn centered_popup(percent_x: u16, height: u16, r: Rect) -> Rect {
    let v = Layout::default().direction(Direction::Vertical).constraints([
        Constraint::Percentage((100u16.saturating_sub(height.min(r.height) * 100 / r.height.max(1))) / 2),
        Constraint::Length(height), Constraint::Min(0),
    ]).split(r);
    Layout::default().direction(Direction::Horizontal).constraints([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ]).split(v[1])[1]
}