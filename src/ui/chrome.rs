// src/ui/chrome.rs — VOID design system primitives

use ratatui::{
    layout::{Alignment, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Clear, Paragraph},
    Frame,
};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::config::*;

pub struct Hint<'a> {
    pub key: &'a str,
    pub label: &'a str,
}

pub fn clear(f: &mut Frame, area: Rect) {
    f.render_widget(Block::default().style(Style::default().bg(BG)), area);
}

pub fn inset(area: Rect, h: u16, v: u16) -> Rect {
    area.inner(Margin::new(h, v))
}

pub fn spinner(tick: u64) -> &'static str {
    const F: [&str; 8] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧"];
    F[(tick as usize) % F.len()]
}

pub fn cursor_blink() -> bool {
    (SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_millis()
        / 530)
        % 2
        == 0
}

pub fn progress_meter(value: f64, max: f64, width: usize) -> String {
    let filled = ((value / max.max(0.001)) * width as f64).round() as usize;
    let filled = filled.min(width);
    format!(
        "{}{}",
        "█".repeat(filled),
        "░".repeat(width.saturating_sub(filled))
    )
}

pub fn draw_brand_bar(f: &mut Frame, area: Rect, product: &str, context: &str, badge: Option<&str>) {
    let mut spans = vec![
        Span::styled(product, Style::default().fg(TEXT_BRIGHT).add_modifier(Modifier::BOLD)),
        Span::styled("  ·  ", Style::default().fg(GHOST)),
        Span::styled(context, Style::default().fg(MUTED)),
    ];
    if let Some(b) = badge {
        let pad = area.width.saturating_sub(
            (product.len() + context.len() + b.len() + 7) as u16,
        );
        if pad > 0 {
            spans.push(Span::raw(" ".repeat(pad as usize)));
        }
        spans.push(Span::styled(b, Style::default().fg(GHOST)));
    }
    f.render_widget(
        Paragraph::new(Line::from(spans)).style(Style::default().bg(BG_SURFACE)),
        area,
    );
}

pub fn draw_rule(f: &mut Frame, area: Rect) {
    let w = area.width as usize;
    if w == 0 {
        return;
    }
    f.render_widget(
        Paragraph::new(Span::styled("─".repeat(w), Style::default().fg(RULE))),
        area,
    );
}

pub fn draw_section_head(f: &mut Frame, area: Rect, left: &str, right: &str) {
    let w = area.width as usize;
    let gap = w.saturating_sub(left.len() + right.len() + 2);
    let line = if gap > 2 {
        format!("{} {} {}", left, "·".repeat(gap.min(48)), right)
    } else {
        format!("{}  {}", left, right)
    };
    f.render_widget(
        Paragraph::new(Span::styled(line, Style::default().fg(GHOST))),
        area,
    );
}

pub fn draw_command_field(
    f: &mut Frame,
    area: Rect,
    icon: &str,
    input: &str,
    cursor: usize,
    placeholder: &str,
    searching: bool,
    tick: u64,
) {
    draw_rounded_frame(f, area, BG_INSET);

    let inner = area.inner(Margin::new(2, 1));
    if inner.height == 0 || inner.width < 4 {
        return;
    }

    let blink = cursor_blink();
    let cursor_ch = if blink { "▍" } else { " " };
    let status = if searching {
        spinner(tick)
    } else if input.is_empty() {
        "○"
    } else {
        "●"
    };

    let before = &input[..cursor.min(input.len())];
    let after = &input[cursor.min(input.len())..];

    let mut spans = vec![
        Span::styled(
            format!("{} ", status),
            Style::default()
                .fg(if searching { ACCENT_SOFT } else { TEXT_DIM })
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!("{} ", icon), Style::default().fg(GHOST)),
    ];

    if input.is_empty() && !searching {
        spans.push(Span::styled(placeholder, Style::default().fg(MUTED)));
        if blink {
            spans.push(Span::styled(cursor_ch, Style::default().fg(TEXT).bg(BG_SELECTED)));
        }
    } else {
        spans.push(Span::styled(before, Style::default().fg(TEXT_BRIGHT)));
        spans.push(Span::styled(
            cursor_ch,
            Style::default().fg(TEXT).bg(BG_SELECTED),
        ));
        spans.push(Span::styled(after, Style::default().fg(TEXT_BRIGHT)));
    }

    f.render_widget(Paragraph::new(Line::from(spans)), inner);
}

