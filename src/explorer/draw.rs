// src/explorer/draw.rs

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

use crate::config::*;
use crate::ui::chrome::{
    clear, draw_brand_bar, draw_hint_bar, draw_modal_shell, draw_rule, draw_scroll_track,
    draw_section_head, inset, rail_span, Hint,
};
use crate::ui::icons::{entry_icon, parent as parent_icon};
use crate::utils::{format_size, format_time, shorten_path};
use super::state::{Confirm, Dialog, ExplorerState, PreviewKind};

struct ListCols {
    name_w: usize,
}

impl ListCols {
    fn from_width(w: u16) -> Self {
        // rail + gap + icon + gap + badge + size + time
        let fixed = 1 + 1 + 2 + 1 + 3 + 9 + 6;
        let name_w = (w as usize).saturating_sub(fixed).max(14);
        Self { name_w }
    }
}

pub fn draw_explorer(f: &mut Frame, state: &mut ExplorerState) {
    let area = f.area();
    clear(f, area);
    state.layout = super::layout::ExplorerLayout::default();
    state.layout.row_height = 1;

    let shell = inset(area, 1, 0);
    let filtering = state.filter_active || !state.filter.is_empty();

    let mut constraints = vec![
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ];
    if filtering {
        constraints.push(Constraint::Length(1));
    }
    constraints.extend([
        Constraint::Length(1),
        Constraint::Min(4),
        Constraint::Length(1),
        Constraint::Length(1),
    ]);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(shell);

    let mut i = 0;
    let path = shorten_path(&state.cwd);
    draw_brand_bar(f, chunks[i], "EXPLORE", &path, None);
    i += 1;
    draw_rule(f, chunks[i]);
    i += 1;
    draw_info_bar(f, state, chunks[i]);
    i += 1;

    let filter_chunk = if filtering {
        let chunk = chunks[i];
        draw_filter_line(f, state, chunk);
        i += 1;
        Some(chunk)
    } else {
        None
    };

    draw_rule(f, chunks[i]);
    i += 1;
    let body = chunks[i];
    i += 1;

    if state.show_preview {
        let split = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(54), Constraint::Percentage(46)])
            .split(body);
        draw_files_panel(f, state, split[0]);
        draw_preview_panel(f, state, split[1]);
    } else {
        draw_files_panel(f, state, body);
    }

    draw_rule(f, chunks[i]);
    i += 1;
    draw_footer(f, state, chunks[i]);

    if let Some(ref dialog) = state.dialog {
        draw_dialog(f, dialog, &state.dialog_input, area);
    }
    if let Some(ref confirm) = state.confirm {
        draw_confirm(f, confirm, area);
    }
    if state.show_open_with {
        draw_open_with(f, state, area);
    }

    if state.filter_active {
        if let Some(fa) = filter_chunk {
            f.set_cursor_position((fa.x + 4 + state.filter.len() as u16, fa.y));
        }
    }
}

fn real_entries(state: &ExplorerState) -> impl Iterator<Item = &super::state::FileEntry> {
    state
        .entries
        .iter()
        .filter(|e| !ExplorerState::is_parent_entry(e))
}

