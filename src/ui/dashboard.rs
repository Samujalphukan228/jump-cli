// src/ui/dashboard.rs

use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem},
    Frame,
};
use std::path::PathBuf;
use std::time::Duration;

use crate::config::*;
use crate::store::Store;
use crate::ui::chrome::{
    clear, draw_brand_bar, draw_card_frame, draw_empty_state,
    draw_hint_bar, draw_rule, inset, progress_meter, Hint,
};
use crate::ui::helpers::{tui_init, tui_restore};
use crate::utils::shorten_path;

pub fn run_list_dashboard(store: &Store) -> std::io::Result<()> {
    let mut terminal = tui_init()?;
    loop {
        terminal.draw(|f| draw_list(f, store))?;
        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc | KeyCode::Enter => break,
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
                    _ => {}
                }
            }
        }
    }
    tui_restore(&mut terminal);
    Ok(())
}

fn draw_list(f: &mut Frame, store: &Store) {
    let area = f.area();
    clear(f, area);

    let shell = inset(area, 1, 0);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(6),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(shell);

    draw_brand_bar(
        f,
        chunks[0],
        "JUMP",
        "memory",
        Some(&format!("{} places", store.frecency.len())),
    );
    draw_rule(f, chunks[1]);

    let has_pins = !store.pins.is_empty();
    let body = if has_pins {
        let split = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(62), Constraint::Percentage(38)])
            .split(chunks[2]);
        draw_history_panel(f, store, split[0]);
        draw_pins_panel(f, store, split[1]);
        chunks[2]
    } else {
        draw_history_panel(f, store, chunks[2]);
        chunks[2]
    };
    let _ = body;

    draw_rule(f, chunks[3]);
    draw_hint_bar(
        f,
        chunks[4],
        &[
            Hint { key: "⏎", label: "close" },
            Hint { key: "esc", label: "close" },
        ],
        Some("frecency + pins"),
    );
}

fn draw_history_panel(f: &mut Frame, store: &Store, area: ratatui::layout::Rect) {
    let inner = draw_card_frame(f, area, "FREQUENCY");

    let mut entries: Vec<(&String, f64)> = store
        .frecency
        .iter()
        .map(|(k, _)| (k, store.frecency_score(k)))
        .filter(|(_, s)| *s > 0.0)
        .collect();
    entries.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    if entries.is_empty() {
        draw_empty_state(
            f,
            inner,
            "◎",
            "No history yet",
            "Jump somewhere — this panel learns where you go",
        );
        return;
    }

    let max = entries.first().map(|(_, s)| *s).unwrap_or(1.0);
    let items: Vec<ListItem> = entries
        .iter()
        .take(30)
        .enumerate()
        .map(|(i, (path, score))| {
            let visits = store.frecency.get(*path).map(|s| s.visits).unwrap_or(0);
            let short = shorten_path(&PathBuf::from(path.as_str()));
            let pb = PathBuf::from(path.as_str());
            let name = pb
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| path.to_string());
            let parent = short
                .rfind('/')
                .map(|p| format!("{}/", &short[..p]))
                .unwrap_or_default();
            let bar = progress_meter(*score, max, 12);
            ListItem::new(Line::from(vec![
                Span::styled(format!(" {:>02} ", i + 1), Style::default().fg(GHOST)),
                Span::styled(format!("{} ", bar), Style::default().fg(TEXT_DIM)),
                Span::styled(format!("{:>5.0} ", score), Style::default().fg(ACCENT_SOFT)),
                Span::styled(format!("{:>3}v ", visits), Style::default().fg(MUTED)),
                Span::styled(parent, Style::default().fg(MUTED)),
                Span::styled(
                    name,
                    Style::default().fg(TEXT).add_modifier(Modifier::BOLD),
                ),
            ]))
        })
        .collect();

    f.render_widget(List::new(items), inner);
}

fn draw_pins_panel(f: &mut Frame, store: &Store, area: ratatui::layout::Rect) {
    let inner = draw_card_frame(f, area, &format!("PINS · {}", store.pins.len()));

    let mut pins: Vec<(&String, &String)> = store.pins.iter().collect();
    pins.sort_by_key(|(k, _)| k.as_str());

    if pins.is_empty() {
        draw_empty_state(f, inner, "@", "No pins", "jump --pin name");
        return;
    }

    let items: Vec<ListItem> = pins.iter().map(|(name, path)| {
        let short = shorten_path(&PathBuf::from(path.as_str()));
        ListItem::new(Line::from(vec![
            Span::styled(
                format!(" @{:<8} ", name),
                Style::default().fg(TEXT_BRIGHT).add_modifier(Modifier::BOLD),
            ),
            Span::styled("→ ", Style::default().fg(GHOST)),
            Span::styled(short, Style::default().fg(TEXT_DIM)),
        ]))
    }).collect();

    f.render_widget(List::new(items), inner);
}