pub fn draw_rounded_frame(f: &mut Frame, area: Rect, fill: Color) {
    if area.width < 2 || area.height < 2 {
        f.render_widget(Block::default().style(Style::default().bg(fill)), area);
        return;
    }

    let w = area.width as usize;
    let top = format!("╭{}╮", "─".repeat(w.saturating_sub(2)));
    let bot = format!("╰{}╯", "─".repeat(w.saturating_sub(2)));
    let side = "│";

    f.render_widget(
        Paragraph::new(Span::styled(top, Style::default().fg(BORDER_DIM).bg(fill))),
        Rect::new(area.x, area.y, area.width, 1),
    );
    for row in 1..area.height.saturating_sub(1) {
        let y = area.y + row;
        f.render_widget(
            Paragraph::new(Span::styled(side, Style::default().fg(BORDER_DIM).bg(fill))),
            Rect::new(area.x, y, 1, 1),
        );
        if w > 2 {
            f.render_widget(
                Block::default().style(Style::default().bg(fill)),
                Rect::new(area.x + 1, y, area.width - 2, 1),
            );
        }
        f.render_widget(
            Paragraph::new(Span::styled(side, Style::default().fg(BORDER_DIM).bg(fill))),
            Rect::new(area.x + area.width - 1, y, 1, 1),
        );
    }
    f.render_widget(
        Paragraph::new(Span::styled(bot, Style::default().fg(BORDER_DIM).bg(fill))),
        Rect::new(area.x, area.y + area.height - 1, area.width, 1),
    );
}

pub fn draw_card_frame(f: &mut Frame, area: Rect, title: &str) -> Rect {
    draw_rounded_frame(f, area, BG_CARD);
    if area.height < 3 || area.width < 4 {
        return area;
    }
    let title_area = Rect::new(area.x + 2, area.y, area.width.saturating_sub(4), 1);
    f.render_widget(
        Paragraph::new(Span::styled(
            format!(" {} ", title),
            Style::default().fg(TEXT_DIM).add_modifier(Modifier::BOLD),
        )),
        title_area,
    );
    Rect::new(
        area.x + 2,
        area.y + 1,
        area.width.saturating_sub(4),
        area.height.saturating_sub(2),
    )
}

pub fn draw_hint_bar(f: &mut Frame, area: Rect, hints: &[Hint], trail: Option<&str>) {
    let mut spans: Vec<Span> = Vec::new();
    for (i, h) in hints.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled("   ", Style::default().fg(GHOST)));
        }
        spans.push(Span::styled(
            h.key,
            Style::default().fg(TEXT_BRIGHT).add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(
            format!(" {}", h.label),
            Style::default().fg(MUTED),
        ));
    }
    if let Some(t) = trail {
        spans.push(Span::styled("    ", Style::default()));
        spans.push(Span::styled(t, Style::default().fg(GHOST)));
    }
    f.render_widget(
        Paragraph::new(Line::from(spans))
            .style(Style::default().bg(BG_SURFACE))
            .alignment(Alignment::Left),
        area,
    );
}

pub fn idx_span(i: usize, selected: bool) -> Span<'static> {
    if selected {
        Span::styled(
            format!("{:>02} ", i + 1),
            Style::default()
                .fg(TEXT_BRIGHT)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        Span::styled(
            format!("{:>02} ", i + 1),
            Style::default().fg(GHOST),
        )
    }
}

pub fn rail_span(selected: bool) -> Span<'static> {
    if selected {
        Span::styled(
            "▎",
            Style::default()
                .fg(RAIL)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        Span::styled(" ", Style::default())
    }
}