fn draw_info_bar(f: &mut Frame, state: &ExplorerState, area: Rect) {
    let dirs = real_entries(state).filter(|e| e.is_dir).count();
    let files = real_entries(state).filter(|e| !e.is_dir).count();
    let total_size: u64 = real_entries(state).map(|e| e.size).sum();

    let mut spans = vec![
        Span::styled(format!("  {dirs} dirs"), Style::default().fg(TEXT_DIM)),
        Span::styled("  ·  ", Style::default().fg(GHOST)),
        Span::styled(format!("{files} files"), Style::default().fg(TEXT_DIM)),
        Span::styled("  ·  ", Style::default().fg(GHOST)),
        Span::styled(format_size(total_size), Style::default().fg(MUTED)),
        Span::styled("  ·  ", Style::default().fg(GHOST)),
        Span::styled(
            format!("sort {}", state.sort_mode.label()),
            Style::default().fg(MUTED),
        ),
    ];

    if state.show_hidden {
        spans.push(Span::styled("  ·  ", Style::default().fg(GHOST)));
        spans.push(Span::styled("hidden", Style::default().fg(TEXT_DIM)));
    }
    if let Some(ref yanked) = state.yanked {
        let label = if state.cut_mode { "cut" } else { "yank" };
        let name = yanked
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        spans.push(Span::styled("  ·  ", Style::default().fg(GHOST)));
        spans.push(Span::styled(
            format!("{label} {name}"),
            Style::default().fg(TEXT_DIM),
        ));
    }
    if !state.mounted_drives.is_empty() {
        spans.push(Span::styled("  ·  ", Style::default().fg(GHOST)));
        spans.push(Span::styled(
            format!("{} drives", state.mounted_drives.len()),
            Style::default().fg(MUTED),
        ));
    }

    f.render_widget(Paragraph::new(Line::from(spans)), area);
}

fn draw_filter_line(f: &mut Frame, state: &ExplorerState, area: Rect) {
    let blink = crate::ui::chrome::cursor_blink();
    let cur = if state.filter_active && blink { "▍" } else { "" };
    let shown = real_entries(state).count();
    let total = state.all_entries.len();
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("  / ", Style::default().fg(MUTED)),
            Span::styled(&state.filter, Style::default().fg(TEXT_BRIGHT)),
            Span::styled(cur, Style::default().fg(TEXT)),
            Span::styled(
                format!("  {shown}/{total}"),
                Style::default().fg(if shown == 0 { MUTED } else { TEXT_DIM }),
            ),
        ])),
        area,
    );
}

fn draw_files_panel(f: &mut Frame, state: &mut ExplorerState, area: Rect) {
    if area.height < 2 {
        return;
    }
    let head = Rect::new(area.x, area.y, area.width, 1);
    let list_area = Rect::new(area.x, area.y + 1, area.width, area.height.saturating_sub(1));

    let total = state.entries.len();
    let pos = if total > 0 {
        format!("{}/{}", state.selected + 1, total)
    } else {
        String::new()
    };
    draw_section_head(f, head, "FILES", &pos);
    state.layout.file_list = draw_list(f, state, list_area);
}

