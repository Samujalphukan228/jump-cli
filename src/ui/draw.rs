// src/ui/draw.rs

use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

use crate::app::App;
use crate::ui::chrome::{clear, draw_brand_bar, draw_rule, inset};
use crate::ui::components::*;

pub fn draw(f: &mut Frame, app: &mut App) {
    let area = f.area();
    clear(f, area);

    let shell = inset(area, 1, 0);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Min(4),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(shell);

    let cwd = std::env::current_dir()
        .ok()
        .map(|p| crate::utils::shorten_path(&p))
        .unwrap_or_else(|| "~".to_string());

    draw_brand_bar(f, chunks[0], "JUMP", &cwd, Some("v0.4.0"));
    draw_rule(f, chunks[1]);
    app.layout.search = chunks[2];
    app.layout.search_text_x = chunks[2].x + 5;
    draw_search_bar(f, app, chunks[2]);
    draw_rule(f, chunks[3]);

    let pins = app.matching_pins();
    let has_pins = !pins.is_empty();
    let has_matches = !app.results.matches.is_empty();

    if !has_matches && has_pins {
        app.layout.list = draw_pins_full(f, app, chunks[4], &pins);
    } else if has_pins && has_matches {
        let split = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(28), Constraint::Percentage(72)])
            .split(chunks[4]);
        draw_pins_sidebar(f, split[0], &pins);
        app.layout.list = draw_matches_panel(f, app, split[1]);
    } else {
        app.layout.list = draw_matches_panel(f, app, chunks[4]);
    }

    draw_rule(f, chunks[5]);
    draw_footer(f, app, chunks[6]);
}