pub fn draw_empty_state(f: &mut Frame, area: Rect, glyph: &str, title: &str, subtitle: &str) {
    let inner = inset(area, 2, 1);
    let lines = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            format!("  {}  ", glyph),
            Style::default().fg(TEXT_DIM),
        )]),
        Line::from(vec![Span::styled(
            format!("  {}", title),
            Style::default().fg(TEXT).add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![Span::styled(
            format!("  {}", subtitle),
            Style::default().fg(MUTED),
        )]),
    ];
    f.render_widget(Paragraph::new(lines), inner);
}

pub fn draw_scroll_track(
    f: &mut Frame,
    track: Rect,
    total: usize,
    scroll_offset: usize,
    visible: usize,
) {
    if total <= visible || track.height == 0 {
        return;
    }
    let th = track.height as f64;
    let thumb = ((visible as f64 / total as f64) * th).max(1.0) as u16;
    let top = ((scroll_offset as f64 / total as f64) * th) as u16;

    for y in 0..track.height {
        let ch = if y >= top && y < top.saturating_add(thumb) {
            "┃"
        } else {
            "│"
        };
        let color = if ch == "┃" { TEXT_DIM } else { BORDER_DIM };
        f.render_widget(
            Paragraph::new(Span::styled(ch, Style::default().fg(color))),
            Rect::new(track.x, track.y + y, 1, 1),
        );
    }
}

pub fn draw_modal_shell(f: &mut Frame, area: Rect, title: &str, subtitle: &str) -> Rect {
    f.render_widget(Clear, area);
    draw_rounded_frame(f, area, BG_PANEL);

    if area.height < 4 || area.width < 6 {
        return area;
    }

    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(
                format!("  {}  ", title),
                Style::default().fg(TEXT_BRIGHT).add_modifier(Modifier::BOLD),
            ),
            Span::styled(subtitle, Style::default().fg(MUTED)),
        ])),
        Rect::new(area.x + 1, area.y + 1, area.width.saturating_sub(2), 1),
    );

    Rect::new(
        area.x + 2,
        area.y + 2,
        area.width.saturating_sub(4),
        area.height.saturating_sub(3),
    )
}

pub fn draw_jump_success(f: &mut Frame, area: Rect, parent: &str, name: &str) {
    clear(f, area);
    let card_h = 7u16;
    let card_w = 60u16.min(area.width.saturating_sub(4));
    let x = area.x + (area.width.saturating_sub(card_w)) / 2;
    let y = area.y + (area.height.saturating_sub(card_h)) / 2;
    let card = Rect::new(x, y, card_w, card_h);

    draw_rounded_frame(f, card, BG_CARD);
    let inner = card.inner(Margin::new(3, 2));
    f.render_widget(
        Paragraph::new(vec![
            Line::from(vec![Span::styled(
                "  JUMPED",
                Style::default().fg(GHOST).add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![
                Span::styled(parent, Style::default().fg(MUTED)),
                Span::styled(
                    name,
                    Style::default().fg(TEXT_BRIGHT).add_modifier(Modifier::BOLD),
                ),
            ]),
        ]),
        inner,
    );
}

pub fn breadcrumb_spans(path_display: &str) -> Vec<Span<'static>> {
    let parts: Vec<&str> = path_display.split('/').collect();
    let mut spans = Vec::new();
    for (i, part) in parts.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled(" / ", Style::default().fg(GHOST)));
        }
        let style = if i == parts.len() - 1 {
            Style::default().fg(TEXT_BRIGHT).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(TEXT_DIM)
        };
        spans.push(Span::styled(part.to_string(), style));
    }
    spans
}

pub fn table_header_spans(cols: &[(&str, u16)]) -> Line<'static> {
    let mut spans = vec![Span::styled("  ", Style::default())];
    for (label, width) in cols {
        spans.push(Span::styled(
            format!("{:<width$}", label, width = *width as usize),
            Style::default().fg(GHOST).add_modifier(Modifier::BOLD),
        ));
    }
    Line::from(spans)
}