fn draw_list(f: &mut Frame, state: &mut ExplorerState, area: Rect) -> Rect {
    if state.entries.is_empty() {
        let msg = if state.filter.is_empty() {
            "  empty"
        } else {
            "  no match"
        };
        f.render_widget(
            Paragraph::new(Span::styled(msg, Style::default().fg(MUTED))),
            area,
        );
        return area;
    }

    let cols = ListCols::from_width(area.width);
    let visible = area.height as usize;
    state.adjust_scroll(visible.max(1));

    let filter_lower = state.filter.to_lowercase();
    let scroll_offset = state.scroll_offset;
    let selected = state.selected;

    let items: Vec<ListItem> = state
        .entries
        .iter()
        .enumerate()
        .skip(scroll_offset)
        .take(visible)
        .map(|(idx, e)| {
            let sel = idx == selected;
            let bg = if sel { BG_SELECTED } else { BG };
            let is_parent = ExplorerState::is_parent_entry(e);

            let icon = if is_parent {
                parent_icon()
            } else {
                entry_icon(&e.name, e.is_dir)
            };

            let label = if e.is_dir && !is_parent {
                format!("{}/", e.name)
            } else {
                e.name.clone()
            };

            let name_style = if sel {
                Style::default()
                    .fg(TEXT_BRIGHT)
                    .bg(bg)
                    .add_modifier(Modifier::BOLD)
            } else if is_parent {
                Style::default().fg(MUTED).bg(bg)
            } else if e.is_dir {
                Style::default().fg(TEXT).bg(bg)
            } else if e.is_hidden {
                Style::default().fg(MUTED).bg(bg)
            } else {
                Style::default().fg(TEXT_DIM).bg(bg)
            };

            let size = if e.is_dir || is_parent {
                "        ".to_string()
            } else {
                format!("{:>9}", format_size(e.size))
            };
            let time = if is_parent {
                "     ".to_string()
            } else {
                format!("{:>5}", format_time(e.modified))
            };

            let badge = if e.is_symlink {
                " l"
            } else if e.is_executable && !e.is_dir {
                " x"
            } else if e.is_readonly && !e.is_dir {
                " r"
            } else {
                "  "
            };

            let mut spans = vec![
                rail_span(sel),
                Span::styled(" ", Style::default().bg(bg)),
                Span::styled(format!("{icon} "), Style::default().fg(TEXT_DIM).bg(bg)),
            ];
            if filter_lower.is_empty() || is_parent {
                spans.push(Span::styled(fit_cell(&label, cols.name_w), name_style));
            } else {
                spans.extend(highlight_spans(
                    &label,
                    &filter_lower,
                    cols.name_w,
                    name_style,
                    sel,
                    bg,
                ));
            }
            spans.extend(vec![
                Span::styled(badge, Style::default().fg(MUTED).bg(bg)),
                Span::styled(size, Style::default().fg(GHOST).bg(bg)),
                Span::styled(format!(" {time}"), Style::default().fg(GHOST).bg(bg)),
            ]);

            let line = Line::from(spans);

            ListItem::new(line).style(Style::default().bg(bg))
        })
        .collect();

    let track_w = if state.entries.len() > visible { 1 } else { 0 };
    let list_rect = Rect {
        width: area.width.saturating_sub(track_w),
        ..area
    };

    if track_w > 0 {
        draw_scroll_track(
            f,
            Rect::new(list_rect.x + list_rect.width, area.y, 1, area.height),
            state.entries.len(),
            scroll_offset,
            visible,
        );
    }

    let mut ls = ListState::default();
    ls.select(Some(
        selected
            .saturating_sub(scroll_offset)
            .min(items.len().saturating_sub(1)),
    ));
    f.render_stateful_widget(
        List::new(items)
            .highlight_style(Style::default().bg(BG_SELECTED))
            .highlight_symbol(""),
        list_rect,
        &mut ls,
    );
    list_rect
}

fn draw_preview_panel(f: &mut Frame, state: &mut ExplorerState, area: Rect) {
    if area.height < 2 {
        return;
    }

    let entry = state.selected_entry();
    let title = entry
        .map(|e| {
            if ExplorerState::is_parent_entry(e) {
                "..".to_string()
            } else if e.is_dir {
                format!("{}/", e.name)
            } else {
                e.name.clone()
            }
        })
        .unwrap_or_else(|| "—".to_string());

    let head = Rect::new(area.x, area.y, area.width, 1);
    let body = Rect::new(area.x, area.y + 1, area.width, area.height.saturating_sub(1));
    draw_section_head(f, head, "PREVIEW", &title);

    if entry.is_none() {
        f.render_widget(
            Paragraph::new(Span::styled("  —", Style::default().fg(MUTED))),
            body,
        );
        return;
    }

    let pw = body.width as usize;
    let ph = body.height as usize;
    let (lines, kind, info) = state.get_preview(pw, ph);
    let lines = lines.to_vec();
    let kind = kind.clone();
    let info = info.to_vec();

    match kind {
        PreviewKind::Image => {
            let info_h = (info.len() as u16 + 1).min(4);
            let split = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(info_h), Constraint::Min(1)])
                .split(body);
            render_info_lines(f, &info, split[0]);
            render_image(f, &lines, split[1]);
        }
        PreviewKind::Video => {
            let info_h = (info.len() as u16 + 1).min(6);
            let split = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(info_h), Constraint::Min(1)])
                .split(body);
            render_info_lines(f, &info, split[0]);
            if lines.iter().any(|l| l.contains("\x1b[")) {
                render_image(f, &lines, split[1]);
            } else {
                render_plain_lines(f, &lines, split[1]);
            }
        }
        PreviewKind::Text => {
            let text: Vec<Line> = lines
                .iter()
                .take(ph)
                .enumerate()
                .map(|(i, line)| {
                    Line::from(vec![
                        Span::styled(format!(" {:>3} ", i + 1), Style::default().fg(GHOST)),
                        Span::styled("│ ", Style::default().fg(BORDER_DIM)),
                        Span::styled(
                            line.to_string(),
                            Style::default().fg(syntax_tone(line)),
                        ),
                    ])
                })
                .collect();
            if text.is_empty() {
                f.render_widget(
                    Paragraph::new(Span::styled("  empty file", Style::default().fg(MUTED))),
                    body,
                );
            } else {
                f.render_widget(Paragraph::new(text).wrap(Wrap { trim: false }), body);
            }
        }
        PreviewKind::Directory
        | PreviewKind::Audio
        | PreviewKind::Binary
        | PreviewKind::TooLarge
        | PreviewKind::Empty => {
            render_plain_lines(f, &lines, body);
        }
    }
}

