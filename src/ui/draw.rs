// src/ui/draw.rs

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders},
    Frame,
};

use crate::app::App;
use crate::config::*;
use crate::ui::components::*;

pub fn draw(f: &mut Frame, app: &mut App) {
    let area = f.area();
    f.render_widget(Block::default().style(Style::default().bg(BG)), area);

    let tq = if app.input.is_empty() {
        Span::styled(" type to search... ", Style::default().fg(MUTED))
    } else {
        Span::styled(format!(" \"{}\" ", app.input),
            Style::default().fg(PHOSPHOR_BRIGHT).add_modifier(Modifier::BOLD))
    };

    f.render_widget(Block::default().borders(Borders::ALL)
        .border_style(Style::default().fg(PHOSPHOR_DIM))
        .title(Line::from(vec![
            Span::styled(" ⚡ jump ", Style::default().fg(PHOSPHOR).add_modifier(Modifier::BOLD)),
            tq,
        ])).title_alignment(Alignment::Left).style(Style::default().bg(BG_PANEL)), area);

    let inner = area.inner(Margin { horizontal: 1, vertical: 1 });
    let chunks = Layout::default().direction(Direction::Vertical).constraints([
        Constraint::Length(3), Constraint::Min(4), Constraint::Length(1),
    ]).split(inner);

    draw_search_bar(f, app, chunks[0]);

    let pins = app.matching_pins();
    let has_pins = !pins.is_empty();
    let has_matches = !app.results.matches.is_empty();

    if !has_matches && has_pins {
        draw_pins_full(f, app, chunks[1], &pins);
    } else if has_pins && has_matches {
        let split = Layout::default().direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(25), Constraint::Percentage(75)])
            .split(chunks[1]);
        draw_pins_sidebar(f, split[0], &pins);
        draw_matches_panel(f, app, split[1]);
    } else {
        draw_matches_panel(f, app, chunks[1]);
    }

    draw_footer(f, app, chunks[2]);
}