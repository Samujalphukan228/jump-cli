// src/ui/components.rs

use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, ListState, Paragraph},
    Frame,
};
use std::path::PathBuf;

use crate::app::App;
use crate::config::*;
use crate::search::match_rank;
use crate::ui::chrome::{
    draw_card_frame, draw_command_field, draw_empty_state, draw_hint_bar,
    draw_jump_success, draw_scroll_track, draw_section_head, idx_span, rail_span,
    spinner, Hint,
};
use crate::utils::{rank_badge, shorten_path};

pub fn draw_search_bar(f: &mut Frame, app: &App, area: Rect) {
    draw_command_field(
        f,
        area,
        "⌕",
        &app.input,
        app.cursor,
        "where do you want to go?",
        app.results.searching,
        app.results.tick,
    );
}

pub fn draw_matches_panel(f: &mut Frame, app: &mut App, area: Rect) -> Rect {
    let searching = app.results.searching;
    let query = app.input.clone();
    let local_count = app.results.local_count;
    let tick = app.results.tick;
    let matches: Vec<PathBuf> = app.results.matches.clone();

    let right_label = if searching && matches.is_empty() {
        format!("{} scanning", spinner(tick))
    } else if matches.is_empty() && !query.is_empty() {
        "no matches".to_string()
    } else if matches.is_empty() {
        "awaiting input".to_string()
    } else {
        let total = matches.len();
        let ind = if searching { " · live" } else { "" };
        if local_count > 0 && local_count < total {
            format!(
                "{} results · {} local · {} global{}",
                total,
                local_count,
                total - local_count,
                ind
            )
        } else {
            format!("{} results{}", total, ind)
        }
    };

    let head = Rect::new(area.x, area.y, area.width, 1);
    let body = Rect::new(area.x, area.y + 1, area.width, area.height.saturating_sub(1));

    draw_section_head(f, head, "RESULTS", &right_label);

    if query.is_empty() {
        draw_frecency_suggestions(f, app, body);
        return body;
    }

    if matches.is_empty() {
        if searching {
            draw_empty_state(f, body, spinner(tick), "Searching filesystem", "Results appear as they are found");
        } else {
            draw_empty_state(f, body, "∅", "Nothing matched", "Try a shorter query or different segment");
        }
        return body;
    }

    let list_height = body.height as usize;
    app.adjust_scroll(list_height);
    let scroll_offset = app.scroll_offset;
    let selected = app.selected;

    let items: Vec<ListItem> = matches
        .iter()
        .enumerate()
        .skip(scroll_offset)
        .take(list_height)
        .map(|(idx, path)| {
            let is_sel = idx == selected;
            let is_local = idx < local_count;
            let bg = if is_sel { BG_SELECTED } else { BG };
            let ps = path.to_string_lossy();
            let score = app.frecency_score(&ps);
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| ps.to_string());
            let dp = shorten_path(path);
            let parent = dp
                .rfind('/')
                .map(|p| format!("{}/", &dp[..p]))
                .unwrap_or_default();
            let rank = match_rank(path, &query);
            let (bt, bc) = rank_badge(rank);

            let freq = if score > 0.0 {
                Span::styled(
                    format!("{:>4.0} ", score),
                    Style::default().fg(ACCENT_SOFT).bg(bg),
                )
            } else {
                Span::styled("     ", Style::default().bg(bg))
            };

            let scope = if is_local {
                Span::styled("near ", Style::default().fg(TEXT_DIM).bg(bg))
            } else {
                Span::styled("     ", Style::default().bg(bg))
            };

            ListItem::new(Line::from(vec![
                rail_span(is_sel),
                idx_span(idx, is_sel),
                Span::styled(
                    format!("{:<6} ", bt.trim()),
                    Style::default().fg(bc).bg(bg),
                ),
                scope,
                freq,
                Span::styled(parent, Style::default().fg(MUTED).bg(bg)),
                Span::styled(
                    name,
                    if is_sel {
                        Style::default()
                            .fg(TEXT_BRIGHT)
                            .bg(bg)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(TEXT).bg(bg)
                    },
                ),
            ]))
            .style(Style::default().bg(bg))
        })
        .collect();

    let track_w = if matches.len() > list_height { 1 } else { 0 };
    let list_area = Rect {
        width: body.width.saturating_sub(track_w),
        ..body
    };

    if track_w > 0 {
        draw_scroll_track(
            f,
            Rect::new(body.x + list_area.width, body.y, 1, body.height),
            matches.len(),
            scroll_offset,
            list_height,
        );
    }

    let mut ls = ListState::default();
    ls.select(Some(
        selected
            .saturating_sub(scroll_offset)
            .min(items.len().saturating_sub(1)),
    ));
    f.render_stateful_widget(
        List::new(items).highlight_style(
            Style::default()
                .bg(BG_SELECTED)
                .fg(TEXT_BRIGHT)
                .add_modifier(Modifier::BOLD),
        ),
        list_area,
        &mut ls,
    );
    list_area
}