fn render_info_lines(f: &mut Frame, info: &[String], area: Rect) {
    let lines: Vec<Line> = info
        .iter()
        .map(|l| Line::from(Span::styled(format!("  {l}"), Style::default().fg(TEXT_DIM))))
        .collect();
    f.render_widget(Paragraph::new(lines), area);
}

fn render_plain_lines(f: &mut Frame, lines: &[String], area: Rect) {
    let tl: Vec<Line> = lines
        .iter()
        .take(area.height as usize)
        .map(|l| Line::from(Span::styled(format!("  {l}"), Style::default().fg(TEXT_DIM))))
        .collect();
    f.render_widget(Paragraph::new(tl), area);
}

fn render_image(f: &mut Frame, lines: &[String], area: Rect) {
    let rendered: Vec<Line> = lines
        .iter()
        .take(area.height as usize)
        .map(|l| Line::from(parse_ansi(l)))
        .collect();
    f.render_widget(Paragraph::new(rendered), area);
}

fn draw_footer(f: &mut Frame, state: &ExplorerState, area: Rect) {
    if let Some(ref msg) = state.status_msg {
        f.render_widget(
            Paragraph::new(Span::styled(
                format!("  {}", msg.trim()),
                Style::default().fg(TEXT_DIM),
            )),
            area,
        );
        return;
    }

    let trail = state.selected_entry().map(|e| {
        if ExplorerState::is_parent_entry(e) {
            "..".to_string()
        } else {
            e.name.clone()
        }
    });

    draw_hint_bar(
        f,
        area,
        &[
            Hint { key: "↑↓", label: "nav" },
            Hint { key: "⏎", label: "open" },
            Hint { key: "⌫", label: "up" },
            Hint { key: "/", label: "find" },
            Hint { key: "v", label: "preview" },
            Hint { key: "y", label: "yank" },
            Hint { key: "p", label: "paste" },
            Hint { key: "q", label: "quit" },
        ],
        trail.as_deref(),
    );
}

fn fit_cell(text: &str, width: usize) -> String {
    let chars: String = text.chars().take(width).collect();
    if text.chars().count() <= width {
        format!("{chars:<width$}")
    } else {
        chars
    }
}

fn highlight_spans(
    text: &str,
    q: &str,
    width: usize,
    normal: Style,
    sel: bool,
    bg: Color,
) -> Vec<Span<'static>> {
    let hi = Style::default()
        .fg(TEXT_BRIGHT)
        .bg(bg)
        .add_modifier(if sel { Modifier::BOLD } else { Modifier::empty() });
    let display = fit_cell(text, width);
    let lower = display.to_lowercase();
    if let Some(p) = lower.find(q) {
        let chars: Vec<char> = display.chars().collect();
        let pre: String = chars[..p].iter().collect();
        let mid: String = chars[p..p + q.len().min(chars.len().saturating_sub(p))].iter().collect();
        let post: String = chars[p + q.len()..].iter().collect();
        return vec![
            Span::styled(pre, normal),
            Span::styled(mid, hi),
            Span::styled(post, normal),
        ];
    }
    vec![Span::styled(display, normal)]
}

