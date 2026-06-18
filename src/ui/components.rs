// src/ui/components.rs

use ratatui::{
    layout::{Alignment, Margin, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::app::App;
use crate::config::*;
use crate::search::match_rank;
use crate::utils::{rank_badge, shorten_path};

pub fn draw_search_bar(f: &mut Frame, app: &App, area: Rect) {
    let spinner_frames = ["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"];
    let spinner = if app.results.searching {
        spinner_frames[(app.results.tick as usize) % spinner_frames.len()]
    } else { "●" };

    let spinner_color = if app.results.searching { AMBER }
        else if app.results.matches.is_empty() && !app.input.is_empty() { MUTED }
        else { GREEN };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(
            if app.results.searching { AMBER } else { PHOSPHOR_DIM }
        ));
    let block_inner = block.inner(area);
    f.render_widget(block, area);

    let blink = (SystemTime::now().duration_since(UNIX_EPOCH)
        .unwrap_or_default().subsec_millis() / 530) % 2 == 0;
    let cursor_char = if blink { "█" } else { "▏" };

    let mut spans = vec![
        Span::styled(format!(" {} ", spinner),
            Style::default().fg(spinner_color).add_modifier(Modifier::BOLD)),
        Span::styled("> ", Style::default().fg(PHOSPHOR_DIM)),
    ];

    let before = &app.input[..app.cursor];
    let after = &app.input[app.cursor..];
    spans.push(Span::styled(before, Style::default().fg(PHOSPHOR_BRIGHT)));
    spans.push(Span::styled(cursor_char, Style::default().fg(PHOSPHOR).bg(BG_SELECTED)));
    spans.push(Span::styled(after, Style::default().fg(PHOSPHOR_BRIGHT)));

    if app.input.is_empty() && !blink {
        spans.push(Span::styled("search directories...", Style::default().fg(MUTED)));
    }

    f.render_widget(Paragraph::new(Line::from(spans)), block_inner);
}

pub fn draw_matches_panel(f: &mut Frame, app: &mut App, area: Rect) {
    let searching = app.results.searching;
    let query = app.input.clone();
    let local_count = app.results.local_count;
    let tick = app.results.tick;
    let matches: Vec<PathBuf> = app.results.matches.clone();

    let count_str = if searching && matches.is_empty() { " scanning... ".to_string() }
        else if matches.is_empty() && !query.is_empty() { " no results ".to_string() }
        else if matches.is_empty() { " type to search ".to_string() }
        else {
            let total = matches.len();
            let ind = if searching { " ⟳" } else { "" };
            if local_count > 0 && local_count < total {
                format!(" {} local + {} global ({}){} ", local_count, total - local_count, total, ind)
            } else { format!(" {} matches{} ", total, ind) }
        };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(PHOSPHOR_DIM))
        .title(Span::styled(count_str, Style::default().fg(PHOSPHOR_DIM)))
        .style(Style::default().bg(BG_PANEL));
    let block_inner = block.inner(area);
    f.render_widget(block, area);

    if query.is_empty() {
        draw_frecency_suggestions(f, app, block_inner);
        return;
    }

    if matches.is_empty() {
        if searching {
            let frames = ["⣾","⣽","⣻","⢿","⡿","⣟","⣯","⣷"];
            let sp = frames[(tick as usize) % frames.len()];
            f.render_widget(Paragraph::new(Line::from(vec![
                Span::styled(format!("  {} ", sp), Style::default().fg(PHOSPHOR)),
                Span::styled("scanning...", Style::default().fg(MUTED)),
            ])), block_inner);
        } else {
            f.render_widget(Paragraph::new(vec![
                Line::from(""),
                Line::from(vec![Span::styled("  ✗ no matches", Style::default().fg(MUTED))]),
            ]), block_inner);
        }
        return;
    }

    let list_height = block_inner.height as usize;
    app.adjust_scroll(list_height);
    let scroll_offset = app.scroll_offset;
    let selected = app.selected;

    let items: Vec<ListItem> = matches.iter().enumerate()
        .skip(scroll_offset).take(list_height)
        .map(|(idx, path)| {
            let is_sel = idx == selected;
            let is_local = idx < local_count;
            let bg = if is_sel { BG_SELECTED } else { BG_PANEL };
            let ps = path.to_string_lossy();
            let score = app.frecency_score(&ps);
            let name = path.file_name().map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| ps.to_string());
            let dp = shorten_path(path);
            let parent = dp.rfind('/').map(|p| format!("{}/", &dp[..p])).unwrap_or_default();
            let rank = match_rank(path, &query);
            let (bt, bc) = rank_badge(rank);
            let freq = if score > 0.0 {
                Span::styled(format!("★{:<3.0} ", score), Style::default().fg(AMBER).bg(bg))
            } else { Span::styled("     ", Style::default().bg(bg)) };
            let loc = if is_local {
                Span::styled("◆ ", Style::default().fg(GREEN).bg(bg))
            } else { Span::styled("  ", Style::default().bg(bg)) };
            let sel = if is_sel {
                Span::styled("▶ ", Style::default().fg(PHOSPHOR_BRIGHT).bg(bg).add_modifier(Modifier::BOLD))
            } else { Span::styled("  ", Style::default().bg(bg)) };

            ListItem::new(Line::from(vec![
                sel, loc,
                Span::styled(format!("[{}] ", bt), Style::default().fg(bc).bg(bg).add_modifier(Modifier::DIM)),
                freq,
                Span::styled(parent, Style::default().fg(MUTED).bg(bg)),
                Span::styled(name, if is_sel {
                    Style::default().fg(PHOSPHOR_BRIGHT).bg(bg).add_modifier(Modifier::BOLD)
                } else { Style::default().fg(TEXT).bg(bg) }),
            ])).style(Style::default().bg(bg))
        }).collect();

    let total = matches.len();
    if total > list_height {
        let sb = Rect { x: block_inner.x, y: area.y, width: block_inner.width, height: 1 };
        f.render_widget(Paragraph::new(Line::from(vec![Span::styled(
            format!(" [{}-{}/{}] ", scroll_offset+1, (scroll_offset+list_height).min(total), total),
            Style::default().fg(PHOSPHOR_DIM),
        )])).alignment(Alignment::Right), sb);
    }

    let mut ls = ListState::default();
    ls.select(Some(selected.saturating_sub(scroll_offset).min(items.len().saturating_sub(1))));
    f.render_stateful_widget(List::new(items).highlight_style(
        Style::default().bg(BG_SELECTED).fg(PHOSPHOR_BRIGHT).add_modifier(Modifier::BOLD)
    ), block_inner, &mut ls);
}