pub fn draw_frecency_suggestions(f: &mut Frame, app: &App, area: Rect) {
    let mut entries: Vec<(&String, f64)> = app
        .frecency
        .iter()
        .map(|(k, _)| (k, app.frecency_score(k)))
        .filter(|(_, s)| *s > 0.0)
        .collect();
    entries.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    if entries.is_empty() {
        draw_empty_state(
            f,
            area,
            "⌕",
            "Instant directory search",
            "Type a folder name — src · api · config · docs",
        );
        return;
    }

    let max = entries.first().map(|(_, s)| *s).unwrap_or(1.0);
    let max_rows = area.height as usize;
    let mut lines: Vec<Line> = vec![Line::from(vec![Span::styled(
        "  RECENT",
        Style::default().fg(GHOST).add_modifier(Modifier::BOLD),
    )])];

    for (i, (path, score)) in entries.iter().take(max_rows.saturating_sub(1)).enumerate() {
        let pb = PathBuf::from(path.as_str());
        let name = pb
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string());
        let short = shorten_path(&pb);
        let parent = short
            .rfind('/')
            .map(|p| format!("{}/", &short[..p]))
            .unwrap_or_default();
        let bar = crate::ui::chrome::progress_meter(*score, max, 8);
        lines.push(Line::from(vec![
            Span::styled(format!("  {:>02} ", i + 1), Style::default().fg(GHOST)),
            Span::styled(format!("{} ", bar), Style::default().fg(TEXT_DIM)),
            Span::styled(format!("{:>4.0} ", score), Style::default().fg(ACCENT_SOFT)),
            Span::styled(parent, Style::default().fg(MUTED)),
            Span::styled(name, Style::default().fg(TEXT)),
        ]));
    }
    f.render_widget(Paragraph::new(lines), area);
}

pub fn draw_pins_sidebar(f: &mut Frame, area: Rect, pins: &[(&String, &String)]) {
    let inner = draw_card_frame(f, area, &format!("PINS · {}", pins.len()));

    let items: Vec<ListItem> = pins
        .iter()
        .take(inner.height as usize)
        .map(|(name, path)| {
            let pb = PathBuf::from(path.as_str());
            let dn = pb
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| path.to_string());
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("@{}", name),
                    Style::default().fg(TEXT_BRIGHT).add_modifier(Modifier::BOLD),
                ),
                Span::styled(" → ", Style::default().fg(GHOST)),
                Span::styled(dn, Style::default().fg(TEXT_DIM)),
            ]))
        })
        .collect();
    f.render_widget(List::new(items), inner);
}

pub fn draw_pins_full(f: &mut Frame, app: &App, area: Rect, pins: &[(&String, &String)]) -> Rect {
    let inner = draw_card_frame(f, area, &format!("PINS · {}", pins.len()));

    if pins.is_empty() {
        draw_empty_state(
            f,
            inner,
            "@",
            "No pins yet",
            "jump --pin work  to bookmark a folder",
        );
        return inner;
    }

    let items: Vec<ListItem> = pins
        .iter()
        .enumerate()
        .map(|(idx, (name, path))| {
            let is_sel = idx == app.selected;
            let bg = if is_sel { BG_SELECTED } else { BG_CARD };
            let short = shorten_path(&PathBuf::from(path.as_str()));
            ListItem::new(Line::from(vec![
                rail_span(is_sel),
                idx_span(idx, is_sel),
                Span::styled(
                    format!("@{:<10} ", name),
                    Style::default()
                        .fg(TEXT_BRIGHT)
                        .bg(bg)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("→ ", Style::default().fg(GHOST).bg(bg)),
                Span::styled(
                    short,
                    if is_sel {
                        Style::default()
                            .fg(TEXT_BRIGHT)
                            .bg(bg)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(TEXT).bg(bg)
                    },
                ),
            ]))
            .style(Style::default().bg(bg))
        })
        .collect();

    let mut ls = ListState::default();
    ls.select(Some(
        app.selected.min(items.len().saturating_sub(1)),
    ));
    f.render_stateful_widget(
        List::new(items)
            .highlight_style(Style::default().bg(BG_SELECTED))
            .highlight_symbol(""),
        inner,
        &mut ls,
    );
    inner
}

pub fn draw_footer(f: &mut Frame, app: &App, area: Rect) {
    let trail = app
        .selected_path()
        .map(|p| format!("→ {}", shorten_path(p)));
    draw_hint_bar(
        f,
        area,
        &[
            Hint { key: "↑↓", label: "move" },
            Hint { key: "⏎", label: "jump" },
            Hint { key: "esc", label: "close" },
            Hint { key: "^u", label: "clear" },
        ],
        trail.as_deref(),
    );
}

pub fn draw_jump_flash(f: &mut Frame, target: &PathBuf) {
    let short = shorten_path(target);
    let name = target
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    let parent = short
        .rfind('/')
        .map(|p| format!("{}/", &short[..p]))
        .unwrap_or_default();
    draw_jump_success(f, f.area(), &parent, &name);
}