fn syntax_tone(line: &str) -> Color {
    let t = line.trim();
    if t.starts_with("//")
        || t.starts_with('#')
        || t.starts_with("--")
        || t.starts_with("/*")
    {
        MUTED
    } else if t.starts_with("fn ")
        || t.starts_with("def ")
        || t.starts_with("pub ")
        || t.starts_with("class ")
        || t.starts_with("struct ")
    {
        TEXT_BRIGHT
    } else if t.starts_with("import ") || t.starts_with("use ") || t.starts_with("from ") {
        TEXT
    } else {
        TEXT_DIM
    }
}

fn parse_ansi(input: &str) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    let mut rest = input;
    let mut fg = TEXT_DIM;
    while !rest.is_empty() {
        if let Some(p) = rest.find("\x1b[") {
            if p > 0 {
                spans.push(Span::styled(rest[..p].to_string(), Style::default().fg(fg)));
            }
            let after = &rest[p + 2..];
            if let Some(m) = after.find('m') {
                if &after[..m] == "0" {
                    fg = TEXT_DIM;
                }
                rest = &after[m + 1..];
            } else {
                break;
            }
        } else {
            spans.push(Span::styled(rest.to_string(), Style::default().fg(fg)));
            break;
        }
    }
    if spans.is_empty() {
        spans.push(Span::raw(""));
    }
    spans
}

fn draw_open_with(f: &mut Frame, state: &ExplorerState, area: Rect) {
    let name = state
        .selected_entry()
        .map(|e| e.name.clone())
        .unwrap_or_default();
    let popup = centered(50, (state.openers.len() as u16 + 5).min(area.height - 2), area);
    let inner = draw_modal_shell(f, popup, "open with", &name);
    let lines: Vec<Line> = state
        .openers
        .iter()
        .enumerate()
        .map(|(i, o)| {
            let sel = i == state.open_with_selected;
            Line::from(Span::styled(
                format!("  {}  {}", o.name, o.command),
                if sel {
                    Style::default().fg(TEXT_BRIGHT).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(TEXT_DIM)
                },
            ))
        })
        .collect();
    f.render_widget(Paragraph::new(lines), inner);
}

fn draw_dialog(f: &mut Frame, dialog: &Dialog, input: &str, area: Rect) {
    let title = match dialog {
        Dialog::NewFile => "new file",
        Dialog::NewDir => "new folder",
        Dialog::Rename => "rename",
    };
    let popup = centered(44, 5, area);
    let inner = draw_modal_shell(f, popup, title, "");
    let blink = crate::ui::chrome::cursor_blink();
    f.render_widget(
        Paragraph::new(Span::styled(
            format!("  {}{}", input, if blink { "▍" } else { "" }),
            Style::default().fg(TEXT_BRIGHT),
        )),
        inner,
    );
}

fn draw_confirm(f: &mut Frame, confirm: &Confirm, area: Rect) {
    let popup = centered(44, 5, area);
    match confirm {
        Confirm::Delete { name, .. } => {
            let inner = draw_modal_shell(f, popup, "delete", name);
            f.render_widget(
                Paragraph::new(Span::styled("  y / esc", Style::default().fg(MUTED))),
                inner,
            );
        }
        Confirm::Overwrite { src, .. } => {
            let name = src
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            let inner = draw_modal_shell(f, popup, "overwrite", &name);
            f.render_widget(
                Paragraph::new(Span::styled("  y / esc", Style::default().fg(MUTED))),
                inner,
            );
        }
    }
}

fn centered(percent_x: u16, height: u16, r: Rect) -> Rect {
    let v = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(
                (100u16.saturating_sub(height.min(r.height) * 100 / r.height.max(1))) / 2,
            ),
            Constraint::Length(height),
            Constraint::Min(0),
        ])
        .split(r);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(v[1])[1]
}