pub fn draw_frecency_suggestions(f: &mut Frame, app: &App, area: Rect) {
    let mut entries: Vec<(&String, f64)> = app.frecency.iter()
        .map(|(k, _)| (k, app.frecency_score(k)))
        .filter(|(_, s)| *s > 0.0).collect();
    entries.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    if entries.is_empty() {
        f.render_widget(Paragraph::new(vec![
            Line::from(""),
            Line::from(vec![Span::styled("  Start typing to search", Style::default().fg(MUTED))]),
            Line::from(""),
            Line::from(vec![Span::styled("  e.g.  src  •  config  •  docs", Style::default().fg(PHOSPHOR_DIM))]),
        ]), area);
        return;
    }

    let max = area.height as usize;
    let mut lines: Vec<Line> = vec![
        Line::from(vec![Span::styled(" ★ recent — type to search",
            Style::default().fg(MUTED).add_modifier(Modifier::ITALIC))]),
        Line::from(""),
    ];
    for (path, score) in entries.iter().take(max.saturating_sub(2)) {
        let pb = PathBuf::from(path.as_str());
        let name = pb.file_name().map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string());
        let short = shorten_path(&pb);
        let parent = short.rfind('/').map(|p| format!("{}/", &short[..p])).unwrap_or_default();
        lines.push(Line::from(vec![
            Span::styled(format!("  ★{:<4.0} ", score), Style::default().fg(AMBER)),
            Span::styled(parent, Style::default().fg(MUTED)),
            Span::styled(name, Style::default().fg(TEXT)),
        ]));
    }
    f.render_widget(Paragraph::new(lines), area);
}

pub fn draw_pins_sidebar(f: &mut Frame, area: Rect, pins: &[(&String, &String)]) {
    let block = Block::default().borders(Borders::ALL)
        .border_style(Style::default().fg(PHOSPHOR_DIM))
        .title(Span::styled(format!(" ⬡ {} pins ", pins.len()), Style::default().fg(PHOSPHOR)))
        .style(Style::default().bg(BG_PANEL));
    let bi = block.inner(area);
    f.render_widget(block, area);

    let items: Vec<ListItem> = pins.iter().take(bi.height as usize).map(|(name, path)| {
        let pb = PathBuf::from(path.as_str());
        let dn = pb.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| path.to_string());
        ListItem::new(Line::from(vec![
            Span::styled(format!(" @{} ", name), Style::default().fg(PHOSPHOR).add_modifier(Modifier::BOLD)),
            Span::styled(dn, Style::default().fg(TEXT)),
        ]))
    }).collect();
    f.render_widget(List::new(items), bi);
}

pub fn draw_pins_full(f: &mut Frame, app: &App, area: Rect, pins: &[(&String, &String)]) {
    let block = Block::default().borders(Borders::ALL)
        .border_style(Style::default().fg(PHOSPHOR_DIM))
        .title(Span::styled(format!(" ⬡ pins ({}) ", pins.len()), Style::default().fg(PHOSPHOR)))
        .style(Style::default().bg(BG_PANEL));
    let bi = block.inner(area);
    f.render_widget(block, area);

    if pins.is_empty() {
        f.render_widget(Paragraph::new(Line::from(vec![
            Span::styled("  No pins. Use: jump --pin <name> [path]", Style::default().fg(MUTED)),
        ])), bi);
        return;
    }

    let items: Vec<ListItem> = pins.iter().enumerate().map(|(idx, (name, path))| {
        let is_sel = idx == app.selected;
        let bg = if is_sel { BG_SELECTED } else { BG_PANEL };
        let short = shorten_path(&PathBuf::from(path.as_str()));
        ListItem::new(Line::from(vec![
            if is_sel { Span::styled("▶ ", Style::default().fg(PHOSPHOR_BRIGHT).bg(bg)) }
            else { Span::styled("  ", Style::default().bg(bg)) },
            Span::styled(format!("@{:<12}", name), Style::default().fg(PHOSPHOR).bg(bg).add_modifier(Modifier::BOLD)),
            Span::styled("→ ", Style::default().fg(PHOSPHOR_DIM).bg(bg)),
            Span::styled(short, if is_sel {
                Style::default().fg(PHOSPHOR_BRIGHT).bg(bg).add_modifier(Modifier::BOLD)
            } else { Style::default().fg(TEXT).bg(bg) }),
        ])).style(Style::default().bg(bg))
    }).collect();

    let mut ls = ListState::default();
    ls.select(Some(app.selected.min(items.len().saturating_sub(1))));
    f.render_stateful_widget(List::new(items).highlight_style(Style::default().bg(BG_SELECTED)).highlight_symbol(""), bi, &mut ls);
}

pub fn draw_footer(f: &mut Frame, app: &App, area: Rect) {
    let mut spans = vec![
        Span::styled(" ↑↓", Style::default().fg(PHOSPHOR)),
        Span::styled(" nav  ", Style::default().fg(MUTED)),
        Span::styled("pgup/dn", Style::default().fg(PHOSPHOR)),
        Span::styled(" page  ", Style::default().fg(MUTED)),
        Span::styled("enter", Style::default().fg(PHOSPHOR)),
        Span::styled(" go  ", Style::default().fg(MUTED)),
        Span::styled("esc", Style::default().fg(PHOSPHOR)),
        Span::styled(" quit  ", Style::default().fg(MUTED)),
        Span::styled("^u", Style::default().fg(PHOSPHOR)),
        Span::styled(" clear", Style::default().fg(MUTED)),
    ];
    if let Some(p) = app.selected_path() {
        spans.push(Span::styled(format!("  → {}", shorten_path(p)), Style::default().fg(PHOSPHOR_DIM)));
    }
    f.render_widget(Paragraph::new(Line::from(spans)), area);
}

pub fn draw_jump_flash(f: &mut Frame, target: &PathBuf) {
    let area = f.area();
    f.render_widget(Block::default().style(Style::default().bg(BG)), area);
    let center = crate::ui::helpers::centered_rect(70, 5, area);
    let name = target.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
    let short = shorten_path(target);
    let parent = short.rfind('/').map(|p| format!("{}/", &short[..p])).unwrap_or_default();
    let block = Block::default().borders(Borders::ALL)
        .border_style(Style::default().fg(GREEN)).style(Style::default().bg(BG_PANEL));
    f.render_widget(block, center);
    let inner = center.inner(Margin { horizontal: 2, vertical: 1 });
    f.render_widget(Paragraph::new(Line::from(vec![
        Span::styled("→  ", Style::default().fg(GREEN).add_modifier(Modifier::BOLD)),
        Span::styled(parent, Style::default().fg(MUTED)),
        Span::styled(name, Style::default().fg(PHOSPHOR_BRIGHT).add_modifier(Modifier::BOLD)),
    ])).alignment(Alignment::Left), inner);
}